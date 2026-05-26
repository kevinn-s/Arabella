use alloc::{format, vec::Vec};
use lyon_geom::euclid::default::Box2D;

use crate::builder::CoverStorage;

pub const TILE_W: usize = 16;
pub const TILE_H: usize = 8;
pub const TILE_W_LOG2: u32 = 4;   // log2(16) = 4
pub const TILE_H_LOG2: u32 = 3;   // log2(8) = 3
pub const TILE_W_F24DOT8: i32 = (TILE_W as i32) << 8;  // 4096
pub const TILE_H_F24DOT8: i32 = (TILE_H as i32) << 8;  // 2048


pub const MAXIMUM_DELTA: i32 = 2048 << 8;

// ============================================================================
// Block: per (segment, tile) record for GPU upload
// ============================================================================

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Block {
    /// Clipped line endpoints in tile-LOCAL F24Dot8 coordinates.
    /// Range: x ∈ [0, TILE_W * 256], y ∈ [0, TILE_H * 256].
    pub p0x: i32,
    pub p0y: i32,
    pub p1x: i32,
    pub p1y: i32,
    #[cfg(target_endian = "little")]
    pub x: u16,
    #[cfg(target_endian = "little")]
    pub y: u16,
    #[cfg(target_endian = "big")]
    pub y: u16,
    #[cfg(target_endian = "big")]
    pub x: u16,
}

impl Block {
    #[inline]
    pub const fn new(x: u16, y: u16, p0x: i32, p0y: i32, p1x: i32, p1y: i32) -> Self {
        Self { p0x, p0y, p1x, p1y, x, y }
    }
}

// ============================================================================
// Blocks: container that accumulates Block records + drives the binning
// ============================================================================

#[derive(Clone, Debug)]
pub struct Blocks {
    pub data: Vec<Block>,
    pub sorted: bool,
}

