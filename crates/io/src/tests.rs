use cadcraft_color::Color;
use cadcraft_doc::*;
use cadcraft_geom::{PolyVertex, Vec2, Vec3};

use crate::*;

fn sample() -> Drawing {
    let mut d = Drawing::new_imperial();
    d.layers.push(Layer { name: "Walls".into(), color: Color::Index(1), ..Layer::default() });
    let c = |l: &str| Common { layer: l.into(), ..Common::default() };
    d.add(&Space::Model, c("Walls"), EntityKind::Line(Line { a: Vec3::new(0.0, 0.0, 0.0), b: Vec3::new(10.0, 5.0, 0.0) })).unwrap();
    d.add(&Space::Model, c("0"), EntityKind::Circle(Circle { center: Vec3::new(3.0, 4.0, 0.0), radius: 2.5 })).unwrap();
    d.add(&Space::Model, c("0"), EntityKind::Arc(Arc { center: Vec3::ZERO, radius: 1.0, start: 0.5, end: 2.0 })).unwrap();
    d.add(
        &Space::Model,
        c("0"),
        EntityKind::LwPolyline(LwPolyline {
            vertices: vec![PolyVertex::new(Vec2::ZERO), PolyVertex::with_bulge(Vec2::new(4.0, 0.0), 0.5), PolyVertex::new(Vec2::new(4.0, 3.0))],
            closed: true,
            const_width: 0.0,
            elevation: 0.0,
            plinegen: false,
        }),
    )
    .unwrap();
    d.add(
        &Space::Model,
        c("0"),
        EntityKind::Text(Text {
            insert: Vec3::new(1.0, 1.0, 0.0),
            align_pt: None,
            height: 0.25,
            value: "Hello Café".into(),
            rotation: 0.3,
            width_factor: 1.0,
            oblique: 0.0,
            style: "Standard".into(),
            halign: HAlign::Left,
            valign: VAlign::Baseline,
        }),
    )
    .unwrap();
    d.add(
        &Space::Model,
        c("0"),
        EntityKind::MText(MText {
            insert: Vec3::new(5.0, 5.0, 0.0),
            height: 0.2,
            width: 3.0,
            attach: 1,
            rotation: 0.0,
            style: "Standard".into(),
            contents: "line one\\Pline two".into(),
            line_spacing: 1.0,
        }),
    )
    .unwrap();
    d.add(
        &Space::Model,
        c("0"),
        EntityKind::Ellipse(Ellipse {
            center: Vec3::new(8.0, 8.0, 0.0),
            major: Vec3::new(3.0, 0.0, 0.0),
            ratio: 0.5,
            start: 0.0,
            end: std::f64::consts::TAU,
        }),
    )
    .unwrap();
    d.add(
        &Space::Model,
        c("0"),
        EntityKind::Spline(cadcraft_geom::Spline::from_fit_points(&[Vec2::ZERO, Vec2::new(1.0, 2.0), Vec2::new(3.0, 1.0), Vec2::new(4.0, 3.0)])),
    )
    .unwrap();
    d.add(
        &Space::Model,
        c("0"),
        EntityKind::Dimension(Dimension {
            kind: DimKind::Linear { rotation: 0.0 },
            defpt: Vec3::new(0.0, -1.0, 0.0),
            text_mid: Vec3::ZERO,
            p13: Vec3::ZERO,
            p14: Vec3::new(10.0, 0.0, 0.0),
            p15: Vec3::ZERO,
            p16: Vec3::ZERO,
            text: String::new(),
            style: "Standard".into(),
            measurement: 10.0,
            text_rotation: 0.0,
            user_text_pos: false,
            block: None,
            overrides: Default::default(),
            assoc: Vec::new(),
        }),
    )
    .unwrap();
    d.add(
        &Space::Model,
        c("0"),
        EntityKind::Hatch(Hatch {
            pattern: "ANSI31".into(),
            solid: false,
            loops: vec![HatchLoop {
                vertices: vec![PolyVertex::new(Vec2::ZERO), PolyVertex::new(Vec2::new(2.0, 0.0)), PolyVertex::new(Vec2::new(2.0, 2.0))],
                outer: true,
            }],
            scale: 1.0,
            angle: 0.0,
            associative: false,
            style: 0,
            elevation: 0.0,
            gradient: None,
            origin: Vec2::ZERO,
            background: None,
        }),
    )
    .unwrap();
    let mut b = Block::new("Bolt");
    b.entities.push(Entity::new(Handle(0x50), EntityKind::Circle(Circle { center: Vec3::ZERO, radius: 0.25 })));
    d.blocks.insert("Bolt".into(), std::sync::Arc::new(b));
    d.add(
        &Space::Model,
        c("0"),
        EntityKind::Insert(Insert {
            block: "Bolt".into(),
            insert: Vec3::new(2.0, 2.0, 0.0),
            scale: Vec3::new(1.0, 1.0, 1.0),
            rotation: 0.0,
            attribs: vec![],
            cols: 1,
            rows: 1,
            col_spacing: 0.0,
            row_spacing: 0.0,
        }),
    )
    .unwrap();
    d.add(&Space::Paper("Layout1".into()), c("0"), EntityKind::Line(Line { a: Vec3::ZERO, b: Vec3::new(1.0, 1.0, 0.0) })).unwrap();
    d
}

