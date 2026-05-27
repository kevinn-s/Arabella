use alloc::{format, vec};
use alloc::vec::Vec;
use core::cell::RefCell;
use fearless_simd::*;
use peniko::Fill;

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
    /// Per-tile clipped lines, written in tile order during `generate_tiles`.
    /// Each line = 4 floats in PIXEL units (p0.x, p0.y, p1.x, p1.y), all
    /// expressed in tile-LOCAL coordinates so the GPU just compares with
    /// `pixel_in_tile`. One RGBA32F texel per line.
    pub segments: Vec<f32>,
    /// Flattened line endpoints in F24Dot8 (cleared per shape).
    /// Layout: 4 i32s per line = [p0x, p0y, p1x, p1y].
    pub(crate) line_buf: Vec<i32>,
    /// Sparse tile records from DDA binning (cleared per shape).
    pub(crate) blocks: Blocks,
    /// Per-shape cover accumulation.
    pub(crate) covers: RefCell<CoverStorage>,
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
                width: 16,
                height: 8,
                _pad: [0, 0],
                backdrop: [0, 0, 0, 0, 0, 0, 0, 0],
                segments: [0.0, 0.0],
                payload: 0,
                paint_and_rect_flag: 0,
                depth_index: 0,
            }),
            segments: Vec::new(),
            line_buf: Vec::new(),
            blocks: Blocks {
                data: Vec::with_capacity(16384),
                sorted: false,
            },
            covers: RefCell::new(CoverStorage::new()),
            bbox: Box2D::new(Point2D::new(0.0, 0.0), Point2D::new(0.0, 0.0)),
            level,
            shape_index: 0,
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

        // ── Phase 2: Process path → line_buf + bbox ──
        self.line_buf.clear();
        dispatch!(self.level, simd => {
            fill_impl(simd, path.iter(), transform, &mut self.line_buf, &mut self.bbox);
        });

        if self.line_buf.is_empty() {
            return;
        }

        // Use the natural per-shape bbox produced by `expand_bbox` in path.rs.
        // Do NOT clamp it to the canvas — the DDA needs to cover the entire
        // geometry (including parts above/left of the canvas) so that winding
        // cancellation works for closed paths whose edges extend off-screen.
        // Off-canvas tiles produce no visible fragments anyway because the
        // vertex shader places them outside the NDC range.

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
        // Each entry in line_buf is one flattened straight line:
        //   [p0x, p0y, p1x, p1y]  (4 i32s, F24Dot8)
        // The DDA clips each line to tile boundaries and emits one Block per
        // (line, tile) pair, with the *clipped* tile-local endpoints stored
        // directly in the Block. No global "line id" indirection is needed.

        let line_count = self.line_buf.len() / 4;
        let mut covers = self.covers.borrow_mut();

        for i in 0..line_count {
            let base = i * 4;
            let p0x = self.line_buf[base];
            let p0y = self.line_buf[base + 1];
            let p1x = self.line_buf[base + 2];
            let p1y = self.line_buf[base + 3];

            self.blocks.build_block(&mut covers, &tile_bounds, p0x, p0y, p1x, p1y);
        }
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
        let final_paint_flag = paint_flag | (fill_rule_word << 24);
        let covers = self.covers.borrow();
        let tile_bounds = TileBounds::from_box2d(&self.bbox);

        if tile_bounds.col_count == 0 || tile_bounds.row_count == 0 {
            return;
        }

        // Sort blocks by (y, x) so all blocks for one tile sit in a contiguous range.
        self.blocks.sort_blocks();

        let current_depth = self.shape_index;
        self.shape_index += 1;

        // Propagate per-row backdrops + emit tiles.
        //
        // Hot loop: runs once per (row, col) cell across the shape's bbox.
        // Two micro-optimizations applied:
        //
        // 1. `acc != [0i16; 8]` is checked via a single 16-byte compare
        //    using bytemuck::cast<[i16; 8], u128>. The compiler-generated
        //    elementwise compare was a measurable hotspot.
        //
        // 2. `acc[s] += crossings[s]` becomes one `i16x8` SIMD add via
        //    fearless_simd. WASM SIMD128 maps this to a single `i16x8.add`
        //    instruction, vs. 8 scalar adds. On every tagged tile.
        //
        // The acc value passed to the Tile struct is stored back into a
        // [i16; 8] array, so downstream code (GPU upload via bytemuck) is
        // unchanged.
        let zero_acc_bits: u128 = 0;
        for row in 0..covers.rows() {
            let mut acc_arr = [0i16; TILE_H];

            for col in 0..covers.cols() {
                let tagged = covers.is_tagged(row, col);

                // Fast-path: 16-byte compare vs zero rather than 8x i16 compare.
                let acc_nonzero =
                    bytemuck::cast::<[i16; 8], u128>(acc_arr) != zero_acc_bits;

                if tagged || acc_nonzero {
                    let global_x = (tile_bounds.min_col + col as i32) as u16;
                    let global_y = (tile_bounds.min_row + row as i32) as u16;

                    // Locate this tile's clipped lines in the sorted blocks vec
                    // and write them as RGBA32F texels into `segments`.
                    let (block_start, block_count) =
                        find_segment_range(&self.blocks.data, global_x, global_y);

                    let line_offset = (self.segments.len() / 4) as u32;
                    for k in block_start..(block_start + block_count) {
                        let b = &self.blocks.data[k];
                        // Convert F24Dot8 → float pixels (tile-local).
                        self.segments.extend_from_slice(&[
                            b.p0x as f32 / 256.0, b.p0y as f32 / 256.0,
                            b.p1x as f32 / 256.0, b.p1y as f32 / 256.0,
                        ]);
                    }
                    let line_count = block_count as u32;

                    self.tiles.push(Tile {
                        x: global_x,
                        y: global_y,
                        width: TILE_W as u8,
                        height: TILE_H as u8,
                        _pad: [0, 0],
                        backdrop: acc_arr,
                        segments: [
                            f32::from_bits(line_offset),
                            f32::from_bits(line_count),
                        ],
                        payload,
                        paint_and_rect_flag: final_paint_flag,
                        depth_index: current_depth,
                    });
                }

                if tagged {
                    let crossings = covers.get_crossings(row, col);
                    // Single i16x8 SIMD add: replaces 8 scalar i16 adds.
                    dispatch!(self.level, simd => {
                        let acc_v = i16x8::from_slice(simd, &acc_arr);
                        let cr_v = i16x8::from_slice(simd, &crossings);
                        let sum = acc_v + cr_v;
                        acc_arr = sum.into();
                    });
                }

                // ── Per-tile dump for a y-pixel range of interest ──
                // Set DEBUG_Y_RANGE to (None, None) to disable, or to
                // (Some(min), Some(max)) to print every tile whose pixel
                // span overlaps [min, max]. This shows exactly what the
                // shader will see for the suspect scanlines.
                const DEBUG_Y_RANGE: (Option<i32>, Option<i32>) = (None, None);
                if let (Some(y_lo), Some(y_hi)) = DEBUG_Y_RANGE {
                    let global_y = tile_bounds.min_row + row as i32;
                    let tile_y_lo = global_y * TILE_H as i32;
                    let tile_y_hi = tile_y_lo + TILE_H as i32 - 1;
                    if tile_y_hi >= y_lo && tile_y_lo <= y_hi
                        && (tagged || acc_nonzero)
                    {
                        let global_x = tile_bounds.min_col + col as i32;
                        let (block_start, block_count) = find_segment_range(
                            &self.blocks.data,
                            global_x as u16,
                            global_y as u16,
                        );
                        web_sys::console::log_1(
                            &alloc::format!(
                                "[TILE-DUMP] tile=({},{}) global_pixel_y=[{}..{}] backdrop={:?} lines={}",
                                global_x, global_y, tile_y_lo, tile_y_hi, acc_arr, block_count,
                            )
                            .into(),
                        );
                        for k in block_start..(block_start + block_count) {
                            let b = &self.blocks.data[k];
                            web_sys::console::log_1(
                                &alloc::format!(
                                    "    line[{}] tile-local F24Dot8: ({},{}) -> ({},{})  (px: ({:.3},{:.3})->({:.3},{:.3}))",
                                    k - block_start,
                                    b.p0x, b.p0y, b.p1x, b.p1y,
                                    b.p0x as f32 / 256.0, b.p0y as f32 / 256.0,
                                    b.p1x as f32 / 256.0, b.p1y as f32 / 256.0,
                                )
                                .into(),
                            );
                        }
                    }
                }
            }

            // ── Row-balance sanity check ──
            // After processing every column in this tile row, the running
            // accumulator MUST be zero on every scanline. A non-zero value
            // means at least one emitted line on that scanline didn't have
            // a matching opposite-direction partner — its winding will
            // streak rightward off the canvas.
            //
            // We log the global tile-row index, the residual values, and
            // which scanlines (within the tile row) leaked. The y-pixel
            // for scanline `s` in tile row `row` is:
            //   y_pixel = (tile_bounds.min_row + row) * TILE_H + s
            // ── Row-balance sanity check (debug builds only) ──
            // After processing every column in this tile row, the running
            // accumulator MUST be zero on every scanline. A non-zero value
            // means at least one emitted line on that scanline didn't have
            // a matching opposite-direction partner — its winding will
            // streak rightward off the canvas.
            //
            // Gated to debug builds because a u128 compare per row is
            // cheap, but the format/log path on a leak is not, and we
            // never want this in the per-frame interactive demo.
            #[cfg(debug_assertions)]
            if bytemuck::cast::<[i16; 8], u128>(acc_arr) != zero_acc_bits {
                let global_row = tile_bounds.min_row + row as i32;
                let mut leaked: alloc::string::String = alloc::string::String::new();
                for s in 0..TILE_H {
                    if acc_arr[s] != 0 {
                        let y_pixel = global_row * TILE_H as i32 + s as i32;
                        leaked.push_str(&alloc::format!(
                            " [row_in_tile={} y_pixel={} residual={}]",
                            s, y_pixel, acc_arr[s],
                        ));
                    }
                }
                web_sys::console::log_1(
                    &alloc::format!(
                        "[ROW-BALANCE LEAK] tile_row={} (global_row={}) acc={:?} leaked:{}",
                        row, global_row, acc_arr, leaked,
                    )
                    .into(),
                );
            }
        }
    }

    pub fn reset(&mut self) {
        self.tiles.clear();
        self.segments.clear();
        self.blocks.reset();
        // Reset depth counter so Scene::reset() (used per frame for the
        // interactive demo) doesn't grow depth_index unboundedly across frames.
        self.shape_index = 0;
    }
}

