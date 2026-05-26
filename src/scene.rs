use core::cell::RefCell;

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use fearless_simd::Level;
use lyon_geom::euclid::{Transform2D, UnknownUnit};
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
        // self.builder.reset();
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