#[test]
fn dxf_roundtrip_preserves_entities() {
    let d = sample();
    let text = write_dxf(&d);
    let back = read_dxf(text.as_bytes()).unwrap();
    let kinds = |d: &Drawing| d.model.iter().map(|e| e.kind.type_name()).collect::<Vec<_>>();
    assert_eq!(kinds(&back), kinds(&d));
    assert_eq!(back.layer("Walls").unwrap().color, Color::Index(1));
    assert!(back.block("Bolt").is_some());
    assert_eq!(back.layout("Layout1").unwrap().entities.len(), 1);
    // Geometry survives.
    for (a, b) in d.model.iter().zip(back.model.iter()) {
        match (&a.kind, &b.kind) {
            (EntityKind::Line(x), EntityKind::Line(y)) => assert_eq!(x, y),
            (EntityKind::Circle(x), EntityKind::Circle(y)) => assert_eq!(x, y),
            (EntityKind::LwPolyline(x), EntityKind::LwPolyline(y)) => assert_eq!(x.vertices, y.vertices),
            (EntityKind::Text(x), EntityKind::Text(y)) => {
                assert_eq!(x.value, y.value);
                assert!((x.rotation - y.rotation).abs() < 1e-9);
            }
            (EntityKind::MText(x), EntityKind::MText(y)) => assert_eq!(x.contents, y.contents),
            (EntityKind::Hatch(x), EntityKind::Hatch(y)) => assert_eq!(x.loops, y.loops),
            _ => {}
        }
        assert_eq!(a.handle, b.handle);
        assert_eq!(a.common.layer, b.common.layer);
    }
    // Handles stay unique after reading.
    let mut back = back;
    let h = back.new_handle();
    assert!(back.entity(h).is_none());
}

#[test]
fn second_roundtrip_is_stable() {
    let d = sample();
    let t1 = write_dxf(&d);
    let d2 = read_dxf(t1.as_bytes()).unwrap();
    let d3 = read_dxf(write_dxf(&d2).as_bytes()).unwrap();
    assert_eq!(d2.model.len(), d3.model.len());
}

#[test]
fn reads_r12_style_polyline_and_paper_flag() {
    let text = "0\nSECTION\n2\nENTITIES\n0\nPOLYLINE\n8\n0\n66\n1\n70\n1\n0\nVERTEX\n8\n0\n10\n0\n20\n0\n0\nVERTEX\n8\n0\n10\n5\n20\n0\n42\n1\n0\nVERTEX\n8\n0\n10\n5\n20\n5\n0\nSEQEND\n0\nLINE\n67\n1\n8\n0\n10\n0\n20\n0\n11\n1\n21\n1\n0\nENDSEC\n0\nEOF\n";
    let d = read(text.as_bytes(), "a.dxf").unwrap();
    assert_eq!(d.model.len(), 1);
    match &d.model.iter().next().unwrap().kind {
        EntityKind::LwPolyline(p) => {
            assert_eq!(p.vertices.len(), 3);
            assert!(p.closed);
            assert_eq!(p.vertices[1].bulge, 1.0);
        }
        _ => panic!(),
    }
    assert_eq!(d.layouts[0].entities.len(), 1);
}

