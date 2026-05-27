//! Wasm runner shim.
//!
//! Builds a wasm crate (selected with `-p`), serves it via cargo-run-wasm,
//! and opens it in the browser.
//!
//! Usage:
//!   cargo run_wasm -p native_webgl --release
//!
//! `body { margin: 0px; overflow: hidden; }` removes the default 8px page
//! gutter and the scrollbars that would otherwise appear because the canvas
//! is sized to inner_width/inner_height.
fn main() {
    cargo_run_wasm::run_wasm_cli_with_css(
        "body { margin: 0px; overflow: hidden; background: #111; }",
    );
}