impl Blocks {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            sorted: false,
        }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn reset(&mut self) {
        self.data.clear();
        self.sorted = false;
    }

    pub fn sort_blocks(&mut self) {
        self.sorted = true;
        self.data.sort_unstable_by_key(|b| {
            ((b.y as u64) << 16) | (b.x as u64)
        });
    }

    pub fn iter(&self) -> impl Iterator<Item = &Block> {
        debug_assert!(self.sorted, "call sort_blocks() before iterating");
        self.data.iter()
    }

    // ========================================================================
    // Main entry point: bin a single straight line segment.
    // ========================================================================

    pub fn build_block(
        &mut self,
        covers: &mut CoverStorage,
        bounds: &TileBounds,
        p0x: i32, p0y: i32,
        p1x: i32, p1y: i32,
    ) {
        self.bin_line(covers, bounds, p0x, p0y, p1x, p1y);
    }

    // ========================================================================
    // Stage 3: Outer DDA — split line across tile rows
    // ========================================================================

    pub fn bin_line(
        &mut self,
        covers: &mut CoverStorage,
        bounds: &TileBounds,
        p0x: i32, p0y: i32,
        p1x: i32, p1y: i32,
    ) {
        if p0y == p1y {
            return;
        }

        let dx = (p1x - p0x).abs();
        let dy = (p1y - p0y).abs();
        if dx > MAXIMUM_DELTA || dy > MAXIMUM_DELTA {
            let mx = (p0x + p1x) >> 1;
            let my = (p0y + p1y) >> 1;
            self.bin_line(covers, bounds, p0x, p0y, mx, my);
            self.bin_line(covers, bounds, mx, my, p1x, p1y);
            return;
        }

        if p0y < p1y {
            // Line going DOWN
            let row0 = ((p0y >> (8 + TILE_H_LOG2)) - bounds.min_row) as usize;
            let row1 = (((p1y - 1) >> (8 + TILE_H_LOG2)) - bounds.min_row) as usize;

            if row0 == row1 {
                let ty = ((row0 as i32) + bounds.min_row) << (8 + TILE_H_LOG2);
                let local_y0 = p0y - ty;
                let local_y1 = p1y - ty;
                self.bin_line_in_row(covers, bounds, row0, p0x, local_y0, p1x, local_y1);
            } else if p0x <= p1x {
                self.outer_dda_down_right(covers, bounds, row0, row1, p0x, p0y, p1x, p1y);
            } else {
                self.outer_dda_down_left(covers, bounds, row0, row1, p0x, p0y, p1x, p1y);
            }
        } else {
            // Line going UP
            let row0 = (((p0y - 1) >> (8 + TILE_H_LOG2)) - bounds.min_row) as usize;
            let row1 = ((p1y >> (8 + TILE_H_LOG2)) - bounds.min_row) as usize;

            if row0 == row1 {
                let ty = ((row0 as i32) + bounds.min_row) << (8 + TILE_H_LOG2);
                let local_y0 = p0y - ty;
                let local_y1 = p1y - ty;
                self.bin_line_in_row(covers, bounds, row0, p0x, local_y0, p1x, local_y1);
            } else if p0x <= p1x {
                self.outer_dda_up_right(covers, bounds, row0, row1, p0x, p0y, p1x, p1y);
            } else {
                self.outer_dda_up_left(covers, bounds, row0, row1, p0x, p0y, p1x, p1y);
            }
        }
    }

    // ── Down-Right (↓→) ──

    fn outer_dda_down_right(
        &mut self,
        covers: &mut CoverStorage,
        bounds: &TileBounds,
        row0: usize, row1: usize,
        p0x: i32, p0y: i32,
        p1x: i32, p1y: i32,
    ) {
        let dx = p1x - p0x; // positive
        let dy = p1y - p0y; // positive

        let ty0 = ((row0 as i32) + bounds.min_row) << (8 + TILE_H_LOG2);
        let fy0 = p0y - ty0;
        let ty1 = ((row1 as i32) + bounds.min_row) << (8 + TILE_H_LOG2);
        let fy1 = p1y - ty1;

        let pp = (TILE_H_F24DOT8 - fy0) * dx;
        let mut cx = p0x + pp / dy;

        self.bin_line_in_row(covers, bounds, row0, p0x, fy0, cx, TILE_H_F24DOT8);

        let mut idy = row0 + 1;
        if idy != row1 {
            let mut mod_ = (pp % dy) - dy;
            let p = TILE_H_F24DOT8 * dx;
            let lift = p / dy;
            let rem = p % dy;

            while idy != row1 {
                let mut delta = lift;
                mod_ += rem;
                if mod_ >= 0 {
                    mod_ -= dy;
                    delta += 1;
                }
                let nx = cx + delta;
                self.bin_line_in_row(covers, bounds, idy, cx, 0, nx, TILE_H_F24DOT8);
                cx = nx;
                idy += 1;
            }
        }

        self.bin_line_in_row(covers, bounds, row1, cx, 0, p1x, fy1);
    }

    // ── Down-Left (↓←) ──

    fn outer_dda_down_left(
        &mut self,
        covers: &mut CoverStorage,
        bounds: &TileBounds,
        row0: usize, row1: usize,
        p0x: i32, p0y: i32,
        p1x: i32, p1y: i32,
    ) {
        let dx = p0x - p1x; // positive (going left)
        let dy = p1y - p0y; // positive (going down)

        let ty0 = ((row0 as i32) + bounds.min_row) << (8 + TILE_H_LOG2);
        let fy0 = p0y - ty0;
        let ty1 = ((row1 as i32) + bounds.min_row) << (8 + TILE_H_LOG2);
        let fy1 = p1y - ty1;

        let pp = (TILE_H_F24DOT8 - fy0) * dx;
        let mut cx = p0x - pp / dy;

        self.bin_line_in_row(covers, bounds, row0, p0x, fy0, cx, TILE_H_F24DOT8);

        let mut idy = row0 + 1;
        if idy != row1 {
            let mut mod_ = (pp % dy) - dy;
            let p = TILE_H_F24DOT8 * dx;
            let lift = p / dy;
            let rem = p % dy;

            while idy != row1 {
                let mut delta = lift;
                mod_ += rem;
                if mod_ >= 0 {
                    mod_ -= dy;
                    delta += 1;
                }
                let nx = cx - delta;
                self.bin_line_in_row(covers, bounds, idy, cx, 0, nx, TILE_H_F24DOT8);
                cx = nx;
                idy += 1;
            }
        }

        self.bin_line_in_row(covers, bounds, row1, cx, 0, p1x, fy1);
    }

    // ── Up-Right (↑→) ──

    fn outer_dda_up_right(
        &mut self,
        covers: &mut CoverStorage,
        bounds: &TileBounds,
        row0: usize, row1: usize,
        p0x: i32, p0y: i32,
        p1x: i32, p1y: i32,
    ) {
        let dx = p1x - p0x; // positive (going right)
        let dy = p0y - p1y; // positive (going up, stored as abs)

        let ty0 = ((row0 as i32) + bounds.min_row) << (8 + TILE_H_LOG2);
        let fy0 = p0y - ty0;
        let ty1 = ((row1 as i32) + bounds.min_row) << (8 + TILE_H_LOG2);
        let fy1 = p1y - ty1;

        let pp = fy0 * dx;
        let mut cx = p0x + pp / dy;

        self.bin_line_in_row(covers, bounds, row0, p0x, fy0, cx, 0);

        let mut idy = row0 - 1;
        if idy != row1 {
            let mut mod_ = (pp % dy) - dy;
            let p = TILE_H_F24DOT8 * dx;
            let lift = p / dy;
            let rem = p % dy;

            while idy != row1 {
                let mut delta = lift;
                mod_ += rem;
                if mod_ >= 0 {
                    mod_ -= dy;
                    delta += 1;
                }
                let nx = cx + delta;
                self.bin_line_in_row(covers, bounds, idy, cx, TILE_H_F24DOT8, nx, 0);
                cx = nx;
                idy -= 1;
            }
        }

        self.bin_line_in_row(covers, bounds, row1, cx, TILE_H_F24DOT8, p1x, fy1);
    }

    // ── Up-Left (↑←) ──

    fn outer_dda_up_left(
        &mut self,
        covers: &mut CoverStorage,
        bounds: &TileBounds,
        row0: usize, row1: usize,
        p0x: i32, p0y: i32,
        p1x: i32, p1y: i32,
    ) {
        let dx = p0x - p1x; // positive (going left, stored as abs)
        let dy = p0y - p1y; // positive (going up, stored as abs)

        let ty0 = ((row0 as i32) + bounds.min_row) << (8 + TILE_H_LOG2);
        let fy0 = p0y - ty0;
        let ty1 = ((row1 as i32) + bounds.min_row) << (8 + TILE_H_LOG2);
        let fy1 = p1y - ty1;

        let pp = fy0 * dx;
        let mut cx = p0x - pp / dy;

        self.bin_line_in_row(covers, bounds, row0, p0x, fy0, cx, 0);

        let mut idy = row0 - 1;
        if idy != row1 {
            let mut mod_ = (pp % dy) - dy;
            let p = TILE_H_F24DOT8 * dx;
            let lift = p / dy;
            let rem = p % dy;

            while idy != row1 {
                let mut delta = lift;
                mod_ += rem;
                if mod_ >= 0 {
                    mod_ -= dy;
                    delta += 1;
                }
                let nx = cx - delta;
                self.bin_line_in_row(covers, bounds, idy, cx, TILE_H_F24DOT8, nx, 0);
                cx = nx;
                idy -= 1;
            }
        }

        self.bin_line_in_row(covers, bounds, row1, cx, TILE_H_F24DOT8, p1x, fy1);
    }

    // ========================================================================
    // Stage 4: Inner DDA — split line across tile columns within one tile row
    // ========================================================================

    fn bin_line_in_row(
        &mut self,
        covers: &mut CoverStorage,
        bounds: &TileBounds,
        tile_row: usize,
        p0x: i32, p0y: i32,
        p1x: i32, p1y: i32,
    ) {
        if p0y == p1y {
            return;
        }

        if p0x == p1x {
            let col_raw = f24dot8_to_tile_col(p0x) - bounds.min_col;
            if col_raw < 0 || col_raw as usize >= covers.cols() {
                return;
            }
            let col = col_raw as usize;
            self.push_to_tile(covers, bounds, tile_row, col, p0x, p0y, p1x, p1y);
            return;
        }

        if p0x < p1x {
            // Going right
            if p0y < p1y {
                self.inner_dda_right_down(covers, bounds, tile_row, p0x, p0y, p1x, p1y);
            } else {
                self.inner_dda_right_up(covers, bounds, tile_row, p0x, p0y, p1x, p1y);
            }
        } else {
            // Going left
            if p0y < p1y {
                self.inner_dda_left_down(covers, bounds, tile_row, p0x, p0y, p1x, p1y);
            } else {
                self.inner_dda_left_up(covers, bounds, tile_row, p0x, p0y, p1x, p1y);
            }
        }
    }

    // ── Right-Down (→↓) ──

    fn inner_dda_right_down(
        &mut self,
        covers: &mut CoverStorage,
        bounds: &TileBounds,
        tile_row: usize,
        p0x: i32, p0y: i32,
        p1x: i32, p1y: i32,
    ) {
        let col0_raw = f24dot8_to_tile_col(p0x) - bounds.min_col;
        let col1_raw = f24dot8_to_tile_col(p1x - 1) - bounds.min_col;
        if col0_raw < 0 || col1_raw < 0 {
            return;
        }
        let col0 = col0_raw as usize;
        let col1 = col1_raw as usize;

        if col0 == col1 {
            self.push_to_tile(covers, bounds, tile_row, col0, p0x, p0y, p1x, p1y);
            return;
        }

        let dx = p1x - p0x; // positive
        let dy = p1y - p0y; // positive

        let fx = p0x - tile_col_to_f24dot8(f24dot8_to_tile_col(p0x));
        let pp = (TILE_W_F24DOT8 - fx) * dy;
        let mut cy = p0y + pp / dx;

        // x at right edge of col0 in global coords:
        let mut cx = tile_col_to_f24dot8(f24dot8_to_tile_col(p0x) + 1);
        self.push_to_tile(covers, bounds, tile_row, col0, p0x, p0y, cx, cy);

        let mut idx = col0 + 1;
        if idx != col1 {
            let mut mod_ = (pp % dx) - dx;
            let p = TILE_W_F24DOT8 * dy;
            let lift = p / dx;
            let rem = p % dx;

            while idx != col1 {
                let mut delta = lift;
                mod_ += rem;
                if mod_ >= 0 {
                    mod_ -= dx;
                    delta += 1;
                }
                let ny = cy + delta;
                let nx = cx + TILE_W_F24DOT8;
                self.push_to_tile(covers, bounds, tile_row, idx, cx, cy, nx, ny);
                cx = nx;
                cy = ny;
                idx += 1;
            }
        }

        self.push_to_tile(covers, bounds, tile_row, col1, cx, cy, p1x, p1y);
    }

    // ── Right-Up (→↑) ──

    fn inner_dda_right_up(
        &mut self,
        covers: &mut CoverStorage,
        bounds: &TileBounds,
        tile_row: usize,
        p0x: i32, p0y: i32,
        p1x: i32, p1y: i32,
    ) {
        let col0 = (f24dot8_to_tile_col(p0x) - bounds.min_col) as usize;
        let col1 = (f24dot8_to_tile_col(p1x - 1) - bounds.min_col) as usize;

        if col0 == col1 {
            self.push_to_tile(covers, bounds, tile_row, col0, p0x, p0y, p1x, p1y);
            return;
        }

        let dx = p1x - p0x; // positive
        let dy = p0y - p1y; // positive (going up, abs)

        let fx = p0x - tile_col_to_f24dot8(f24dot8_to_tile_col(p0x));
        let pp = (TILE_W_F24DOT8 - fx) * dy;
        let mut cy = p0y - pp / dx;

        let mut cx = tile_col_to_f24dot8(f24dot8_to_tile_col(p0x) + 1);
        self.push_to_tile(covers, bounds, tile_row, col0, p0x, p0y, cx, cy);

        let mut idx = col0 + 1;
        if idx != col1 {
            let mut mod_ = (pp % dx) - dx;
            let p = TILE_W_F24DOT8 * dy;
            let lift = p / dx;
            let rem = p % dx;

            while idx != col1 {
                let mut delta = lift;
                mod_ += rem;
                if mod_ >= 0 {
                    mod_ -= dx;
                    delta += 1;
                }
                let ny = cy - delta;
                let nx = cx + TILE_W_F24DOT8;
                self.push_to_tile(covers, bounds, tile_row, idx, cx, cy, nx, ny);
                cx = nx;
                cy = ny;
                idx += 1;
            }
        }

        self.push_to_tile(covers, bounds, tile_row, col1, cx, cy, p1x, p1y);
    }

    // ── Left-Down (←↓) ──

    fn inner_dda_left_down(
        &mut self,
        covers: &mut CoverStorage,
        bounds: &TileBounds,
        tile_row: usize,
        p0x: i32, p0y: i32,
        p1x: i32, p1y: i32,
    ) {
        let col0 = (f24dot8_to_tile_col(p0x - 1) - bounds.min_col) as usize;
        let col1 = (f24dot8_to_tile_col(p1x) - bounds.min_col) as usize;

        if col0 == col1 {
            self.push_to_tile(covers, bounds, tile_row, col0, p0x, p0y, p1x, p1y);
            return;
        }

        let dx = p0x - p1x; // positive (going left, abs)
        let dy = p1y - p0y; // positive (going down)

        let fx = p0x - tile_col_to_f24dot8(f24dot8_to_tile_col(p0x - 1));
        let pp = fx * dy;
        let mut cy = p0y + pp / dx;

        // x at left edge of col0 in global coords:
        let mut cx = tile_col_to_f24dot8(f24dot8_to_tile_col(p0x - 1));
        self.push_to_tile(covers, bounds, tile_row, col0, p0x, p0y, cx, cy);

        let mut idx = col0 - 1;
        if idx != col1 {
            let mut mod_ = (pp % dx) - dx;
            let p = TILE_W_F24DOT8 * dy;
            let lift = p / dx;
            let rem = p % dx;

            while idx != col1 {
                let mut delta = lift;
                mod_ += rem;
                if mod_ >= 0 {
                    mod_ -= dx;
                    delta += 1;
                }
                let ny = cy + delta;
                let nx = cx - TILE_W_F24DOT8;
                self.push_to_tile(covers, bounds, tile_row, idx, cx, cy, nx, ny);
                cx = nx;
                cy = ny;
                idx -= 1;
            }
        }

        self.push_to_tile(covers, bounds, tile_row, col1, cx, cy, p1x, p1y);
    }

    // ── Left-Up (←↑) ──

    fn inner_dda_left_up(
        &mut self,
        covers: &mut CoverStorage,
        bounds: &TileBounds,
        tile_row: usize,
        p0x: i32, p0y: i32,
        p1x: i32, p1y: i32,
    ) {
        let col0 = (f24dot8_to_tile_col(p0x - 1) - bounds.min_col) as usize;
        let col1 = (f24dot8_to_tile_col(p1x) - bounds.min_col) as usize;

        if col0 == col1 {
            self.push_to_tile(covers, bounds, tile_row, col0, p0x, p0y, p1x, p1y);
            return;
        }

        let dx = p0x - p1x; // positive (going left, abs)
        let dy = p0y - p1y; // positive (going up, abs)

        let fx = p0x - tile_col_to_f24dot8(f24dot8_to_tile_col(p0x - 1));
        let pp = fx * dy;
        let mut cy = p0y - pp / dx;

        let mut cx = tile_col_to_f24dot8(f24dot8_to_tile_col(p0x - 1));
        self.push_to_tile(covers, bounds, tile_row, col0, p0x, p0y, cx, cy);

        let mut idx = col0 - 1;
        if idx != col1 {
            let mut mod_ = (pp % dx) - dx;
            let p = TILE_W_F24DOT8 * dy;
            let lift = p / dx;
            let rem = p % dx;

            while idx != col1 {
                let mut delta = lift;
                mod_ += rem;
                if mod_ >= 0 {
                    mod_ -= dx;
                    delta += 1;
                }
                let ny = cy - delta;
                let nx = cx - TILE_W_F24DOT8;
                self.push_to_tile(covers, bounds, tile_row, idx, cx, cy, nx, ny);
                cx = nx;
                cy = ny;
                idx -= 1;
            }
        }

        self.push_to_tile(covers, bounds, tile_row, col1, cx, cy, p1x, p1y);
    }

    // ========================================================================
    // Stage 5: Push — record crossings + clip to tile + emit Block
    // ========================================================================

    /// Inputs:
    ///   - `x0g, x1g`: global F24Dot8 x-coords (line endpoints clipped to tile column).
    ///   - `y0l, y1l`: tile-LOCAL F24Dot8 y-coords (already clipped to tile row by outer DDA).
    /// Stored in `Block`:
    ///   - tile-LOCAL F24Dot8 endpoints, both axes clamped to `[0, TILE_*_F24DOT8]`.
    #[inline]
    fn push_to_tile(
        &mut self,
        covers: &mut CoverStorage,
        bounds: &TileBounds,
        tile_row: usize,
        tile_col: usize,
        x0g: i32, y0l: i32,
        x1g: i32, y1l: i32,
    ) {
        if y0l == y1l {
            return;
        }
        if tile_row >= covers.rows() || tile_col >= covers.cols() {
            return;
        }

        // Crossings only depend on y, which is already tile-local.
        record_per_scanline_crossings(covers.crossings_at(tile_row, tile_col), y0l, y1l);
        covers.set_tag(tile_row, tile_col);

        // Convert global x → tile-local x (and clamp defensively).
        let global_col = bounds.min_col + tile_col as i32;
        let tile_origin_x = tile_col_to_f24dot8(global_col);
        let x0l = (x0g - tile_origin_x).clamp(0, TILE_W_F24DOT8);
        let x1l = (x1g - tile_origin_x).clamp(0, TILE_W_F24DOT8);
        let y0c = y0l.clamp(0, TILE_H_F24DOT8);
        let y1c = y1l.clamp(0, TILE_H_F24DOT8);

        let global_col_u = global_col as u16;
        let global_row = (bounds.min_row + tile_row as i32) as u16;
        self.data.push(Block::new(global_col_u, global_row, x0l, y0c, x1l, y1c));
    }
}

