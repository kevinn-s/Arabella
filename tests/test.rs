// ============================================================================
// Complete WebGL2 demo test for the custom vector rendering engine
// ============================================================================
//
// This file demonstrates how to set up the renderer end-to-end:
//   1. Create a canvas and WebGL2 context
//   2. Build a Scene and add a shape
//   3. Run the binning pipeline (CPU)
//   4. Upload tiles + segments to GPU
//   5. Issue one instanced draw call
//
// It's designed as a wasm-bindgen test that runs in a browser.
// ============================================================================
#![cfg(all(target_arch = "wasm32", feature = "webgl"))]

use kurbo::{BezPath, Triangle, Rect, PathEl};
use wasm_bindgen::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;
use wasm_bindgen::JsCast;
use web_sys::{HtmlCanvasElement, WebGl2RenderingContext};

use peniko::{
    Color, Fill,
    kurbo::{Affine, Circle},
};

use fearless_simd::Level;

use arabella::{
    Scene,
    Tile,
    WebGlRenderer,
    RenderSize,
    PicoSvg,
    Item
};


wasm_bindgen_test_configure!(run_in_browser);

// ============================================================================
// Test 1: Sanity — does the canvas show up?
// ============================================================================

// #[wasm_bindgen_test]
// async fn test_canvas_clears_to_known_color() {
//     console_error_panic_hook::set_once();
//     let _ = console_log::init_with_level(log::Level::Debug);

//     let canvas = create_canvas(200, 200);
//     let gl = get_webgl2_context(&canvas);

//     gl.viewport(0, 0, 200, 200);
//     gl.clear_color(0.0, 0.5, 1.0, 1.0); // sky blue
//     gl.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);

//     log::debug!("If you see a sky-blue canvas, WebGL2 setup works");
// }

// ============================================================================
// Test 2: Render one solid pink circle
// ============================================================================
struct FillItem {
    color: Color,
    path: BezPath, // The path to draw
}

// A simple SVG root, with all items to render.
struct Svg {
    items: Vec<FillItem>,
}
#[wasm_bindgen_test]
async fn test_renders_pink_circle() {

    console_error_panic_hook::set_once();
    let _ = console_log::init_with_level(log::Level::Debug);

    const W: u16 = 1080;
    const H: u16 = 720;

    let dpr = web_sys::window().unwrap().device_pixel_ratio();

        // ── Step 1: build the scene ──
    let mut scene = Scene::new(W, H);

    // Pink circle, well inside the viewport so we can see it
    let circle = Circle::new((400.0, 400.0), 300.0);
    let triangle = Triangle::new((20.0, 20.0), (50.0, 50.0), (100.0,100.0));
    let pink = Color::from_rgb8(242, 140, 168);

    scene.fill(
        Fill::NonZero,
        Affine::IDENTITY,
        pink,
        None,
        &circle,
    );




  

}

fn collect_fills(item: &Item, transform: Affine, out: &mut Vec<(Affine, Color, BezPath)>) {
    match item {
        Item::Fill(fill) => {
            out.push((transform, fill.color, fill.path.clone()));
        }
        Item::Stroke(_) => {
            // Skip strokes for now (your renderer doesn't support them yet)
        }
        Item::Group(group) => {
            let combined = transform * group.affine;
            for child in &group.children {
                collect_fills(child, combined, out);
            }
        }
    }
}

#[wasm_bindgen_test]
async fn test_renders_tiger_svg() {
    console_error_panic_hook::set_once();
    let _ = console_log::init_with_level(log::Level::Debug);
    const W: u16 = 1440;
    const H: u16 = 720;
    let dpr = web_sys::window().unwrap().device_pixel_ratio();
    log::debug!("Device pixel ratio: {}", dpr);

    // ── Step 1: build the scene ──
    let mut scene = Scene::new(W, H);

    let svg_str = include_str!("../assets/Ghostscript_Tiger.svg");
    let pico_svg = PicoSvg::load(svg_str, 1.0).expect("Failed to parse SVG");
let mut all_fills: Vec<(Affine, Color, BezPath)> = Vec::new();
for item in &pico_svg.items {
    collect_fills(item, Affine::IDENTITY, &mut all_fills);
}



        let pink = Color::from_rgb8(242, 140, 168);
    log::debug!("{:?}", all_fills.len());

    // Apply a fixed transform (e.g., scaling—as in SvgScene::tiger).
    let transform = Affine::scale(2.0);
log::info!("TILE DATA CHECK:");
log::info!(" - Size of u16: {}", core::mem::size_of::<u16>());
log::info!(" - Size of u32: {}", core::mem::size_of::<u32>());
log::info!(" - Calculated manual size: {}", (2*2) + (1*1+2) + (2*4) + (2*4) + 4 + 4 + 4);
log::info!(" - Actual Struct Stride: {}", core::mem::size_of::<Tile>());

for (svg_transform, color, path) in all_fills {
    scene.fill(
        Fill::NonZero,
        transform ,
        color,
        None,
        &path
    );
    log::info!("depth_index is {:}", scene.depth());
}
 
    // ── Step 2: create renderer ──
    let canvas = create_canvas(W as u32, H as u32, dpr);
    let mut renderer = WebGlRenderer::new(&canvas);

    // ── Step 3: render ──
    let render_size = arabella::RenderSize {
        width: (W as f64 * dpr) as u32,
        height: (H as f64 * dpr) as u32,
    };
    renderer.render(&mut scene, &render_size);

}

fn create_canvas(width: u32, height: u32, dpr: f64) -> HtmlCanvasElement {
    let document = web_sys::window().unwrap().document().unwrap();
    let canvas: HtmlCanvasElement = document
        .create_element("canvas")
        .unwrap()
        .dyn_into()
        .unwrap();
    
    // 1. Set the INTERNAL WebGL buffer to the high-res scaled size
    canvas.set_width((width as f64 * dpr) as u32);
    canvas.set_height((height as f64 * dpr) as u32);
    
    // 2. Set the CSS display size to the original logical size
    canvas
        .style()
        .set_property("width", &format!("{}px", width))
        .unwrap();
    canvas
        .style()
        .set_property("height", &format!("{}px", height))
        .unwrap();
    canvas
        .style()
        .set_property("border", "1px solid black")
        .unwrap();
        
    document.body().unwrap().append_child(&canvas).unwrap();
    canvas
}

fn get_webgl2_context(canvas: &HtmlCanvasElement) -> WebGl2RenderingContext {
    let context_options = js_sys::Object::new();
    js_sys::Reflect::set(&context_options, &"antialias".into(), &JsValue::FALSE).unwrap();
    js_sys::Reflect::set(&context_options, &"depth".into(), &JsValue::TRUE).unwrap();

    canvas
        .get_context_with_context_options("webgl2", &context_options)
        .unwrap()
        .unwrap()
        .dyn_into()
        .unwrap()
}