
pub(crate) mod encode;
pub use encode::{
    EncodedPaint, EncodedGradient, EncodedBlurredRoundedRectangle, EncodedImage, EncodedKind,
    RadialKind, FocalData, FromF32Color
};
mod simd;
mod math;

pub(crate) mod paint;
pub use paint::{IndexedPaint};

mod pixmap;

pub(crate) mod gradient_cache;
