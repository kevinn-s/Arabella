use core::cell::RefCell;

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use fearless_simd::Level;
use kurbo::{BezPath, PathEl, Point as KurboPoint, Stroke, StrokeCtx, StrokeOpts};
use lyon_geom::euclid::{Transform2D, UnknownUnit};
use lyon_path::geom::point as lyon_point;
use lyon_path::{FillRule, Path};
use peniko::{BrushRef, Color, Fill};

use crate::builder::{Builder, CoverStorage};
use crate::tile::Tile;

// ============================================================================
// Scene
// ============================================================================
const COLOR_SOURCE_PAYLOAD: u32 = 0;  // color stored inline in payload
const COLOR_SOURCE_PAINT: u32   = 1;  // color stored in encoded_paints

const PAINT_TYPE_SOLID: u32     = 0;
const PAINT_TYPE_LINEAR: u32    = 1;
const PAINT_TYPE_RADIAL: u32    = 2;
const PAINT_TYPE_SWEEP: u32     = 3;
const PAINT_TYPE_IMAGE: u32     = 4;

const FILL_RULE_NONZERO: u32  = 0;
const FILL_RULE_EVENODD: u32  = 1;

const COLOR_SOURCE_SHIFT: u32 = 30;
const PAINT_TYPE_SHIFT: u32 = 27;
const FILL_RULE_SHIFT: u32 = 24;

pub struct Scene {
    pub(crate) width: u16,
    pub(crate) height: u16,
    pub(crate) builder: Builder,
    paint_index_counter: u32,
}

impl Scene {
    pub fn new(width: u16, height: u16) -> Scene {
        let settings = RenderSettings::default();
        Scene {
            width,
            height,
            builder: Builder::new(width, height, settings.level),
            paint_index_counter: 0,
        }
    }
    fn encode_paint(&mut self, brush: BrushRef<'_>) -> (u32, u32) {
    match brush {
        BrushRef::Solid(color) => {
            // Pack RGBA8 (peniko's AlphaColor → premultiplied RGBA8)
            let rgba8 = color.to_rgba8();
            let payload = rgba8.to_u32();
            let flag = (COLOR_SOURCE_PAYLOAD << COLOR_SOURCE_SHIFT)
                | (PAINT_TYPE_SOLID << PAINT_TYPE_SHIFT);
            (payload, flag)
        }
        _ => {
            // TODO: gradients, images
            (0xFFFF00FFu32, 0)
                }
            }
        }

    /// Fill a path with the given paint.
    pub fn fill<'b>(
        &mut self,
        path: &Path,
        fill_rule: FillRule,
        transform: Transform2D<f32, UnknownUnit, UnknownUnit>,
        brush: impl Into<BrushRef<'b>>,
    ) {
        
        // Register paint and get index
        let paint_index = self.paint_index_counter;
    self.paint_index_counter += 1;
     let brush_ref: BrushRef = brush.into();
        let (payload, paint_flag) = self.encode_paint(brush_ref);
 
         let fill_rule_word = match fill_rule {
            FillRule::NonZero => FILL_RULE_NONZERO,
            FillRule::EvenOdd => FILL_RULE_EVENODD,
        };
        let final_paint_flag = paint_flag | (fill_rule_word << FILL_RULE_SHIFT);
        // Phase 1: Flatten + DDA binning → covers + blocks
        self.builder.build_path(path, fill_rule, transform);

        // Phase 2: Propagate covers → emit Tile structs
        self.builder.generate_tiles(paint_index, Fill::NonZero,   payload,
        paint_flag);

    }

    /// Stroke a path with the given paint, expanding the stroke geometry into
    /// a filled path via `kurbo::stroke_with` (which implements Raph Levien's
    /// parallel-curve stroking method, the same approach Vello uses).
    ///
    /// The expanded outline is then rendered through the regular fill
    /// pipeline. This means strokes share the entire fill rasterizer — no
    /// separate stroke-rendering code is needed.
    ///
    /// # Stroke style
    ///
    /// `style` controls width, line cap (butt/round/square), line join
    /// (miter/bevel/round), and miter limit. The kurbo `Stroke` type provides
    /// builder methods for each.
    ///
    /// # Tolerance
    ///
    /// The stroke-expansion tolerance is automatically derived from the
    /// transform's scale factor so that strokes look right after scaling.
    /// A larger transform → tighter tolerance → more curve subdivisions.
    pub fn stroke<'b>(
        &mut self,
        path: &Path,
        style: &Stroke,
        transform: Transform2D<f32, UnknownUnit, UnknownUnit>,
        brush: impl Into<BrushRef<'b>>,
    ) {
        // Convert the lyon path into a kurbo BezPath. kurbo's stroke
        // expander operates on `kurbo::PathEl` iterators.
        let kurbo_in = lyon_path_to_kurbo_bez(path);

        // Compute a sensible flattening tolerance for the stroke expansion,
        // accounting for the transform's scale so screen-space tolerance
        // stays roughly constant. Vello uses the same trick.
        const BASE_TOL: f64 = 0.25;
        let max_scale = (transform.m11.abs())
            .max(transform.m22.abs())
            .max(1.0) as f64;
        let tolerance = BASE_TOL / max_scale;

        // Expand stroke → fill. `stroke_with` writes the outline into
        // `stroke_ctx`, which we then iterate as a stream of PathEl.
        let mut stroke_ctx = StrokeCtx::default();
        kurbo::stroke_with(
            kurbo_in.iter(),
            style,
            &StrokeOpts::default(),
            tolerance,
            &mut stroke_ctx,
        );

        // Convert the expanded outline back to a lyon path so the fill
        // pipeline can consume it.
        //
        // kurbo's `BezPath::iter()` yields `PathEl` by value (not by
        // reference), so we pass the iterator directly — no `.copied()`
        // needed (and adding it would be a type error).
        let lyon_out = kurbo_iter_to_lyon_path(stroke_ctx.output().iter());

        // Render the outline as a non-zero fill. Strokes are always non-zero
        // because their boundary is closed and oriented consistently.
        self.fill(&lyon_out, FillRule::NonZero, transform, brush);
    }




    /// Get the generated tiles for GPU upload.
    pub fn tiles(&self) -> &[Tile] {
        self.builder.tiles.as_slice()
    }

    /// Get the curve segments buffer for GPU texture upload.
    pub fn segments(&self) -> &[f32] {
        &self.builder.segments
    }
    

    /// Reset scene for a new frame.
    pub fn reset(&mut self) {
        self.builder.reset();
        self.paint_index_counter = 0;
    }

    pub fn covers(&self) -> &RefCell<CoverStorage> {
    &self.builder.covers
}
     pub fn segment_list(&self) -> &[u32] {
        // segment_list is no longer used — clipped lines are stored directly
        // per-tile in `segments`. Kept as `&[]` for API compatibility with tests.
        &[]
    }
}

