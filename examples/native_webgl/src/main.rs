//! Wasm entry point for the interactive Arabella demo.
//!
//! Run with `cargo run_wasm -p native_webgl --release` from the workspace
//! root. cargo-run-wasm builds this as a wasm-bindgen cdylib, serves it
//! locally, and opens it in the browser. The native target is a no-op.

#![allow(
    clippy::cast_possible_truncation,
    reason = "truncation is acceptable in this demo"
)]

fn main() {
    #[cfg(target_arch = "wasm32")]
    {
        use native_webgl::run_interactive;

        // Panic hook + log adapter — surfaces Rust panics + log macros to
        // the browser console.
        console_error_panic_hook::set_once();
        let _ = console_log::init_with_level(log::Level::Debug);

        let window = web_sys::window().unwrap();
        let dpr = window.device_pixel_ratio();
        let inner_w = window.inner_width().unwrap().as_f64().unwrap();
        let inner_h = window.inner_height().unwrap().as_f64().unwrap();
        let width = (inner_w * dpr) as u16;
        let height = (inner_h * dpr) as u16;

        wasm_bindgen_futures::spawn_local(async move {
            run_interactive(width, height).await;
        });
    }
}