// ============================================================================
// TileBounds: pre-computed tile-space bounds for the current shape
// ============================================================================

pub struct TileBounds {
    pub min_col: i32,
    pub min_row: i32,
    pub col_count: usize,
    pub row_count: usize,
}

impl TileBounds {
    pub fn from_box2d(bounds: &Box2D<f32>) -> Self {
        let min_col = (bounds.min.x / TILE_W as f32).floor() as i32;
        let min_row = (bounds.min.y / TILE_H as f32).floor() as i32;
        let max_col = (bounds.max.x / TILE_W as f32).ceil() as i32;
        let max_row = (bounds.max.y / TILE_H as f32).ceil() as i32;
        Self {
            min_col,
            min_row,
            col_count: (max_col - min_col) as usize,
            row_count: (max_row - min_row) as usize,
        }
    }
}

// ============================================================================
// record_per_scanline_crossings — integer winding at pixel centers
//
// MUST match the GPU shader's `line_contribution` exactly.
//
// The shader does, for each pixel center `pixel.y = r + 0.5`:
//   if (y_min <= pixel.y < y_max) winding += sign
//
// We mirror that on the CPU side: for each scanline `r` (0..TILE_H), the
// pixel center in tile-local F24Dot8 is `r*256 + 128`. A line `(y0, y1)`
// contributes ±256 (= one full winding in 8.8 fixed-point) to scanline r
// iff `y_min <= r*256 + 128 < y_max`. The sign convention (down→-1, up→+1)
// also matches the shader.
//
// This is a pure point-sample winding rule. It is NOT area-based — area
// would put fractional values on partially-covered scanlines, which the
// shader cannot reproduce because it only sees the pixel center.
// ============================================================================