// ============================================================================
// RenderSettings
// ============================================================================

#[derive(Copy, Clone, Debug)]
pub struct RenderSettings {
    pub level: Level,
}

impl Default for RenderSettings {
    fn default() -> Self {
        Self {
            level: Level::try_detect().unwrap_or(Level::baseline()),
        }
    }
}

// ============================================================================
// lyon ↔ kurbo path conversions
//
// Used by `Scene::stroke` to bridge the type systems: arabella's fill
// pipeline takes `lyon_path::Path`, but kurbo's stroke expander produces
// and consumes `kurbo::PathEl`. The conversions are linear-time, no
// flattening or interpolation — just a 1:1 mapping of path elements.
// ============================================================================

/// Convert an arabella `lyon_path::Path` into a `kurbo::BezPath`.
fn lyon_path_to_kurbo_bez(path: &Path) -> BezPath {
    use lyon_path::PathEvent as LE;
    let mut out = BezPath::new();
    for ev in path.iter() {
        match ev {
            LE::Begin { at } => {
                out.move_to(KurboPoint::new(at.x as f64, at.y as f64));
            }
            LE::Line { to, .. } => {
                out.line_to(KurboPoint::new(to.x as f64, to.y as f64));
            }
            LE::Quadratic { ctrl, to, .. } => {
                out.quad_to(
                    KurboPoint::new(ctrl.x as f64, ctrl.y as f64),
                    KurboPoint::new(to.x as f64, to.y as f64),
                );
            }
            LE::Cubic { ctrl1, ctrl2, to, .. } => {
                out.curve_to(
                    KurboPoint::new(ctrl1.x as f64, ctrl1.y as f64),
                    KurboPoint::new(ctrl2.x as f64, ctrl2.y as f64),
                    KurboPoint::new(to.x as f64, to.y as f64),
                );
            }
            LE::End { close, .. } => {
                if close {
                    out.close_path();
                }
            }
        }
    }
    out
}

/// Convert a stream of `kurbo::PathEl` into a `lyon_path::Path`.
///
/// Used after `kurbo::stroke_with` produces the expanded stroke outline.
/// The kurbo stream uses `MoveTo` to mark sub-path boundaries, while lyon
/// uses explicit `begin` / `end` events; we translate by tracking the
/// "currently-open" sub-path state.
fn kurbo_iter_to_lyon_path(els: impl Iterator<Item = PathEl>) -> Path {
    let mut builder = Path::builder();
    let mut open = false;

    for el in els {
        match el {
            PathEl::MoveTo(p) => {
                if open {
                    // kurbo's MoveTo without a prior ClosePath leaves the
                    // sub-path "open" — close it on the lyon side without
                    // a closing segment.
                    builder.end(false);
                }
                builder.begin(lyon_point(p.x as f32, p.y as f32));
                open = true;
            }
            PathEl::LineTo(p) => {
                builder.line_to(lyon_point(p.x as f32, p.y as f32));
            }
            PathEl::QuadTo(c, p) => {
                builder.quadratic_bezier_to(
                    lyon_point(c.x as f32, c.y as f32),
                    lyon_point(p.x as f32, p.y as f32),
                );
            }
            PathEl::CurveTo(c1, c2, p) => {
                builder.cubic_bezier_to(
                    lyon_point(c1.x as f32, c1.y as f32),
                    lyon_point(c2.x as f32, c2.y as f32),
                    lyon_point(p.x as f32, p.y as f32),
                );
            }
            PathEl::ClosePath => {
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
