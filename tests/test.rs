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

use kurbo::Triangle;
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
    RenderSize
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

#[wasm_bindgen_test]
async fn test_renders_pink_circle() {
    console_error_panic_hook::set_once();
    let _ = console_log::init_with_level(log::Level::Debug);

    const W: u16 = 1080;
    const H: u16 = 720;

    // ── Step 1: build the scene ──
    let mut scene = Scene::new(W, H);

    // Pink circle, well inside the viewport so we can see it
    let circle = Circle::new((100.0, 100.0), 100.0);
    let triangle = Triangle::new((20.0, 20.0), (50.0, 50.0), (100.0,100.0));
    let pink = Color::from_rgb8(242, 140, 168);

    scene.fill(
        Fill::NonZero,
        Affine::IDENTITY,
        pink,
        None,
        &circle,
    );

    


    log::debug!("Scene has {} tiles after binning", scene.tiles().len());
    log::debug!("Scene has {} segment floats", scene.segments().len());

    // ── Step 2: create renderer ──
    let canvas = create_canvas(W as u32, H as u32);
    let mut renderer = WebGlRenderer::new(&canvas);

    // ── Step 3: render ──
    let render_size = crate::RenderSize {
        width: W as u32,
        height: H as u32,
    };
    renderer.render(&mut scene, &render_size);

    log::debug!("BBRender complete — you should see a pink circle on the canvas");
}

// ============================================================================
// Test 3: Render multiple overlapping shapes (depth ordering)
// ============================================================================

// #[wasm_bindgen_test]
// async fn test_renders_overlapping_circles() {
//     console_error_panic_hook::set_once();
//     let _ = console_log::init_with_level(log::Level::Debug);

//     const W: u16 = 300;
//     const H: u16 = 300;
//     let mut scene = Scene::new(W, H);

//     // Three overlapping circles, painter's order: red behind, green middle, blue front
//     scene.fill(
//         Fill::NonZero,
//         Affine::IDENTITY,
//         Color::from_rgb8(220, 80, 80),
//         None,
//         &Circle::new((120.0, 150.0), 70.0),
//     );
//     scene.fill(
//         Fill::NonZero,
//         Affine::IDENTITY,
//         Color::from_rgb8(80, 200, 120),
//         None,
//         &Circle::new((180.0, 150.0), 70.0),
//     );
//     scene.fill(
//         Fill::NonZero,
//         Affine::IDENTITY,
//         Color::from_rgb8(80, 140, 220),
//         None,
//         &Circle::new((150.0, 200.0), 70.0),
//     );

//     let canvas = create_canvas(W as u32, H as u32);
//     let mut renderer = WebGlRenderer::new(&canvas);
//     let render_size = crate::RenderSize { width: W as u32, height: H as u32 };
//     renderer.render(&mut scene, &render_size);

//     log::debug!(
//         "Rendered {} tiles for 3 overlapping circles",
//         scene.tiles().len()
//     );
// }

// ============================================================================
// Helpers
// ============================================================================

fn create_canvas(width: u32, height: u32) -> HtmlCanvasElement {
    let document = web_sys::window().unwrap().document().unwrap();
    let canvas: HtmlCanvasElement = document
        .create_element("canvas")
        .unwrap()
        .dyn_into()
        .unwrap();
    canvas.set_width(width);
    canvas.set_height(height);
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