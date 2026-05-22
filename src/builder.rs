use alloc::vec;
use alloc::vec::Vec;
use peniko::Fill;
use core::cell::RefCell;
use fearless_simd::*;

use crate::{
    blocks::{Block, Blocks, TileBounds},
    path::fill_impl,
    tile::{Tile, TileMap},
};
use lyon_geom::Box2D;
use lyon_path::{Event, FillRule, Iter, Path, PathEvent, polygon::PathEvents};

const COLOR_SOURCE_SHIFT: u32 = 30;
const PAINT_TYPE_SHIFT: u32 = 27;
const FILL_RULE_SHIFT: u32 = 24;

const COLOR_SOURCE_PAYLOAD: u32 = 0;
const COLOR_SOURCE_PAINT: u32 = 1;

const PAINT_TYPE_SOLID: u32 = 0;
const PAINT_TYPE_LINEAR: u32 = 1;
const PAINT_TYPE_RADIAL: u32 = 2;
const PAINT_TYPE_SWEEP: u32 = 3;
const PAINT_TYPE_IMAGE: u32 = 4;

const FILL_RULE_NONZERO: u32 = 0;
const FILL_RULE_EVENODD: u32 = 1;

use fearless_simd::*;
use lyon_geom::euclid::default::Point2D;

use lyon_geom::euclid::{Transform2D, UnknownUnit};

