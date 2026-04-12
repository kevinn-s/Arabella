use peniko::{
    BlendMode, Blob, Brush, BrushRef, Color, ColorStop, ColorStops, ColorStopsSource, Compose,
    Extend, Fill, FontData, Gradient, ImageBrush, ImageBrushRef, ImageData, StyleRef,
    color::{AlphaColor, DynamicColor, Srgb, palette},
    kurbo::{Affine, BezPath, Point, Rect, Shape, Stroke, StrokeOpts, Vec2, PathEl},
};

struct Scene {
  
}

impl Scene {

    pub fn fill<'b>(
        &mut self,
        style: Fill,
        transform: Affine,
        brush: impl Into<BrushRef<'b>>,
        brush_transform: Option<Affine>,
        shape: &impl Shape,
    ) { 
        
        
    }
}

