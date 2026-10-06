# CADCraft roadmap

Status as of **2026-10-07**. CADCraft targets full parity with AutoCAD (2D drafting first, then
annotation, layouts and plotting, DWG, parametrics and 3D), plus things AutoCAD doesn't have:
agent control over MCP, a scriptable CLI, a web build and a free licence.

## Where we are

| Measure | Value |
|---|---|
| Menu breadth (`cargo xtask parity`, [docs/parity.md](docs/parity.md)) | **233 / 491 reference menu items (47%)** |
| Registered commands | 290 (133 with interactive prompts) |
| **Estimated overall feature parity (weighted by how much each area matters)** | **≈ 26%** |
| Tests | geometry, colour, document, fonts, render, DXF, engine (commands, prompts, snaps, selection, undo, scripts, hostile input), MCP agent tasks, xtask |
| Gates | `cargo xtask ci`: fmt, clippy -D warnings, tests, asset attribution, layering, wasm — green |

The weighted estimate counts depth: most covered menu items work in their common forms but lack
some options (e.g. ARC has 3-point and start-center-end but not all eleven variants; TRIM works on
lines, arcs, circles and polylines but not ellipses or splines; dimensions render but have no
interactive DIM commands yet).

## Milestones

| # | Milestone | Status | Estimate (Opus 5.5 wall-clock hours) |
|---|---|---|---|
| M0 | Skeleton + vertical slice: workspace, AutoCAD-style UI, command line, draw/modify basics, DXF, CLI, MCP, web | **done** | — |
| M1 | Drafting core: every Draw-menu 2D command and option, dynamic input fields, object snap tracking, temporary snap overrides | ~85% (all 2D Draw items exist) | 8 |
| M2 | Modify: grips (stretch/move/rotate/scale/mirror, multifunctional), PEDIT, SPLINEDIT, LENGTHEN, BLEND, ALIGN, associative arrays, trim/extend for all curve types, MATCHPROP UI, Quick Properties | ~65% (PEDIT, LENGTHEN, ALIGN, BLEND, grips API, TRIM on splines) | 18 |
| M3 | Layers & properties: full Layer Properties Manager (filters, VP overrides, states), linetype manager, lineweight display, transparency, QSELECT dialog, Properties for every object type | ~45% | 20 |
| M4 | Annotation: DIM* commands, DIMSTYLE manager, associative dimensions, MLEADER + styles, in-place MTEXT editor, fields, tables + table styles, annotative scaling, TrueType fonts, SHX reader | ~45% (TrueType text, MTEXT formatting codes, full DIMSTYLE variables + overrides, all arrowheads, associative dimensions, TABLE editing; not yet: DXF round-trip of the new data, jogged/ordinate polish, tolerance frames, fields, annotative scaling, SHX) | 35 |
| M5 | Hatch & blocks: pick-point boundary detection, islands, gradients, BLOCK/WBLOCK/INSERT dialogs, attributes (ATTDEF/ATTEDIT/BATTMAN), block editor, dynamic block parameters, xrefs, groups, Blocks palette | ~20% | 70 |
| M6 | Layouts & plotting: paper space, viewports (rect/polygonal/object, scale, lock, per-VP layers), page setups, PLOT to PDF/PNG/SVG, plot styles (CTB/STB), PUBLISH | ~55% (layouts, viewports, MSPACE/PSPACE through viewports, page setup, PLOT/EXPORTPDF) | 25 |
| M7 | Files: DWG read/write (done via acadrust), DXF fidelity (all object types, round-trip of unknown data), RECOVER/AUDIT, PURGE, templates, autosave, ETRANSMIT, PDF/raster underlays | ~45% | 60 |
| M8 | Inquiry & utilities: MEASUREGEOM, MASSPROP, QuickCalc, Find/Replace, spell check, COUNT, DWG Compare, Settings/OPTIONS, CUI-style customisation, alias editor | ~20% | 30 |
| M9 | Parametric: geometric + dimensional constraints, AutoConstrain, Parameters Manager (constraint solver, see plan/adr/0001) | ~60% (own solver crate, all GC*/DC* commands, AUTOCONSTRAIN, PARAMETERS, conflict detection, re-solve after edits; not yet: constraint glyph/dimension rendering, DWG/DXF persistence, inference while drawing) | 20 |
| M10 | Performance: GPU canvas (wgpu batches), R-tree spatial index, incremental regen, 1M-entity drawings at 60 fps | ~60% (GPU canvas, R-tree: pick 50 ms → 0.001 ms at 200k entities) | 15 |
| M11 | 3D: UCS, orbit, visual styles, solids (box…loft, booleans, fillet edges), meshes, surfaces, sections, rendering | 0% | 200 |
| M12 | Automation: an embedded safe AutoLISP-compatible interpreter, action recorder, sheet sets, CLI/MCP parity tests | ~25% | 60 |
| M13 | 1.0 polish: preferences, workspaces, themes, localisation, accessibility, signed packages for every platform, docs | ~10% | 40 |
| | **Remaining total** | | **≈ 620 hours** |

At roughly 620 more hours of Opus 5.5 wall-clock work (with parallel agents this compresses to
about 155–200 hours of elapsed time), CADCraft would reach broad AutoCAD parity. 2D drafting parity
(M1–M8) is the first ≈ 300 hours.

## Current focus

1. Constraint glyphs and dimensional constraints on the canvas; DXF persistence of constraints, DIMSTYLE overrides and associativity.
2. Quick Properties, QSELECT and the full Layer Properties Manager.
3. Plot preview; polygonal/object viewports.
4. 3D foundations (UCS, orbit, solids via truck/csgrs per plan/adr/0001).
5. First release-candidate build through the release pipeline.