pub(crate) struct Builder {
    pub tiles: TileMap<Tile>,
    /// Curve control points for GPU texture (persists across shapes).
    pub segments: Vec<f32>,
    /// Per-tile segment index lists for GPU indirection texture.
    pub segment_list: Vec<u32>,
    /// Flattened line endpoints in F24Dot8 (cleared per shape).
    pub(crate) line_buf: Vec<i32>,
    /// Sparse tile records from DDA binning (cleared per shape).
    pub(crate) blocks: Blocks,
    /// Per-shape cover accumulation.
    pub(crate) covers: RefCell<CoverStorage>,
    /// Cached bounding box for the current shape.
    pub(crate) bbox: Box2D<f32>,
    pub(crate) level: Level,
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
            line_buf: Vec::new(),
            blocks: Blocks {
                data: Vec::with_capacity(16384),
                sorted: false,
            },
            covers: RefCell::new(CoverStorage::new()),
            bbox: Box2D::new(Point2D::new(0.0, 0.0), Point2D::new(0.0, 0.0)),
            level,
        }
    }

    pub fn build_path(
        &mut self,
        path: &Path,
        fill_rule: FillRule,
        transform: Transform2D<f32, UnknownUnit, UnknownUnit>,
    ) {
        // ── Phase 1: Initialize bbox to "empty" ──
        self.bbox = Box2D::new(
            Point2D::new(f32::INFINITY, f32::INFINITY),
            Point2D::new(f32::NEG_INFINITY, f32::NEG_INFINITY),
        );

        // ── Phase 2: Process path → segments + line_buf + bbox ──
        self.line_buf.clear();
        let curve_base = (self.segments.len() / 8) as u32;
        dispatch!(self.level, simd => {
            fill_impl(simd, path.iter(), transform, &mut self.segments, &mut self.line_buf, &mut self.bbox);
        });

        if self.line_buf.is_empty() {
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

        // ── Phase 4: Reset per-shape state ──
        self.blocks.reset();
        {
            let mut covers = self.covers.borrow_mut();
            covers.reset_for_shape(&self.bbox);
        }

        // ── Phase 5: DDA binning ──
        // Now we need to map each flattened line → which curve it came from.
        // Since fill_impl pushes 8 floats per curve to segments, and multiple
        // flattened lines can come from one curve, we need the mapping.
        //
        // Solution: fill_impl also outputs a parallel array mapping
        // line_index → curve_index. But that's a bigger refactor.
        //
        // SIMPLER APPROACH: For the DDA, use the curve_index directly.
        // Each flattened line belongs to exactly one curve. Since curves
        // produce 1+ flattened lines, we need fill_impl to tell us.
        //
        // SIMPLEST FIX: Store curve_id per flattened line alongside line_buf.

        let line_count = self.line_buf.len() / 5; // NOW 5 i32s per line: [p0x, p0y, p1x, p1y, curve_id]
        let mut covers = self.covers.borrow_mut();

        for i in 0..line_count {
            let base = i * 5;
            let p0x = self.line_buf[base];
            let p0y = self.line_buf[base + 1];
            let p1x = self.line_buf[base + 2];
            let p1y = self.line_buf[base + 3];
            let curve_id = self.line_buf[base + 4] as u32;

            self.blocks.build_block(
                &mut covers,
                &tile_bounds,
                curve_id, // ← THIS is the curve index into segments texture
                p0x,
                p0y,
                p1x,
                p1y,
            );
        }
    }

    pub fn generate_tiles(&mut self, paint_index: u32, fill_rule: Fill,   payload: u32,
        paint_flag: u32) {
             let fill_rule_word = match fill_rule {
            Fill::NonZero => FILL_RULE_NONZERO,
            Fill::EvenOdd => FILL_RULE_EVENODD,
        };
             let final_paint_flag = paint_flag | (fill_rule_word << 24);
        let covers = self.covers.borrow();
        let tile_bounds = TileBounds::from_box2d(&self.bbox);

        if tile_bounds.col_count == 0 || tile_bounds.row_count == 0 {
            return;
        }

        // Sort blocks
        self.blocks.sort_blocks();

        let fill_rule_flag = match fill_rule {
            Fill::NonZero => FILL_RULE_NONZERO,
            Fill::EvenOdd => FILL_RULE_EVENODD,
        };
        let packed_paint = (COLOR_SOURCE_PAINT << COLOR_SOURCE_SHIFT)
            | (PAINT_TYPE_SOLID << PAINT_TYPE_SHIFT)
            | (fill_rule_flag << FILL_RULE_SHIFT)
            | paint_index;

        // Propagate covers + emit tiles
        for row in 0..covers.rows() {
            let mut acc = [0i16; TILE_H];

            for col in 0..covers.cols() {
                let tagged = covers.is_tagged(row, col);

                if tagged || acc != [0i16; TILE_H] {
                    let global_x = (tile_bounds.min_col + col as i32) as u16;
                    let global_y = (tile_bounds.min_row + row as i32) as u16;

                    // Find segment range and build segment_list
                    let list_offset = self.segment_list.len() as u32;
                    let (block_start, block_count) =
                        find_segment_range(&self.blocks.data, global_x, global_y);
                    for k in block_start..(block_start + block_count) {
                        self.segment_list.push(self.blocks.data[k].segment_id);
                    }

                    let backdrop = [
                        (acc[0] as i32) | ((acc[1] as i32) << 16),
                        (acc[2] as i32) | ((acc[3] as i32) << 16),
                    ];

                    self.tiles.push(Tile {
                        x: global_x,
                        y: global_y,
                        width: TILE_W as u8,
                        height: TILE_H as u8,
                        _pad: [0, 0],
                        backdrop,
                        segments: [
                            f32::from_bits(list_offset),
                            f32::from_bits(block_count as u32),
                        ],
                        payload: payload,
                        paint_and_rect_flag: final_paint_flag,
                        depth_index: 0,
                    });
                }

                if tagged {
                    let crossings = covers.get_crossings(row, col);
                    for s in 0..TILE_H {
                        acc[s] += crossings[s] as i16;
                    }
                }
            }
        }
    }

    pub fn reset(&mut self) {
        self.tiles.clear();
        self.segments.clear();
        self.segment_list.clear();
        self.line_buf.clear();
        self.blocks.reset();
    }
}

/// Binary search for blocks at tile position (x, y) in sorted blocks array.
fn find_segment_range(blocks: &[Block], x: u16, y: u16) -> (usize, usize) {
    let start = blocks.partition_point(|b| (b.y, b.x) < (y, x));
    let end = blocks.partition_point(|b| (b.y, b.x) <= (y, x));
    (start, end - start)
}

pub const TILE_W: usize = 4;
pub const TILE_H: usize = 4;

pub struct CoverStorage {
    /// Bit vector: 1 bit per cell. Packed into u32 words.
    pub tag: Vec<u32>,
    /// Dense crossings: one [i8; TILE_H] per cell in the path's bbox.
    pub backdrops: Vec<[i8; TILE_H]>,
    /// Cached column count for current shape (avoids recomputing from bounds).
    col_count: usize,
    /// Cached row count for current shape.
    row_count: usize,
}

impl CoverStorage {
    /// Create an empty CoverStorage. Call `reset_for_shape` before use.
    pub fn new() -> Self {
        Self {
            tag: Vec::new(),
            backdrops: Vec::new(),
            col_count: 0,
            row_count: 0,
        }
    }

    #[inline(always)]
    pub fn tile_cols_from_bounds(bounds: &Box2D<f32>) -> usize {
        let width = (bounds.max.x - bounds.min.x).ceil() as usize;
        width.div_ceil(TILE_W)
    }

