#![cfg(all(target_arch = "wasm32", feature = "webgl"))]

use lyon_path::{Path, path::Builder, geom::point};
use wasm_bindgen::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;
use wasm_bindgen::JsCast;
use web_sys::{HtmlCanvasElement, WebGl2RenderingContext};


use lyon_geom::euclid::{Transform2D, UnknownUnit};
use lyon_path::FillRule;
use peniko::Color;
use fearless_simd::Level;

use arabella::{
    Item, PicoSvg, RenderSize, Scene, WebGlRenderer
};

wasm_bindgen_test_configure!(run_in_browser);

/// A single SVG paint operation in document order.
///
/// SVG paths can declare both `fill` and `stroke`. The pico_svg parser splits
/// such paths into two separate `Item` entries (Fill then Stroke), preserving
/// document order. Rendering each `PaintOp` in the order they were collected
/// matches the SVG painter's algorithm: each later op composites on top of
/// previous ops via alpha blending.
enum PaintOp {
    Fill(FillItem),
    Stroke(StrokeFillItem),
}

/// Collected fill item with lyon_path::Path instead of kurbo::BezPath
struct FillItem {
    color: Color,
    path: Path,
    transform: Transform2D<f32, UnknownUnit, UnknownUnit>,
}

/// Collected stroke item with lyon_path::Path + kurbo::Stroke style.
///
/// SVG strokes carry a width (and per-spec, line cap / join / miter-limit
/// attributes which `pico_svg` doesn't currently parse). We fall back to
/// SVG defaults: cap = butt, join = miter, miter-limit = 4.
#[derive(Debug)]
struct StrokeFillItem {
    color: Color,
    path: Path,
    style: kurbo::Stroke,
    transform: Transform2D<f32, UnknownUnit, UnknownUnit>,
}

/// Walk the SVG tree in document order, emitting every fill and stroke as a
/// `PaintOp`. The original two-pass collector (collect all fills, then all
/// strokes) gave the wrong z-order: highlight overlays got drawn under their
/// base shapes. This single-pass collector preserves SVG-spec painter order.
///
/// We also DO NOT filter out white fills here. Translucent white fills are
/// the technique the Ghostscript_Tiger.svg uses to create highlights on the
/// orange face — skipping them erases the cheekbone / muzzle / ear shading.
fn collect_paint_ops(
    item: &Item,
    parent_transform: Transform2D<f32, UnknownUnit, UnknownUnit>,
    out: &mut Vec<PaintOp>,
) {
    match item {
        Item::Fill(fill) => {
            // Skip fills with effectively-zero alpha (truly invisible). We
            // do NOT skip white opaque fills any more — they may legitimately
            // be the page background, but they may also be highlight overlays
            // with reduced opacity that we want to keep.
            if fill.color.components[3] < 0.00 {
                return;
            }

            let lyon_path = kurbo_to_lyon(&fill.path);
            out.push(PaintOp::Fill(FillItem {
                color: fill.color,
                path: lyon_path,
                transform: parent_transform,
            }));
        }
        Item::Stroke(stroke) => {
            if stroke.color.components[3] < 0.005 {
                return;
            }

            let lyon_path = kurbo_to_lyon(&stroke.path);
            let style = kurbo::Stroke::new(stroke.width);
            out.push(PaintOp::Stroke(StrokeFillItem {
                color: stroke.color,
                path: lyon_path,
                style,
                transform: parent_transform,
            }));
        }
        Item::Group(group) => {
            // Compose the group's affine onto the parent transform and recurse.
            let coeffs = group.affine.as_coeffs();
            let group_tf = Transform2D::new(
                coeffs[0] as f32, coeffs[1] as f32,
                coeffs[2] as f32, coeffs[3] as f32,
                coeffs[4] as f32, coeffs[5] as f32,
            );
            let combined = parent_transform.then(&group_tf);

            for child in &group.children {
                collect_paint_ops(child, combined, out);
            }
        }
    }
}

