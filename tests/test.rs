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

/// Collected fill item with lyon_path::Path instead of kurbo::BezPath
struct FillItem {
    color: Color,
    path: Path,
    transform: Transform2D<f32, UnknownUnit, UnknownUnit>,
}

fn collect_fills(
    item: &Item,
    parent_transform: Transform2D<f32, UnknownUnit, UnknownUnit>,
    out: &mut Vec<FillItem>,
) {
    match item {
        Item::Fill(fill) => {
            let comps = fill.color.components;
            let is_white = comps[0] > 0.99 && comps[1] > 0.99 && comps[2] > 0.99 && comps[3] > 0.99;
            if is_white { return; }

            // Convert kurbo BezPath → lyon_path::Path
            let lyon_path = kurbo_to_lyon(&fill.path);
            out.push(FillItem {
                color: fill.color,
                path: lyon_path,
                transform: parent_transform,
            });
        }
        Item::Stroke(_) => {}
        Item::Group(group) => {
            // Combine transforms
            let affine = group.affine;
            let coeffs = affine.as_coeffs();
            let group_tf = Transform2D::new(
                coeffs[0] as f32, coeffs[1] as f32,
                coeffs[2] as f32, coeffs[3] as f32,
                coeffs[4] as f32, coeffs[5] as f32,
            );
            let combined = parent_transform.then(&group_tf);

            for child in &group.children {
                collect_fills(child, combined, out);
            }
        }
    }
}

/// Convert kurbo::BezPath to lyon_path::Path
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
    // Ensure path is closed if builder hasn't seen an explicit close
    builder.build()
}

// #[wasm_bindgen_test]
async fn test_cpu_binning_tiger_svg() {
    console_error_panic_hook::set_once();
    let _ = console_log::init_with_level(log::Level::Debug);

    const W: u16 = 1080;
    const H: u16 = 520;

    let performance = web_sys::window().unwrap().performance().unwrap();

    // ── Parse SVG ──
    let svg_str = include_str!("../assets/Ghostscript_Tiger.svg");
    let pico_svg = PicoSvg::load(svg_str, 1.0).expect("Failed to parse SVG");

    // ── Collect fills with lyon paths ──
    let scale = 1.0_f32;
    let base_transform = Transform2D::new(
        scale, 0.0,
        0.0, -scale,
        20.0, H as f32,
    );

    let mut fills: Vec<FillItem> = Vec::new();
    for item in &pico_svg.items {
        collect_fills(item, base_transform, &mut fills);
    }

    web_sys::console::log_1(
        &format!("Collected {} fill paths from tiger SVG", fills.len()).into()
    );

    // ── Benchmark CPU binning ──
    let mut scene = Scene::new(W, H);

    let t0 = performance.now();

    for fill_item in &fills {
        scene.fill(
            &fill_item.path,
            FillRule::NonZero,
            fill_item.transform,
            fill_item.color,
        );
    }

    let t1 = performance.now();

    let tile_count = scene.tiles().len();
    let segment_count = scene.segments().len() / 4;
    let segment_list_count = scene.segment_list().len();
    web_sys::console::log_1(&format!(
        "CPU binning: {:.2} ms | {} tiles | {} segments | {} segment-list entries",
        t1 - t0, tile_count, segment_count, scene.segment_list().len()
    ).into());


    // ── Run multiple iterations for stable timing ──
    let iterations = 10;
    let t_start = performance.now();
    for _ in 0..iterations {
        scene.reset();
        for fill_item in &fills {
            scene.fill(
                &fill_item.path,
                FillRule::NonZero,
                fill_item.transform,
                fill_item.color,
            );
        }
    }
    let t_end = performance.now();

    let avg_ms = (t_end - t_start) / iterations as f64;
    web_sys::console::log_1(&format!(
        "Average CPU binning over {} iterations: {:.2} ms ({:.1} FPS equivalent)",
        iterations, avg_ms, 1000.0 / avg_ms
    ).into());
}

#[wasm_bindgen_test]
async fn test_renders_tiger_svg() {
    console_error_panic_hook::set_once();
    let _ = console_log::init_with_level(log::Level::Debug);

    const W: u16 = 1080;
    const H: u16 = 520;

    let performance = web_sys::window().unwrap().performance().unwrap();

    // ── Parse SVG ──
    let svg_str = include_str!("../assets/Ghostscript_Tiger.svg");
    let pico_svg = PicoSvg::load(svg_str, 1.0).expect("Failed to parse SVG");

    let scale = 1.0_f32;
    let base_transform = Transform2D::new(
        scale, 0.0,
        0.0, -scale,
        20.0, H as f32,
    );

    let mut fills: Vec<FillItem> = Vec::new();
    for item in &pico_svg.items {
        collect_fills(item, base_transform, &mut fills);
    }

    web_sys::console::log_1(
        &format!("Collected {} fill paths from tiger SVG", fills.len()).into()
    );

    // ── Build scene (CPU binning) ──
    let mut scene = Scene::new(W, H);

    let t0 = performance.now();
    for fill_item in &fills {
        scene.fill(
            &fill_item.path,
            FillRule::NonZero,
            fill_item.transform,
            fill_item.color,
        );
    }
    let t1 = performance.now();

    web_sys::console::log_1(&format!(
        "CPU binning: {:.2} ms | {} tiles | {} segments | {} segment-list entries",
        t1 - t0,
        scene.tiles().len(),
        scene.segments().len() / 8,  // 8 floats per curve (2 texels)
        scene.segment_list().len()
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