    #[inline(always)]
    pub fn tile_rows_from_bounds(bounds: &Box2D<f32>) -> usize {
        let height = (bounds.max.y - bounds.min.y).ceil() as usize;
        height.div_ceil(TILE_H)
    }

    #[inline(always)]
    pub fn cols(&self) -> usize {
        self.col_count
    }

    #[inline(always)]
    pub fn rows(&self) -> usize {
        self.row_count
    }

    /// Reset and resize for a new shape. Reuses allocation if large enough.
    pub fn reset_for_shape(&mut self, bounds: &Box2D<f32>) {
        self.col_count = Self::tile_cols_from_bounds(bounds);
        self.row_count = Self::tile_rows_from_bounds(bounds);
        let total_cells = self.col_count * self.row_count;
        let tag_words = (total_cells + 31) / 32;

        self.tag.clear();
        self.tag.resize(tag_words, 0);

        self.backdrops.clear();
        self.backdrops.resize(total_cells, [0i8; TILE_H]);
    }

    /// Set the tag bit for cell (row, col). Called during binning.
    #[inline(always)]
    pub fn set_tag(&mut self, row: usize, col: usize) {
        debug_assert!(row < self.row_count);
        debug_assert!(col < self.col_count);
        let bit_index = row * self.col_count + col;
        let word = bit_index / 32;
        let bit = bit_index % 32;
        self.tag[word] |= 1u32 << bit;
    }

    /// Check if cell (row, col) has been tagged.
    #[inline(always)]
    pub fn is_tagged(&self, row: usize, col: usize) -> bool {
        debug_assert!(row < self.row_count);
        debug_assert!(col < self.col_count);
        let bit_index = row * self.col_count + col;
        let word = bit_index / 32;
        let bit = bit_index % 32;
        (self.tag[word] >> bit) & 1 != 0
    }

    /// Conditionally set tag. Returns true if this is the first time (was unset).
    #[inline(always)]
    pub fn conditional_set_tag(&mut self, row: usize, col: usize) -> bool {
        debug_assert!(row < self.row_count);
        debug_assert!(col < self.col_count);
        let bit_index = row * self.col_count + col;
        let word = bit_index / 32;
        let bit = bit_index % 32;
        let mask = 1u32 << bit;
        let was_zero = (self.tag[word] & mask) == 0;
        self.tag[word] |= mask;
        was_zero
    }

    /// Get mutable reference to crossings for cell (row, col).
    #[inline(always)]
    pub fn crossings_at(&mut self, row: usize, col: usize) -> &mut [i8; TILE_H] {
        debug_assert!(row < self.row_count);
        debug_assert!(col < self.col_count);
        &mut self.backdrops[row * self.col_count + col]
    }

    /// Read crossings for cell (row, col).
    #[inline(always)]
    pub fn get_crossings(&self, row: usize, col: usize) -> [i8; TILE_H] {
        debug_assert!(row < self.row_count);
        debug_assert!(col < self.col_count);
        self.backdrops[row * self.col_count + col]
    }

    /// Propagate crossings into backdrops via left-to-right prefix scan.
    /// `bounds` is the same Box2D used in `reset_for_shape`.
    pub fn propagate(&self, bounds: &Box2D<f32>) -> Vec<(u16, u16, [i16; TILE_H])> {
        let cols = self.col_count;
        let rows = self.row_count;
        let mut result = Vec::new();

        let origin_col = (bounds.min.x / TILE_W as f32).floor() as u16;
        let origin_row = (bounds.min.y / TILE_H as f32).floor() as u16;

        for row in 0..rows {
            let mut acc = [0i16; TILE_H];
            let row_start = row * cols;

            for col in 0..cols {
                let cell_index = row_start + col;
                let word = cell_index / 32;
                let bit = cell_index % 32;
                let tagged = (self.tag[word] >> bit) & 1 != 0;

                if tagged {
                    let global_row = origin_row + row as u16;
                    let global_col = origin_col + col as u16;
                    result.push((global_row, global_col, acc));

                    let crossings = &self.backdrops[cell_index];
                    for s in 0..TILE_H {
                        acc[s] += crossings[s] as i16;
                    }
                } else if acc != [0i16; TILE_H] {
                    let global_row = origin_row + row as u16;
                    let global_col = origin_col + col as u16;
                    result.push((global_row, global_col, acc));
                }
            }
        }

        result
    }
}