fn kurbo_to_lyon(bez: &kurbo::BezPath) -> Path {
    let mut builder = Path::builder();
    for el in bez.elements() {
        match el {
            kurbo::PathEl::MoveTo(p) => {
                builder.begin(point(p.x as f32, p.y as f32));
            }
            kurbo::PathEl::LineTo(p) => {
                builder.line_to(point(p.x as f32, p.y as f32));
            }
            kurbo::PathEl::QuadTo(ctrl, to) => {
                builder.quadratic_bezier_to(
                    point(ctrl.x as f32, ctrl.y as f32),
                    point(to.x as f32, to.y as f32),
                );
            }
            kurbo::PathEl::CurveTo(c1, c2, to) => {
                builder.cubic_bezier_to(
                    point(c1.x as f32, c1.y as f32),
                    point(c2.x as f32, c2.y as f32),
                    point(to.x as f32, to.y as f32),
                );
            }
            kurbo::PathEl::ClosePath => {
                builder.end(true);
            }
        }
    }
    builder.build()
}

#[wasm_bindgen_test]
async fn test_renders_tiger_svg() {
    console_error_panic_hook::set_once();
    let _ = console_log::init_with_level(log::Level::Debug);

    const W: u16 = 1080;
    const H: u16 = 520;

    let performance = web_sys::window().unwrap().performance().unwrap();

    // ── Parse SVG ──
    // Switch back to the tiger SVG — it actually has strokes (thin black
    // outlines on the pink and white shapes), unlike SVG_Logo.svg which is
    // fill-only. This stresses the new Scene::stroke pipeline.
    let svg_str = include_str!("../assets/Ghostscript_Tiger.svg");
    let pico_svg = PicoSvg::load(svg_str, 1.0).expect("Failed to parse SVG");

    let scale = 3.0_f32;
    let base_transform = Transform2D::new(
        scale, 0.0,
        0.0, -scale,
        20.0, H as f32,
    );

    // Walk the SVG tree once in document order, collecting every fill and
    // stroke into a single ordered list. Document order is what the SVG
    // painter's algorithm expects — each later op composites on top of
    // previous ops via alpha blending.
    let mut paint_ops: Vec<PaintOp> = Vec::new();
    for item in &pico_svg.items {
        collect_paint_ops(item, base_transform, &mut paint_ops);
    }

    let n_fills = paint_ops.iter().filter(|op| matches!(op, PaintOp::Fill(_))).count();
    let n_strokes = paint_ops.iter().filter(|op| matches!(op, PaintOp::Stroke(_))).count();
    web_sys::console::log_1(&format!(
        "SVG parsed: {} paint ops ({} fills + {} strokes)",
        paint_ops.len(),
        n_fills,
        n_strokes,
    ).into());

    // ── Build scene (CPU binning) ──
    // Render every paint op in document order. Each fill / stroke composites
    // on top of previous ops via alpha blending.
    let mut scene = Scene::new(W, H);
    let t0 = performance.now();

    for op in &paint_ops {
        match op {
            PaintOp::Fill(f) => {
                scene.fill(&f.path, FillRule::NonZero, f.transform, f.color);
            }
            PaintOp::Stroke(s) => {
                scene.stroke(&s.path, &s.style, s.transform, s.color);
            }
        }
    }

    let t1 = performance.now();

    web_sys::console::log_1(&format!(
        "CPU Stage: {:.2} ms ({} fills + {} strokes)",
        t1 - t0,
        n_fills,
        n_strokes,
    ).into());


    // ── Create canvas + WebGL renderer ──
    let canvas = create_canvas(W as u32, H as u32, 1.0);
    let mut renderer = WebGlRenderer::new(&canvas);

    // ── Render ──
    let render_size = RenderSize {
        width: W as u32,
        height: H as u32,
    };

    let t2 = performance.now();
    renderer.render(&scene, &render_size);
    let t3 = performance.now();

    web_sys::console::log_1(&format!(
        "GPU render: {:.2} ms (upload + draw)",
        t3 - t2
    ).into());
}

