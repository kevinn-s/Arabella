#![no_std]
extern crate alloc;

mod scene;
#[cfg(target_arch = "wasm32")]
mod render;
mod paint;
mod path;
mod pico_svg;
mod tile;
mod builder;
mod blocks;
pub use render::{Config, RenderSize};
#[cfg(all(target_arch = "wasm32", feature = "webgl"))]
pub use render::webgl::WebGlRenderer;
pub use scene::{Scene};
pub use pico_svg::{PicoSvg, FillItem, Item, GroupItem, StrokeItem};
pub use paint::IndexedPaint;

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

const COLOR_SOURCE_SHIFT: u32 = 30;

pub const TILE_WIDTH: f32 = 4.0;
pub const TILE_HEIGHT: f32 = 4.0;

