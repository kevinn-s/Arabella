#![no_std]
extern crate alloc;

mod scene;
#[cfg(target_arch = "wasm32")]
mod render;
mod paint;
mod path;
pub use render::{Config, Tile, RenderSize};
#[cfg(all(target_arch = "wasm32", feature = "webgl"))]
pub use render::webgl::WebGlRenderer;
pub use scene::{Scene};


use thiserror::Error;

/// Errors that can occur during rendering.
#[derive(Error, Debug)]
pub enum RenderError {
    /// No slots available for rendering.
    ///
    /// This error is likely to occur if a scene has an extreme number of nested layers
    /// (clipping, blending, masks, or opacity layers).
    ///
    /// TODO: Consider supporting more than a single column of slots in slot textures.
    #[error("No slots available for rendering")]
    SlotsExhausted,

}