// #[wasm_bindgen_test]
async fn test_renders_circle() {
    console_error_panic_hook::set_once();
    let _ = console_log::init_with_level(log::Level::Debug);

    const W: u16 = 400;
    const H: u16 = 400;

    let performance = web_sys::window().unwrap().performance().unwrap();

    // ── Build a circle path using lyon ──
    let mut builder = Path::builder();

    // Circle center (200, 200), radius 100
    // Approximated with 4 cubic beziers (standard circle approximation)
    let cx = 200.0_f32;
    let cy = 200.0_f32;
    let r = 100.0_f32;
    let k = 0.5522847498_f32; // magic number for cubic circle approximation

    builder.begin(point(cx + r, cy));
    builder.cubic_bezier_to(
        point(cx + r, cy + r * k),
        point(cx + r * k, cy + r),
        point(cx, cy + r),
    );
    builder.cubic_bezier_to(
        point(cx - r * k, cy + r),
        point(cx - r, cy + r * k),
        point(cx - r, cy),
    );
    builder.cubic_bezier_to(
        point(cx - r, cy - r * k),
        point(cx - r * k, cy - r),
        point(cx, cy - r),
    );
    builder.cubic_bezier_to(
        point(cx + r * k, cy - r),
        point(cx + r, cy - r * k),
        point(cx + r, cy),
    );
    builder.end(true);

    let circle_path = builder.build();

    // ── Build scene ──
    let mut scene = Scene::new(W, H);

    let transform = Transform2D::identity();

    scene.fill(
        &circle_path,
        FillRule::NonZero,
        transform,
        Color::from_rgb8(50, 100, 200),  // blue circle
    );
//  for (i, tile) in scene.tiles().iter().enumerate() {
//                     web_sys::console::log_1(
//                         &format!(
//                             "
//                             Tile #{}
//                             pos        = ({}, {})
//                             size       = {}x{}
//                             backdrop   = [{:.2}, {:.2}, {:.2}, {:.2}]
//                             segment    = [{}, {}]
//                             payload    = {}
//                             paint_flag = {}
//                             depth      = {}
//                             ",
//                             i,
//                             tile.x,
//                             tile.y,
//                             tile.width,
//                             tile.height,
//                             tile.backdrop[0],
//                             tile.backdrop[1],
//                             tile.backdrop[2],
//                             tile.backdrop[3],

//                             tile.segments[0],
//                             tile.segments[1],
//                             tile.payload,
//                             tile.paint_and_rect_flag,
//                             tile.depth_index,
//                         )
//                         .into(),
//                     );

//                     let seg_offset = tile.segments[1] as usize;
//                     let seg_count = tile.segments[0] as usize;

//                     for s in 0..seg_count {
//                         let seg_index = seg_offset + s;

//                         let base = seg_index * 6;

//                         if base + 5 < scene.segments().len() {
//                             let p0x = scene.segments()[base + 0];
//                             let p0y = scene.segments()[base + 1];

//                             let p1x = scene.segments()[base + 2];
//                             let p1y = scene.segments()[base + 3];

//                             let p2x = scene.segments()[base + 4];
//                             let p2y = scene.segments()[base + 5];

//                             web_sys::console::log_1(
//                                 &format!(
//                                     "
//                                         Tile #{}
//                                         Segment #{}
//                                             p0   = ({:.2}, {:.2})
//                                             ctrl = ({:.2}, {:.2})
//                                             p2   = ({:.2}, {:.2})
//                                         ",
//                                     i, seg_index, p0x, p0y, p1x, p1y, p2x, p2y,
//                                 )
//                                 .into(),
//                             );
//                         }
//                     }
//                 }

    let t0 = performance.now();
    let t1 = performance.now();

    web_sys::console::log_1(&format!(
        "Circle: {} tiles | {} segments | {:?} segment-list entries",
        scene.tiles().len(),
        scene.segments().len() / 8,
        scene.segments()
    ).into());

    // ── Create canvas + render ──
    let canvas = create_canvas(W as u32, H as u32, 1.0);
    let mut renderer = WebGlRenderer::new(&canvas);

    let render_size = RenderSize {
        width: W as u32,
        height: H as u32,
    };

    renderer.render(&scene, &render_size);

    web_sys::console::log_1(&"Circle rendered".into());
}

fn create_canvas(width: u32, height: u32, dpr: f64) -> HtmlCanvasElement {
    let document = web_sys::window().unwrap().document().unwrap();
    let canvas: HtmlCanvasElement = document
        .create_element("canvas")
        .unwrap()
        .dyn_into()
        .unwrap();

    canvas.set_width((width as f64 * dpr) as u32);
    canvas.set_height((height as f64 * dpr) as u32);

    canvas.style().set_property("width", &format!("{}px", width)).unwrap();
    canvas.style().set_property("height", &format!("{}px", height)).unwrap();
    canvas.style().set_property("border", "1px solid black").unwrap();

    document.body().unwrap().append_child(&canvas).unwrap();
    canvas
}