/// Binary search for blocks at tile position (x, y) in sorted blocks array.
fn find_segment_range(blocks: &[Block], x: u16, y: u16) -> (usize, usize) {
    let start = blocks.partition_point(|b| (b.y, b.x) < (y, x));
    let end = blocks.partition_point(|b| (b.y, b.x) <= (y, x));
    (start, end - start)
}

pub const TILE_W: usize = 16;
pub const TILE_H: usize = 8;

#[derive(Clone)]
pub struct CoverStorage {
    /// Bit vector: 1 bit per cell. Packed into u32 words.
    pub tag: Vec<u32>,
    /// Dense crossings: one [i8; TILE_H] per cell in the path's bbox.
    pub backdrops: Vec<[i16; TILE_H]>,
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

    /// Tile column count for a shape, computed by the SAME rule used by
    /// `TileBounds::from_box2d`: `ceil(max.x / TILE_W) - floor(min.x / TILE_W)`.
    /// (Computing it as `ceil(max-min)/TILE_W` is off-by-one whenever the
    /// bbox doesn't start exactly on a tile boundary, which silently drops
    /// edge-row writes in the DDA.)
    #[inline(always)]
    pub fn tile_cols_from_bounds(bounds: &Box2D<f32>) -> usize {
        let max_col = (bounds.max.x / TILE_W as f32).ceil() as i32;
        let min_col = (bounds.min.x / TILE_W as f32).floor() as i32;
        (max_col - min_col).max(0) as usize
    }

    /// Tile row count for a shape — same rule as `tile_cols_from_bounds`.
    #[inline(always)]
    pub fn tile_rows_from_bounds(bounds: &Box2D<f32>) -> usize {
        let max_row = (bounds.max.y / TILE_H as f32).ceil() as i32;
        let min_row = (bounds.min.y / TILE_H as f32).floor() as i32;
        (max_row - min_row).max(0) as usize
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
        self.backdrops.resize(total_cells, [0i16; TILE_H]);
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
    pub fn crossings_at(&mut self, row: usize, col: usize) -> &mut [i16; TILE_H] {
        debug_assert!(row < self.row_count);
        debug_assert!(col < self.col_count);
        &mut self.backdrops[row * self.col_count + col]
    }

    /// Read crossings for cell (row, col).
    #[inline(always)]
    pub fn get_crossings(&self, row: usize, col: usize) -> [i16; TILE_H] {
        debug_assert!(row < self.row_count);
        debug_assert!(col < self.col_count);
        self.backdrops[row * self.col_count + col]
    }

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
