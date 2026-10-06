//! TrueType/OpenType text: system fonts found at run time (never bundled), glyph outlines as
//! closed polylines in CAD text units (text height = cap height).

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock, PoisonError};

use cadcraft_geom::Vec2;
use skrifa::instance::{LocationRef, Size};
use skrifa::outline::{DrawSettings, OutlinePen};
use skrifa::{FontRef, MetadataProvider};

use crate::Run;

/// Font bytes by lower-case family/file stem.
type Db = HashMap<String, Arc<Vec<u8>>>;

fn db() -> &'static Mutex<Option<Db>> {
    static DB: OnceLock<Mutex<Option<Db>>> = OnceLock::new();
    DB.get_or_init(|| Mutex::new(None))
}

#[cfg(not(target_arch = "wasm32"))]
fn scan() -> Db {
    let mut out = Db::new();
    let mut dirs: Vec<std::path::PathBuf> = Vec::new();
    if cfg!(target_os = "macos") {
        dirs.extend(["/System/Library/Fonts", "/System/Library/Fonts/Supplemental", "/Library/Fonts"].map(Into::into));
        if let Some(h) = std::env::var_os("HOME") {
            dirs.push(std::path::PathBuf::from(h).join("Library/Fonts"));
        }
    } else if cfg!(windows) {
        dirs.push("C:\\Windows\\Fonts".into());
    } else {
        dirs.extend(["/usr/share/fonts", "/usr/local/share/fonts"].map(Into::into));
        if let Some(h) = std::env::var_os("HOME") {
            dirs.push(std::path::PathBuf::from(&h).join(".fonts"));
            dirs.push(std::path::PathBuf::from(h).join(".local/share/fonts"));
        }
    }
    let mut stack = dirs;
    let mut seen = 0usize;
    while let Some(d) = stack.pop() {
        let Ok(rd) = std::fs::read_dir(&d) else { continue };
        for e in rd.flatten() {
            seen += 1;
            if seen > 20_000 {
                return out;
            }
            let p = e.path();
            if p.is_dir() {
                stack.push(p);
                continue;
            }
            let ext = p.extension().map(|x| x.to_string_lossy().to_ascii_lowercase()).unwrap_or_default();
            if ext != "ttf" && ext != "otf" {
                continue;
            }
            if let Some(stem) = p.file_stem().map(|s| s.to_string_lossy().to_ascii_lowercase()) {
                out.entry(stem).or_insert_with(|| Arc::new(Vec::new()));
                // Load lazily: store the path in a side table by using an empty marker.
                PATHS
                    .get_or_init(|| Mutex::new(HashMap::new()))
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner)
                    .insert(p.file_stem().map(|s| s.to_string_lossy().to_ascii_lowercase()).unwrap_or_default(), p.clone());
            }
        }
    }
    out
}

#[cfg(not(target_arch = "wasm32"))]
static PATHS: OnceLock<Mutex<HashMap<String, std::path::PathBuf>>> = OnceLock::new();

#[cfg(target_arch = "wasm32")]
fn scan() -> Db {
    Db::new()
}

/// Register font bytes under a name (web builds and tests).
pub fn register(name: &str, bytes: Vec<u8>) {
    let mut g = db().lock().unwrap_or_else(PoisonError::into_inner);
    g.get_or_insert_with(Db::new).insert(name.to_ascii_lowercase(), Arc::new(bytes));
}

/// Normalise a style font name ("arial.ttf", "Arial", "ARIALBD.TTF") to a lookup key.
fn key(name: &str) -> String {
    let n = name.trim().to_ascii_lowercase();
    n.trim_end_matches(".ttf").trim_end_matches(".otf").trim_end_matches(".ttc").to_string()
}

