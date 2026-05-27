//! Interactive Arabella demo: pan + zoom + FPS overlay.
//!
//! Exposes a single entry point — `run_interactive(width, height)` — that
//! installs the canvas, FPS overlay, event handlers, and the
//! requestAnimationFrame loop, then returns. The page keeps running
//! because closures are leaked via `.forget()` and the JS event listeners
//! hold them alive.
//!
//! Run with `cargo run_wasm -p native_webgl --release` from the workspace
//! root. Mirrors `vello_hybrid/examples/native_webgl/src/lib.rs`.

#![allow(
    clippy::cast_possible_truncation,
    reason = "truncation is acceptable in this demo"
)]

use core::cell::RefCell;
use std::rc::Rc;

use lyon_geom::euclid::{Transform2D, UnknownUnit};
use lyon_path::{FillRule, Path, geom::point};
use peniko::Color;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::{
    Event, HtmlCanvasElement, HtmlElement, KeyboardEvent, MouseEvent, WheelEvent,
};

use arabella::{Item, PicoSvg, RenderSize, Scene, WebGlRenderer};

// ─────────────────────────────────────────────────────────────────────────────
// Paint-op collection
// ─────────────────────────────────────────────────────────────────────────────

enum PaintOp {
    Fill(FillItem),
    Stroke(StrokeFillItem),
}

struct FillItem {
    color: Color,
    path: Path,
    transform: Transform2D<f32, UnknownUnit, UnknownUnit>,
}

struct StrokeFillItem {
    color: Color,
    path: Path,
    style: kurbo::Stroke,
    transform: Transform2D<f32, UnknownUnit, UnknownUnit>,
}

