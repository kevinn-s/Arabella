//! Wasm entry point for the Arabella benchmark.
//!
//! Run from the workspace root:
//!   cargo run_wasm -p bench_webgl --release
//!
//! cargo-run-wasm builds this as a wasm-bindgen cdylib, serves it locally,
//! and opens it in the browser. The benchmark runs once on load and prints
//! a Markdown table to the console and onto the page. The native target is
//! a no-op.

fn main() {
    #[cfg(target_arch = "wasm32")]
    {
        use bench_webgl::run_benchmark;

        console_error_panic_hook::set_once();
        let _ = console_log::init_with_level(log::Level::Debug);

        // Defer to the next task so the page body exists before we append
        // the canvas and results.
        wasm_bindgen_futures::spawn_local(async move {
            run_benchmark();
        });
    }
}