#[inline(always)]
fn record_per_scanline_crossings(crossings: &mut [i16; TILE_H], y0: i32, y1: i32) {
    if y0 == y1 { return; }

    let (y_min, y_max, sign): (i32, i32, i16) = if y0 < y1 {
        (y0, y1, -256)   // line goes DOWN → -1 winding (in 8.8)
    } else {
        (y1, y0, 256)    // line goes UP   → +1 winding (in 8.8)
    };

    // Iterate the scanlines whose pixel centers can possibly fall in [y_min, y_max).
    // Pixel center for scanline `r` is `r*256 + 128`.
    //
    // Smallest r with center >= y_min:   r >= (y_min - 128) / 256, ceil
    //                                  ⇒ r_lo = (y_min - 128 + 255) >> 8  for y_min > 128
    //                                    (use a generic clamp + check below)
    // Largest r with center < y_max:     r < (y_max - 128) / 256
    //                                  ⇒ r_hi = (y_max - 129) >> 8       (inclusive)
    //
    // To stay simple and correct at boundaries, we just walk every TILE_H scanline
    // and test. TILE_H is 8 — branch cost is negligible.
    for r in 0..TILE_H {
        let center = (r as i32) * 256 + 128;
        if y_min <= center && center < y_max {
            crossings[r] = crossings[r].saturating_add(sign);
        }
    }
}


// ============================================================================
// Coordinate helpers
// ============================================================================

#[inline(always)]
fn f24dot8_to_tile_col(x: i32) -> i32 {
    x >> (8 + TILE_W_LOG2)
}

#[inline(always)]
fn tile_col_to_f24dot8(col: i32) -> i32 {
    col << (8 + TILE_W_LOG2)
}