fn collect_paint_ops(
    item: &Item,
    parent_transform: Transform2D<f32, UnknownUnit, UnknownUnit>,
    out: &mut Vec<PaintOp>,
) {
    match item {
        Item::Fill(fill) => {
            if fill.color.components[3] < 0.005 {
                return;
            }
            out.push(PaintOp::Fill(FillItem {
                color: fill.color,
                path: kurbo_to_lyon(&fill.path),
                transform: parent_transform,
            }));
        }
        Item::Stroke(stroke) => {
            if stroke.color.components[3] < 0.005 {
                return;
            }
            out.push(PaintOp::Stroke(StrokeFillItem {
                color: stroke.color,
                path: kurbo_to_lyon(&stroke.path),
                style: kurbo::Stroke::new(stroke.width),
                transform: parent_transform,
            }));
        }
        Item::Group(group) => {
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

/// Convert a kurbo `BezPath` into an arabella-compatible `lyon_path::Path`.
///
/// lyon requires every sub-path opened with `.begin(..)` to be closed with
/// `.end(..)` before `.build()` is called. Kurbo doesn't have that
/// constraint — a `MoveTo` implicitly ends the previous sub-path, and a
/// final sub-path without `ClosePath` is just an open polyline. We bridge
/// by tracking an `open` flag.
fn kurbo_to_lyon(bez: &kurbo::BezPath) -> Path {
    let mut builder = Path::builder();
    let mut open = false;
    for el in bez.elements() {
        match el {
            kurbo::PathEl::MoveTo(p) => {
                if open {
                    builder.end(false);
                }
                builder.begin(point(p.x as f32, p.y as f32));
                open = true;
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
                if open {
                    builder.end(true);
                    open = false;
                }
            }
        }
    }
    if open {
        builder.end(false);
    }
    builder.build()
}

// ─────────────────────────────────────────────────────────────────────────────
// Scene library
// ─────────────────────────────────────────────────────────────────────────────

struct SceneAsset {
    name: &'static str,
    paint_ops: Vec<PaintOp>,
    /// Base transform: scale + Y-flip + recenter into the canvas. Computed
    /// once per asset using the canvas size at startup.
    #[allow(dead_code)]
    base_transform: Transform2D<f32, UnknownUnit, UnknownUnit>,
}

fn load_assets(canvas_w: u32, canvas_h: u32) -> Vec<SceneAsset> {
    fn load(
        name: &'static str,
        svg: &str,
        scale: f32,
        canvas_w: u32,
        canvas_h: u32,
    ) -> SceneAsset {
        let pico = PicoSvg::load(svg, 1.0).expect("Failed to parse SVG");

        // Y-flip + translate so origin lands inside the canvas. The vertex
        // shader maps pixel_y=0 → NDC -1 (bottom), so we flip here.
        let tx = 20.0_f32;
        let ty = canvas_h as f32;
        let base_transform = Transform2D::new(scale, 0.0, 0.0, -scale, tx, ty);

        let mut paint_ops = Vec::new();
        for item in &pico.items {
            collect_paint_ops(item, base_transform, &mut paint_ops);
        }

        let _ = canvas_w; // currently unused; reserved for per-asset framing
        SceneAsset {
            name,
            paint_ops,
            base_transform,
        }
    }

    vec![
        load(
            "Ghostscript Tiger",
            include_str!("../../../assets/Ghostscript_Tiger.svg"),
            3.0,
            canvas_w,
            canvas_h,
        ),
        load(
            "SVG Logo",
            include_str!("../../../assets/SVG_Logo.svg"),
            2.0,
            canvas_w,
            canvas_h,
        ),
        load(
            "Bismillah",
            include_str!("../../../assets/bismillah.svg"),
            2.0,
            canvas_w,
            canvas_h,
        ),
    ]
}

// ─────────────────────────────────────────────────────────────────────────────
// AppState
// ─────────────────────────────────────────────────────────────────────────────

struct AppState {
    assets: Vec<SceneAsset>,
    current: usize,
    /// View transform applied AFTER the asset's base transform.
    view: Transform2D<f32, UnknownUnit, UnknownUnit>,
    mouse_down: bool,
    last_cursor: Option<(f64, f64)>,

    width: u32,
    height: u32,
    canvas: HtmlCanvasElement,
    scene: Scene,
    renderer: WebGlRenderer,

    last_frame_time: f64,
    fps_window: Vec<f64>,
    last_cpu_ms: f64,
    last_gpu_ms: f64,
    fps_overlay: HtmlElement,
    /// Counts frames so we only refresh the DOM overlay periodically — the
    /// FPS readout would be unreadable if updated every frame.
    frame_counter: u32,
}

impl AppState {
    fn new(
        canvas: HtmlCanvasElement,
        assets: Vec<SceneAsset>,
        fps_overlay: HtmlElement,
    ) -> Self {
        let width = canvas.width();
        let height = canvas.height();
        let scene = Scene::new(width as u16, height as u16);
        let renderer = WebGlRenderer::new(&canvas);
        Self {
            assets,
            current: 0,
            view: Transform2D::identity(),
            mouse_down: false,
            last_cursor: None,
            width,
            height,
            canvas,
            scene,
            renderer,
            last_frame_time: 0.0,
            fps_window: Vec::with_capacity(60),
            last_cpu_ms: 0.0,
            last_gpu_ms: 0.0,
            fps_overlay,
            frame_counter: 0,
        }
    }

    fn render(&mut self) {
        let perf = web_sys::window().unwrap().performance().unwrap();

        let cpu_start = perf.now();
        self.scene.reset();
        let asset = &self.assets[self.current];
        for op in &asset.paint_ops {
            match op {
                PaintOp::Fill(f) => {
                    let tf = f.transform.then(&self.view);
                    self.scene.fill(&f.path, FillRule::NonZero, tf, f.color);
                }
                PaintOp::Stroke(s) => {
                    let tf = s.transform.then(&self.view);
                    self.scene.stroke(&s.path, &s.style, tf, s.color);
                }
            }
        }
        let cpu_end = perf.now();

        let gpu_start = perf.now();
        let render_size = RenderSize {
            width: self.width,
            height: self.height,
        };
        self.renderer.render(&self.scene, &render_size);
        let gpu_end = perf.now();

        self.last_cpu_ms = cpu_end - cpu_start;
        self.last_gpu_ms = gpu_end - gpu_start;

        let now = perf.now();
        if self.last_frame_time > 0.0 {
            let dt = now - self.last_frame_time;
            self.fps_window.push(dt);
            if self.fps_window.len() > 60 {
                self.fps_window.remove(0);
            }
        }
        self.last_frame_time = now;

        self.frame_counter = self.frame_counter.wrapping_add(1);
        if self.frame_counter % 10 == 0 {
            self.update_overlay();
        }
    }

    fn update_overlay(&self) {
        let avg_dt = if self.fps_window.is_empty() {
            0.0
        } else {
            self.fps_window.iter().sum::<f64>() / self.fps_window.len() as f64
        };
        let fps = if avg_dt > 0.0 { 1000.0 / avg_dt } else { 0.0 };

        let asset = &self.assets[self.current];
        let scale_x = self.view.m11.hypot(self.view.m12);
        let html = format!(
            "<b>arabella</b> &nbsp; <span style='color:#9cf'>{}</span><br/>\
             FPS: <b>{:>5.1}</b> ({:>5.2} ms)<br/>\
             CPU: {:>5.2} ms &nbsp; GPU: {:>5.2} ms<br/>\
             zoom: {:.2}× &nbsp; ops: {}<br/>\
             <span style='color:#aaa'>drag pan · wheel zoom · space reset · ←/→ scene</span>",
            asset.name,
            fps,
            avg_dt,
            self.last_cpu_ms,
            self.last_gpu_ms,
            scale_x,
            asset.paint_ops.len(),
        );
        self.fps_overlay.set_inner_html(&html);
    }

    fn resize(&mut self, width: u32, height: u32) {
        self.canvas.set_width(width);
        self.canvas.set_height(height);
        self.width = width;
        self.height = height;
        self.scene = Scene::new(width as u16, height as u16);
    }

    fn next_scene(&mut self) {
        self.current = (self.current + 1) % self.assets.len();
        self.view = Transform2D::identity();
    }

    fn prev_scene(&mut self) {
        self.current = if self.current == 0 {
            self.assets.len() - 1
        } else {
            self.current - 1
        };
        self.view = Transform2D::identity();
    }

    fn reset_view(&mut self) {
        self.view = Transform2D::identity();
    }

    fn handle_mouse_down(&mut self, x: f64, y: f64) {
        self.mouse_down = true;
        self.last_cursor = Some((x, y));
    }

    fn handle_mouse_up(&mut self) {
        self.mouse_down = false;
    }

    fn handle_mouse_move(&mut self, x: f64, y: f64) {
        if self.mouse_down
            && let Some((lx, ly)) = self.last_cursor
        {
            let dx = (x - lx) as f32;
            // Scene/pixel space is y-up (vertex shader maps pixel_y=0 to
            // NDC -1 = bottom; base_transform Y-flips the SVG). Browser
            // cursors are y-down; negate so dragging down moves content
            // down on screen.
            let dy = -(y - ly) as f32;
            self.view = self.view.then_translate(lyon_geom::Vector::new(dx, dy));
        }
        self.last_cursor = Some((x, y));
    }

    fn handle_wheel(&mut self, delta_y: f64) {
        const ZOOM_STEP: f64 = 0.1;
        let zoom_factor = ((1.0 + delta_y * ZOOM_STEP).max(0.1)) as f32;

        let (cx_browser, cy_browser) = self.last_cursor.unwrap_or((
            self.width as f64 * 0.5,
            self.height as f64 * 0.5,
        ));
        // Convert cursor to scene-pixel space (y-up) so the world point
        // under the cursor stays locked while everything scales around it.
        let cx = cx_browser as f32;
        let cy = (self.height as f64 - cy_browser) as f32;

        // view' = T(c) · S(z) · T(-c) · view
        // With Transform2D's `then`-style composition (A.then(B) == B·A):
        //   view.then(T(-c)).then(S(z)).then(T(c))
        let v = self.view;
        let v = v.then_translate(lyon_geom::Vector::new(-cx, -cy));
        let v = v.then_scale(zoom_factor, zoom_factor);
        let v = v.then_translate(lyon_geom::Vector::new(cx, cy));
        self.view = v;
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Bootstrap
// ─────────────────────────────────────────────────────────────────────────────

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = requestAnimationFrame)]
    fn request_animation_frame(f: &Closure<dyn FnMut()>);
}

/// Build the canvas, FPS overlay, and event handlers. Returns once the
/// `requestAnimationFrame` loop is running and the closures are leaked
/// to JS. The page keeps running because JS holds the closures alive.
pub async fn run_interactive(canvas_width: u16, canvas_height: u16) {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let body = document.body().unwrap();
    body.style().set_property("background-color", "#111").unwrap();
    body.style().set_property("margin", "0").unwrap();
    body.style().set_property("overflow", "hidden").unwrap();

    let dpr = window.device_pixel_ratio();
    let width = canvas_width as u32;
    let height = canvas_height as u32;
    // CSS-pixel size = device-pixel size / dpr, used to keep the canvas
    // the same physical size on screen as the user expected.
    let css_w = (width as f64 / dpr).round() as u32;
    let css_h = (height as f64 / dpr).round() as u32;

    let canvas: HtmlCanvasElement = document
        .create_element("canvas")
        .unwrap()
        .dyn_into()
        .unwrap();
    canvas.set_width(width);
    canvas.set_height(height);
    canvas.style().set_property("display", "block").unwrap();
    canvas
        .style()
        .set_property("width", &format!("{css_w}px"))
        .unwrap();
    canvas
        .style()
        .set_property("height", &format!("{css_h}px"))
        .unwrap();
    canvas.style().set_property("touch-action", "none").unwrap();
    body.append_child(&canvas).unwrap();

    // FPS / info overlay (top-left).
    let overlay: HtmlElement = document
        .create_element("div")
        .unwrap()
        .dyn_into()
        .unwrap();
    overlay.set_inner_html("starting…");
    {
        let s = overlay.style();
        s.set_property("position", "fixed").unwrap();
        s.set_property("top", "10px").unwrap();
        s.set_property("left", "10px").unwrap();
        s.set_property("padding", "8px 12px").unwrap();
        s.set_property("background", "rgba(0,0,0,0.6)").unwrap();
        s.set_property("color", "#eee").unwrap();
        s.set_property("font-family", "ui-monospace, Menlo, Consolas, monospace")
            .unwrap();
        s.set_property("font-size", "12px").unwrap();
        s.set_property("line-height", "1.5").unwrap();
        s.set_property("border-radius", "6px").unwrap();
        s.set_property("pointer-events", "none").unwrap();
        s.set_property("z-index", "10").unwrap();
        s.set_property("white-space", "nowrap").unwrap();
    }
    body.append_child(&overlay).unwrap();

    // Build state.
    let assets = load_assets(width, height);
    web_sys::console::log_1(
        &format!(
            "Loaded {} assets ({} ops in first asset)",
            assets.len(),
            assets[0].paint_ops.len(),
        )
        .into(),
    );
    let app = Rc::new(RefCell::new(AppState::new(
        canvas.clone(),
        assets,
        overlay,
    )));

    // Animation loop.
    {
        let f: Rc<RefCell<Option<Closure<dyn FnMut()>>>> = Rc::new(RefCell::new(None));
        let g = f.clone();
        let app = app.clone();
        *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
            app.borrow_mut().render();
            request_animation_frame(f.borrow().as_ref().unwrap());
        }) as Box<dyn FnMut()>));
        request_animation_frame(g.borrow().as_ref().unwrap());
        Box::leak(Box::new(g));
    }

    // Helper: scale a client-coordinate event into canvas pixels (DPR-aware).
    fn to_canvas_xy(canvas: &HtmlCanvasElement, x: f64, y: f64) -> (f64, f64) {
        let rect = canvas.get_bounding_client_rect();
        let nx = (x - rect.left()) * (canvas.width() as f64 / rect.width());
        let ny = (y - rect.top()) * (canvas.height() as f64 / rect.height());
        (nx, ny)
    }

    // Mouse down (canvas-only; releasing outside still ends the drag, see below).
    {
        let app = app.clone();
        let canvas2 = canvas.clone();
        let closure = Closure::wrap(Box::new(move |ev: MouseEvent| {
            ev.prevent_default();
            let (x, y) = to_canvas_xy(&canvas2, ev.client_x() as f64, ev.client_y() as f64);
            app.borrow_mut().handle_mouse_down(x, y);
        }) as Box<dyn FnMut(_)>);
        canvas
            .add_event_listener_with_callback("mousedown", closure.as_ref().unchecked_ref())
            .unwrap();
        closure.forget();
    }

    // Mouse up (window-level so it fires even off-canvas).
    {
        let app = app.clone();
        let closure = Closure::wrap(Box::new(move |_ev: MouseEvent| {
            app.borrow_mut().handle_mouse_up();
        }) as Box<dyn FnMut(_)>);
        window
            .add_event_listener_with_callback("mouseup", closure.as_ref().unchecked_ref())
            .unwrap();
        closure.forget();
    }

    // Mouse move.
    {
        let app = app.clone();
        let canvas2 = canvas.clone();
        let closure = Closure::wrap(Box::new(move |ev: MouseEvent| {
            let (x, y) = to_canvas_xy(&canvas2, ev.client_x() as f64, ev.client_y() as f64);
            app.borrow_mut().handle_mouse_move(x, y);
        }) as Box<dyn FnMut(_)>);
        canvas
            .add_event_listener_with_callback("mousemove", closure.as_ref().unchecked_ref())
            .unwrap();
        closure.forget();
    }

    // Wheel — zoom centered on cursor.
    {
        let app = app.clone();
        let canvas2 = canvas.clone();
        let closure = Closure::wrap(Box::new(move |ev: WheelEvent| {
            ev.prevent_default();
            let (x, y) = to_canvas_xy(&canvas2, ev.client_x() as f64, ev.client_y() as f64);
            {
                let mut a = app.borrow_mut();
                a.last_cursor = Some((x, y));
            }
            let delta = -ev.delta_y() / 100.0;
            app.borrow_mut().handle_wheel(delta);
        }) as Box<dyn FnMut(_)>);
        canvas
            .add_event_listener_with_callback("wheel", closure.as_ref().unchecked_ref())
            .unwrap();
        closure.forget();
    }

    // Keyboard.
    {
        let app = app.clone();
        let closure = Closure::wrap(Box::new(move |ev: KeyboardEvent| {
            match ev.key().as_str() {
                "ArrowRight" => app.borrow_mut().next_scene(),
                "ArrowLeft" => app.borrow_mut().prev_scene(),
                " " => app.borrow_mut().reset_view(),
                _ => return,
            }
            ev.prevent_default();
        }) as Box<dyn FnMut(_)>);
        document
            .add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref())
            .unwrap();
        closure.forget();
    }

    // Window resize.
    {
        let app = app.clone();
        let canvas2 = canvas.clone();
        let closure = Closure::wrap(Box::new(move |_ev: Event| {
            let win = web_sys::window().unwrap();
            let dpr = win.device_pixel_ratio();
            let iw = win.inner_width().unwrap().as_f64().unwrap() as u32;
            let ih = win.inner_height().unwrap().as_f64().unwrap() as u32;
            let w = ((iw as f64) * dpr) as u32;
            let h = ((ih as f64) * dpr) as u32;
            canvas2
                .style()
                .set_property("width", &format!("{iw}px"))
                .unwrap();
            canvas2
                .style()
                .set_property("height", &format!("{ih}px"))
                .unwrap();
            app.borrow_mut().resize(w, h);
        }) as Box<dyn FnMut(_)>);
        window
            .add_event_listener_with_callback("resize", closure.as_ref().unchecked_ref())
            .unwrap();
        closure.forget();
    }

    web_sys::console::log_1(
        &"Interactive demo ready. Drag to pan, wheel to zoom, space to reset.".into(),
    );

    // Build-profile diagnostic so we can confirm we're in a real release
    // build, not a wasm-bindgen-test harness.
    web_sys::console::log_1(
        &format!(
            "[BUILD] debug_assertions = {}",
            cfg!(debug_assertions),
        )
        .into(),
    );
}