/// Bytes of a font by name, if installed. Stroke-font names (".shx", empty) return `None`.
pub fn find(name: &str) -> Option<Arc<Vec<u8>>> {
    let k = key(name);
    if k.is_empty() || k.ends_with(".shx") || k == crate::BUILTIN_FONT.to_ascii_lowercase() || k == "txt" || k == "simplex" || k == "romans" {
        return None;
    }
    let mut g = db().lock().unwrap_or_else(PoisonError::into_inner);
    let d = g.get_or_insert_with(scan);
    let candidates = [k.clone(), k.replace(' ', ""), format!("{k} regular"), format!("{}-regular", k.replace(' ', ""))];
    for c in &candidates {
        if let Some(b) = d.get(c) {
            if !b.is_empty() {
                return Some(b.clone());
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
                let path = PATHS.get().and_then(|m| m.lock().unwrap_or_else(PoisonError::into_inner).get(c).cloned())?;
                let bytes = std::fs::read(path).ok()?;
                let arc = Arc::new(bytes);
                d.insert(c.clone(), arc.clone());
                return Some(arc);
            }
        }
    }
    None
}

struct Pen {
    contours: Vec<Vec<Vec2>>,
    cur: Vec<Vec2>,
    scale: f64,
    dx: f64,
    wf: f64,
    shear: f64,
}

impl Pen {
    fn pt(&self, x: f32, y: f32) -> Vec2 {
        let y = f64::from(y) * self.scale;
        Vec2::new(self.dx + f64::from(x) * self.scale * self.wf + y * self.shear, y)
    }
    fn flush(&mut self) {
        if self.cur.len() > 1 {
            if let Some(f) = self.cur.first().copied() {
                self.cur.push(f);
            }
            self.contours.push(std::mem::take(&mut self.cur));
        } else {
            self.cur.clear();
        }
    }
}

impl OutlinePen for Pen {
    fn move_to(&mut self, x: f32, y: f32) {
        self.flush();
        let p = self.pt(x, y);
        self.cur.push(p);
    }
    fn line_to(&mut self, x: f32, y: f32) {
        let p = self.pt(x, y);
        self.cur.push(p);
    }
    fn quad_to(&mut self, cx0: f32, cy0: f32, x: f32, y: f32) {
        let a = self.cur.last().copied().unwrap_or_default();
        let c = self.pt(cx0, cy0);
        let b = self.pt(x, y);
        for i in 1..=6 {
            let t = f64::from(i) / 6.0;
            let u = 1.0 - t;
            self.cur.push(a * (u * u) + c * (2.0 * u * t) + b * (t * t));
        }
    }
    fn curve_to(&mut self, cx0: f32, cy0: f32, cx1: f32, cy1: f32, x: f32, y: f32) {
        let a = self.cur.last().copied().unwrap_or_default();
        let c0 = self.pt(cx0, cy0);
        let c1 = self.pt(cx1, cy1);
        let b = self.pt(x, y);
        for i in 1..=8 {
            let t = f64::from(i) / 8.0;
            let u = 1.0 - t;
            self.cur.push(a * (u * u * u) + c0 * (3.0 * u * u * t) + c1 * (3.0 * u * t * t) + b * (t * t * t));
        }
    }
    fn close(&mut self) {
        self.flush();
    }
}

/// Lay out one line with a TrueType font: closed glyph contours (baseline at y = 0).
pub fn layout_line(font: &[u8], s: &str, height: f64, width_factor: f64, oblique: f64) -> Option<Run> {
    let f = FontRef::new(font).ok()?;
    let upem = 1000.0f32;
    let metrics = f.metrics(Size::new(upem), LocationRef::default());
    let cap = metrics.cap_height.filter(|c| *c > 0.0).unwrap_or(metrics.ascent * 0.72).max(1.0);
    let scale = height / f64::from(cap);
    let wf = if width_factor.is_finite() && width_factor.abs() > 1e-6 { width_factor } else { 1.0 };
    let gm = f.glyph_metrics(Size::new(upem), LocationRef::default());
    let charmap = f.charmap();
    let outlines = f.outline_glyphs();
    let mut pen = Pen { contours: Vec::new(), cur: Vec::new(), scale, dx: 0.0, wf, shear: oblique.tan().clamp(-10.0, 10.0) };
    for (c, _, _) in crate::decode_controls(s) {
        let gid = charmap.map(c).unwrap_or_default();
        if let Some(g) = outlines.get(gid) {
            let _ = g.draw(DrawSettings::unhinted(Size::new(upem), LocationRef::default()), &mut pen);
            pen.flush();
        }
        pen.dx += f64::from(gm.advance_width(gid).unwrap_or(upem * 0.5)) * scale * wf;
    }
    Some(Run { strokes: pen.contours, width: pen.dx })
}
