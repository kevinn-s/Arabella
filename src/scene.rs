use alloc::vec::Vec;
use fearless_simd::Level;
use lyon_geom::euclid::{Transform2D, UnknownUnit};
use lyon_path::{FillRule, Path};
use peniko::Color;

use crate::builder::Builder;
use crate::tile::Tile;

// ============================================================================
// Scene
// ============================================================================

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

    /// Fill a path with the given paint.
    pub fn fill(
        &mut self,
        path: &Path,
        fill_rule: FillRule,
        transform: Transform2D<f32, UnknownUnit, UnknownUnit>,
        color: Color,
    ) {
        // Register paint and get index
        let paint_index = self.paint_index_counter;
        self.paint_index_counter += 1;

        // Phase 1: Flatten + DDA binning → covers + blocks
        self.builder.build_path(path, fill_rule, transform);

        // Phase 2: Propagate covers → emit Tile structs
        self.builder.generate_tiles(paint_index, fill_rule);
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

     pub fn segment_list(&self) -> &[u32] {
        &self.builder.segment_list
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