#[test]
fn hostile_dxf_does_not_panic() {
    for t in [
        "0\nSECTION\n2\nENTITIES\n0\nHATCH\n91\n999999999\n92\n2\n93\n99999\n0\nENDSEC\n0\nEOF\n",
        "0\nSECTION\n2\nENTITIES\n0\nSPLINE\n71\n99\n40\n1\n10\n0\n20\n0\n0\nENDSEC\n0\nEOF\n",
        "0\nSECTION\n2\nENTITIES\n0\nINSERT\n66\n1\n2\nX\n0\nATTRIB\n0\nENDSEC\n",
        "0\nSECTION\n2\nBLOCKS\n0\nBLOCK\n2\nA\n0\nINSERT\n2\nA\n0\nENDBLK\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nINSERT\n2\nA\n0\nENDSEC\n0\nEOF\n",
        "0\nSECTION\n2\nENTITIES\n0\nLWPOLYLINE\n42\n5\n20\n1\n0\nENDSEC\n0\nEOF\n",
    ] {
        if let Ok(d) = read(t.as_bytes(), "x.dxf") {
            let _ = cadcraft_render::build(&d, &Space::Model, &cadcraft_render::Options::default());
            let _ = d.extents(&Space::Model);
        }
    }
}

#[test]
fn svg_and_png_export() {
    let d = sample();
    let svg = String::from_utf8(write(&d, "a.svg").unwrap()).unwrap();
    assert!(svg.starts_with("<svg") && svg.contains("polyline"));
    let png = write(&d, "a.png").unwrap();
    assert_eq!(&png[1..4], b"PNG");
}

/// Structural sanity check of a PDF: header, object count, xref offsets pointing at `n 0 obj`.
fn check_pdf(bytes: &[u8]) -> String {
    assert!(bytes.starts_with(b"%PDF-1.4"));
    let find = |pat: &[u8]| bytes.windows(pat.len()).position(|w| w == pat);
    let rfind = |pat: &[u8]| bytes.windows(pat.len()).rposition(|w| w == pat);
    let tail = String::from_utf8_lossy(&bytes[bytes.len().saturating_sub(64)..]).to_string();
    assert!(tail.trim_end().ends_with("%%EOF"));
    let sx = rfind(b"startxref\n").unwrap();
    let after = String::from_utf8_lossy(&bytes[sx + 10..]).to_string();
    let xref_off: usize = after.lines().next().unwrap().trim().parse().unwrap();
    assert!(bytes[xref_off..].starts_with(b"xref\n"));
    let xref = String::from_utf8_lossy(&bytes[xref_off..]).to_string();
    let mut lines = xref.lines().skip(1);
    let count: usize = lines.next().unwrap().split_whitespace().nth(1).unwrap().parse().unwrap();
    let objs = bytes.windows(7).filter(|w| w == b" 0 obj\n").count();
    assert_eq!(count, objs + 1, "xref size = objects + free entry");
    assert_eq!(bytes.windows(6).filter(|w| w == b"endobj").count(), objs);
    let entries: Vec<&str> = lines.take(count).collect();
    assert!(entries[0].starts_with("0000000000 65535 f"));
    for (i, e) in entries.iter().enumerate().skip(1) {
        let off: usize = e[..10].parse().unwrap();
        assert!(bytes[off..].starts_with(format!("{i} 0 obj").as_bytes()), "object {i} offset");
    }
    assert!(xref.contains(&format!("/Size {count}")));
    // The content stream's /Length matches its data.
    let sp = find(b"/Length ").unwrap();
    let after = String::from_utf8_lossy(&bytes[sp + 8..sp + 30]).to_string();
    let len: usize = after.split(|c: char| !c.is_ascii_digit()).next().unwrap().parse().unwrap();
    let start = find(b"stream\n").unwrap() + 7;
    assert!(bytes[start + len..].starts_with(b"\nendstream"));
    let compressed = find(b"/FlateDecode").is_some();
    let data = &bytes[start..start + len];
    if compressed {
        String::from_utf8(miniz_oxide::inflate::decompress_to_vec_zlib(data).unwrap()).unwrap()
    } else {
        String::from_utf8(data.to_vec()).unwrap()
    }
}

fn media_box(bytes: &[u8]) -> (f64, f64) {
    let text = String::from_utf8_lossy(bytes).to_string();
    let i = text.find("/MediaBox [0 0 ").unwrap() + 15;
    let v: Vec<f64> = text[i..].split(']').next().unwrap().split_whitespace().map(|x| x.parse().unwrap()).collect();
    (v[0], v[1])
}

