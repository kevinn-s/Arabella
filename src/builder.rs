use alloc::vec;
use alloc::vec::Vec;
use peniko::Fill;
use fearless_simd::*;

use crate::{
    blocks::{Block, Blocks, TileBounds, TILE_H, TILE_W, compute_row_backdrops},
    path::fill_impl,
    tile::{Tile, TileMap},
};
use lyon_geom::Box2D;
use lyon_path::{FillRule, Path};

use lyon_geom::euclid::{Transform2D, UnknownUnit};

const FILL_RULE_SHIFT: u32 = 24;

const FILL_RULE_NONZERO: u32 = 0;
const FILL_RULE_EVENODD: u32 = 1;

pub(crate) struct Builder {
    pub tiles: TileMap<Tile>,
    /// Curve control points for GPU texture (persists across shapes).
    /// 8 floats per curve: [p0.x, p0.y, p1.x, p1.y, p2.x, p2.y, 0, 0]
    pub segments: Vec<f32>,
    /// Per-tile segment index lists for GPU indirection texture.
    pub segment_list: Vec<u32>,
    /// Sparse tile records from analytical binning (cleared per shape).
    pub(crate) blocks: Blocks,
    /// Cached bounding box for the current shape.
    pub(crate) bbox: Box2D<f32>,
    pub(crate) level: Level,
    pub(crate) shape_index: u32,
}

impl Builder {
    pub(crate) fn new(width: u16, height: u16, level: Level) -> Builder {
        Builder {
            tiles: TileMap::new(|| Tile {
                x: 0,
                y: 0,
                width: 0,
                height: 0,
                _pad: [0, 0],
                backdrop: [0, 0],
                segments: [0.0, 0.0],
                payload: 0,
                paint_and_rect_flag: 0,
                depth_index: 0,
            }),
            segments: Vec::new(),
            segment_list: Vec::new(),
            blocks: Blocks::new(),
            bbox: Box2D::new(Point2D::new(0.0, 0.0), Point2D::new(0.0, 0.0)),
            level,
            shape_index: 0,
        }
    }

    pub fn build_path(
        &mut self,
        path: &Path,
        _fill_rule: FillRule,
        transform: Transform2D<f32, UnknownUnit, UnknownUnit>,
    ) {
        // ── Phase 1: Initialize bbox to "empty" ──
        self.bbox = Box2D::new(
            Point2D::new(f32::INFINITY, f32::INFINITY),
            Point2D::new(f32::NEG_INFINITY, f32::NEG_INFINITY),
        );

        // ── Phase 2: Process path → segments + bbox ──
        // Record the curve_id base before this shape's curves
        let curve_base = (self.segments.len() / 8) as u32;

        dispatch!(self.level, simd => {
            fill_impl(simd, path.iter(), transform, &mut self.segments, &mut self.bbox);
        });

        let curve_count_this_shape = (self.segments.len() / 8) as u32 - curve_base;
        if curve_count_this_shape == 0 {
            return;
        }

        if self.bbox.min.x >= self.bbox.max.x || self.bbox.min.y >= self.bbox.max.y {
            return;
        }

        // ── Phase 3: Compute tile bounds ──
        let tile_bounds = TileBounds::from_box2d(&self.bbox);
        if tile_bounds.col_count == 0 || tile_bounds.row_count == 0 {
            return;
        }

        // ── Phase 4: Analytical curve binning ──
        // For each curve in this shape, determine which tiles it touches
        // by intersecting it with the tile grid lines (GLLabel approach).
        self.blocks.reset();

        for i in 0..curve_count_this_shape {
            let curve_idx = curve_base + i;
            let base = (curve_idx as usize) * 8;
            let p0x = self.segments[base];
            let p0y = self.segments[base + 1];
            let p1x = self.segments[base + 2];
            let p1y = self.segments[base + 3];
            let p2x = self.segments[base + 4];
            let p2y = self.segments[base + 5];

            self.blocks.bin_curve(&tile_bounds, curve_idx, p0x, p0y, p1x, p1y, p2x, p2y);
        }

        // Sort and deduplicate: each (tile, curve) pair should appear once
        self.blocks.sort_blocks();
        self.blocks.dedup();
    }

    pub fn generate_tiles(
        &mut self,
        paint_index: u32,
        fill_rule: Fill,
        payload: u32,
        paint_flag: u32,
    ) {
        let fill_rule_word = match fill_rule {
            Fill::NonZero => FILL_RULE_NONZERO,
            Fill::EvenOdd => FILL_RULE_EVENODD,
        };
        let final_paint_flag = paint_flag | (fill_rule_word << FILL_RULE_SHIFT);

        let tile_bounds = TileBounds::from_box2d(&self.bbox);
        if tile_bounds.col_count == 0 || tile_bounds.row_count == 0 {
            return;
        }

        let curve_count = self.segments.len() / 8;

        // ── For each tile row, compute analytical backdrops ──
        for row in 0..tile_bounds.row_count {
            let global_tile_row = tile_bounds.min_row + row as i32;

            // Compute backdrop for every column in this row
            let row_backdrops = compute_row_backdrops(
                &self.segments,
                curve_count,
                global_tile_row,
                &tile_bounds,
            );

            for col in 0..tile_bounds.col_count {
                let global_x = (tile_bounds.min_col + col as i32) as u16;
                let global_y = global_tile_row as u16;

                // Find segments for this tile from the sorted blocks
                let (block_start, block_count) =
                    find_segment_range(&self.blocks.data, global_x, global_y);

                let backdrop = &row_backdrops[col];

                // Only emit tile if it has curves OR non-zero backdrop
                let has_backdrop = backdrop.iter().any(|&b| b != 0);

                if block_count == 0 && !has_backdrop {
                    continue;
                }

                // Build segment_list entries for this tile
                let list_offset = self.segment_list.len() as u32;
                for k in block_start..(block_start + block_count) {
                    self.segment_list.push(self.blocks.data[k].segment_id);
                }

                // Pack backdrop: 4 i16 values into 2 i32s
                // backdrop[0..1] → i32[0], backdrop[2..3] → i32[1]
                let packed_backdrop = [
                    (backdrop[0] as i32 & 0xFFFF) | ((backdrop[1] as i32 & 0xFFFF) << 16),
                    (backdrop[2] as i32 & 0xFFFF) | ((backdrop[3] as i32 & 0xFFFF) << 16),
                ];

                let current_depth = self.shape_index;
                self.shape_index += 1;

                self.tiles.push(Tile {
                    x: global_x,
                    y: global_y,
                    width: TILE_W as u8,
                    height: TILE_H as u8,
                    _pad: [0, 0],
                    backdrop: packed_backdrop,
                    segments: [
                        f32::from_bits(list_offset),
                        f32::from_bits(block_count as u32),
                    ],
                    payload,
                    paint_and_rect_flag: final_paint_flag,
                    depth_index: current_depth,
                });
            }
        }
    }

    pub fn reset(&mut self) {
        self.tiles.clear();
        self.segments.clear();
        self.segment_list.clear();
        self.blocks.reset();
        self.shape_index = 0;
    }
}

/// Binary search for blocks at tile position (x, y) in sorted blocks array.
fn find_segment_range(blocks: &[Block], x: u16, y: u16) -> (usize, usize) {
    let start = blocks.partition_point(|b| (b.y, b.x) < (y, x));
    let end = blocks.partition_point(|b| (b.y, b.x) <= (y, x));
    (start, end - start)
}