#[test]
fn pdf_model_fitted_to_sheet() {
    let d = sample();
    let bytes = plot(&d, &Space::Model, &serde_json::json!({"paper": "A4", "landscape": true})).unwrap();
    let content = check_pdf(&bytes);
    let (w, h) = media_box(&bytes);
    assert!((w - 841.89).abs() < 0.01 && (h - 595.276).abs() < 0.01, "A4 landscape in points: {w} x {h}");
    assert!(content.contains(" m\n") && content.contains(" l\n") && content.contains("S\n"));
    // Walls layer is red; colour 7 prints black.
    assert!(content.contains("1 0 0 RG"));
    assert!(content.contains("0 0 0 RG"));
    // Uncompressed output is plain text and also valid.
    let raw = plot(&d, &Space::Model, &serde_json::json!({"paper": "Letter", "compress": false, "landscape": false})).unwrap();
    let c2 = check_pdf(&raw);
    assert!(!String::from_utf8_lossy(&raw).contains("FlateDecode"));
    assert_eq!(media_box(&raw), (612.0, 792.0));
    assert!(c2.contains("re W n"));
    // write() picks PDF by extension.
    assert!(write(&d, "x.pdf").unwrap().starts_with(b"%PDF"));
}

#[test]
fn pdf_layout_one_to_one_with_viewport() {
    let mut d = sample();
    let paper = Space::Paper("Layout1".into());
    d.layouts[0].page.lineweights = true;
    d.add(
        &paper,
        Common::default(),
        EntityKind::Viewport(Viewport {
            center: Vec3::new(5.0, 4.0, 0.0),
            width: 8.0,
            height: 6.0,
            view_center: Vec2::new(5.0, 2.5),
            view_height: 12.0,
            id: 2,
            locked: false,
            frozen_layers: Vec::new(),
            layer_colors: Vec::new(),
        }),
    )
    .unwrap();
    let bytes = plot(&d, &paper, &serde_json::json!({})).unwrap();
    let content = check_pdf(&bytes);
    // ANSI A landscape (inches → points).
    assert_eq!(media_box(&bytes), (792.0, 612.0));
    // Viewport border at 1:1: x from 1in to 9in = 72pt..648pt.
    assert!(content.contains("72 72 m") || content.contains("72 72 l"), "{content}");
    // Lineweights on: default 0.25 mm = 0.709 pt.
    assert!(content.contains("0.709 w"));
    let off = plot(&d, &paper, &serde_json::json!({"lineweights": false})).unwrap();
    assert!(check_pdf(&off).contains("0 w"));
    assert!(plot(&d, &Space::Paper("Nope".into()), &serde_json::json!({})).is_err());
    assert!(plot(&d, &paper, &serde_json::json!({"paper": "Z9"})).is_err());
}

#[test]
fn pdf_hostile_options_and_empty() {
    let d = Drawing::new_metric();
    let bytes = plot(&d, &Space::Model, &serde_json::json!({})).unwrap();
    check_pdf(&bytes);
    let (w, h) = media_box(&bytes);
    assert!((w - 841.89).abs() < 0.01 && (h - 595.276).abs() < 0.01, "metric default is A4 landscape");
    let s = sample();
    for o in [
        serde_json::json!({"width": 1e308, "height": -5}),
        serde_json::json!({"width": 1e308, "height": 1e308, "scale": 1e308, "fit": false}),
        serde_json::json!({"scale": -1, "fit": false, "title": "a(b)\\c\u{e9}"}),
        serde_json::json!(null),
        serde_json::json!([1, 2]),
    ] {
        check_pdf(&plot(&s, &Space::Model, &o).unwrap());
    }
}

#[test]
fn dxf_roundtrips_page_setup() {
    let mut d = sample();
    let a3 = cadcraft_render::paper_size("A3").unwrap();
    {
        let p = &mut d.layouts[0].page;
        p.paper = a3.name.into();
        p.width_mm = a3.width_mm;
        p.height_mm = a3.height_mm;
        p.landscape = false;
        p.margins_mm = [5.0, 6.0, 7.0, 8.0];
    }
    let back = read_dxf(write_dxf(&d).as_bytes()).unwrap();
    let p = &back.layout("Layout1").unwrap().page;
    assert_eq!((p.width_mm, p.height_mm, p.landscape), (297.0, 420.0, false));
    assert_eq!(p.margins_mm, [5.0, 6.0, 7.0, 8.0]);
    assert_eq!(p.paper, a3.name);
}
