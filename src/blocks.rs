use alloc::vec::Vec;
use lyon_geom::euclid::default::Box2D;

use crate::builder::CoverStorage;

pub const TILE_W: usize = 4;
pub const TILE_H: usize = 4;
pub const TILE_W_LOG2: u32 = 2;
pub const TILE_H_LOG2: u32 = 2;
pub const TILE_W_F24DOT8: i32 = (TILE_W as i32) << 8;
pub const TILE_H_F24DOT8: i32 = (TILE_H as i32) << 8;
pub const MAXIMUM_DELTA: i32 = 2048 << 8;

// ============================================================================
// Block: per (segment, tile) record for GPU upload
// ============================================================================

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Block {
    pub segment_id: u32,
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
    pub const fn new(x: u16, y: u16, segment_id: u32) -> Self {
        Self { segment_id, x, y }
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
            ((b.y as u64) << 48) | ((b.x as u64) << 32) | b.segment_id as u64
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
        segment_id: u32,
        p0x: i32, p0y: i32,
        p1x: i32, p1y: i32,
    ) {
        self.bin_line(covers, bounds, segment_id, p0x, p0y, p1x, p1y);
    }

    // ========================================================================
    // Stage 3: Outer DDA — split line across tile rows
    // ========================================================================

    pub fn bin_line(
        &mut self,
        covers: &mut CoverStorage,
        bounds: &TileBounds,
        segment_id: u32,
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
            self.bin_line(covers, bounds, segment_id, p0x, p0y, mx, my);
            self.bin_line(covers, bounds, segment_id, mx, my, p1x, p1y);
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
                self.bin_line_in_row(covers, bounds, segment_id, row0, p0x, local_y0, p1x, local_y1);
            } else if p0x <= p1x {
                self.outer_dda_down_right(covers, bounds, segment_id, row0, row1, p0x, p0y, p1x, p1y);
            } else {
                self.outer_dda_down_left(covers, bounds, segment_id, row0, row1, p0x, p0y, p1x, p1y);
            }
        } else {
            // Line going UP
            let row0 = (((p0y - 1) >> (8 + TILE_H_LOG2)) - bounds.min_row) as usize;
            let row1 = ((p1y >> (8 + TILE_H_LOG2)) - bounds.min_row) as usize;

            if row0 == row1 {
                let ty = ((row0 as i32) + bounds.min_row) << (8 + TILE_H_LOG2);
                let local_y0 = p0y - ty;
                let local_y1 = p1y - ty;
                self.bin_line_in_row(covers, bounds, segment_id, row0, p0x, local_y0, p1x, local_y1);
            } else if p0x <= p1x {
                self.outer_dda_up_right(covers, bounds, segment_id, row0, row1, p0x, p0y, p1x, p1y);
            } else {
                self.outer_dda_up_left(covers, bounds, segment_id, row0, row1, p0x, p0y, p1x, p1y);
            }
        }
    }

    // ── Down-Right (↓→) ──

    fn outer_dda_down_right(
        &mut self,
        covers: &mut CoverStorage,
        bounds: &TileBounds,
        segment_id: u32,
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

        self.bin_line_in_row(covers, bounds, segment_id, row0, p0x, fy0, cx, TILE_H_F24DOT8);

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
                self.bin_line_in_row(covers, bounds, segment_id, idy, cx, 0, nx, TILE_H_F24DOT8);
                cx = nx;
                idy += 1;
            }
        }

        self.bin_line_in_row(covers, bounds, segment_id, row1, cx, 0, p1x, fy1);
    }

    // ── Down-Left (↓←) ──

    fn outer_dda_down_left(
        &mut self,
        covers: &mut CoverStorage,
        bounds: &TileBounds,
        segment_id: u32,
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

        self.bin_line_in_row(covers, bounds, segment_id, row0, p0x, fy0, cx, TILE_H_F24DOT8);

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
                self.bin_line_in_row(covers, bounds, segment_id, idy, cx, 0, nx, TILE_H_F24DOT8);
                cx = nx;
                idy += 1;
            }
        }

        self.bin_line_in_row(covers, bounds, segment_id, row1, cx, 0, p1x, fy1);
    }

    // ── Up-Right (↑→) ──

    fn outer_dda_up_right(
        &mut self,
        covers: &mut CoverStorage,
        bounds: &TileBounds,
        segment_id: u32,
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

        self.bin_line_in_row(covers, bounds, segment_id, row0, p0x, fy0, cx, 0);

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
                self.bin_line_in_row(covers, bounds, segment_id, idy, cx, TILE_H_F24DOT8, nx, 0);
                cx = nx;
                idy -= 1;
            }
        }

        self.bin_line_in_row(covers, bounds, segment_id, row1, cx, TILE_H_F24DOT8, p1x, fy1);
    }

    // ── Up-Left (↑←) ──

    fn outer_dda_up_left(
        &mut self,
        covers: &mut CoverStorage,
        bounds: &TileBounds,
        segment_id: u32,
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

        self.bin_line_in_row(covers, bounds, segment_id, row0, p0x, fy0, cx, 0);

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
                self.bin_line_in_row(covers, bounds, segment_id, idy, cx, TILE_H_F24DOT8, nx, 0);
                cx = nx;
                idy -= 1;
            }
        }

        self.bin_line_in_row(covers, bounds, segment_id, row1, cx, TILE_H_F24DOT8, p1x, fy1);
    }

    // ========================================================================
    // Stage 4: Inner DDA — split line across tile columns within one tile row
    // ========================================================================

    fn bin_line_in_row(
        &mut self,
        covers: &mut CoverStorage,
        bounds: &TileBounds,
        segment_id: u32,
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
            let col = (f24dot8_to_tile_col(p0x) - bounds.min_col) as usize;
            self.push_to_tile(covers, bounds, segment_id, tile_row, col, p0y, p1y);
            return;
        }

        if p0x < p1x {
            // Going right
            if p0y < p1y {
                self.inner_dda_right_down(covers, bounds, segment_id, tile_row, p0x, p0y, p1x, p1y);
            } else {
                self.inner_dda_right_up(covers, bounds, segment_id, tile_row, p0x, p0y, p1x, p1y);
            }
        } else {
            // Going left
            if p0y < p1y {
                self.inner_dda_left_down(covers, bounds, segment_id, tile_row, p0x, p0y, p1x, p1y);
            } else {
                self.inner_dda_left_up(covers, bounds, segment_id, tile_row, p0x, p0y, p1x, p1y);
            }
        }
    }

    // ── Right-Down (→↓) ──

    fn inner_dda_right_down(
        &mut self,
        covers: &mut CoverStorage,
        bounds: &TileBounds,
        segment_id: u32,
        tile_row: usize,
        p0x: i32, p0y: i32,
        p1x: i32, p1y: i32,
    ) {
         let col0_raw = f24dot8_to_tile_col(p0x) - bounds.min_col;
    let col1_raw = f24dot8_to_tile_col(p1x - 1) - bounds.min_col;

    // Guard against out-of-bounds from rounding
    if col0_raw < 0 || col1_raw < 0 {
        return;
    }
        let col0 = (f24dot8_to_tile_col(p0x) - bounds.min_col) as usize;
        let col1 = (f24dot8_to_tile_col(p1x - 1) - bounds.min_col) as usize;

        if col0 == col1 {
            self.push_to_tile(covers, bounds, segment_id, tile_row, col0, p0y, p1y);
            return;
        }

        let dx = p1x - p0x; // positive
        let dy = p1y - p0y; // positive

        let fx = p0x - tile_col_to_f24dot8(f24dot8_to_tile_col(p0x));
        let pp = (TILE_W_F24DOT8 - fx) * dy;
        let mut cy = p0y + pp / dx;

        self.push_to_tile(covers, bounds, segment_id, tile_row, col0, p0y, cy);

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
                self.push_to_tile(covers, bounds, segment_id, tile_row, idx, cy, ny);
                cy = ny;
                idx += 1;
            }
        }

        self.push_to_tile(covers, bounds, segment_id, tile_row, col1, cy, p1y);
    }

    // ── Right-Up (→↑) ──

    fn inner_dda_right_up(
        &mut self,
        covers: &mut CoverStorage,
        bounds: &TileBounds,
        segment_id: u32,
        tile_row: usize,
        p0x: i32, p0y: i32,
        p1x: i32, p1y: i32,
    ) {
        let col0 = (f24dot8_to_tile_col(p0x) - bounds.min_col) as usize;
        let col1 = (f24dot8_to_tile_col(p1x - 1) - bounds.min_col) as usize;

        if col0 == col1 {
            self.push_to_tile(covers, bounds, segment_id, tile_row, col0, p0y, p1y);
            return;
        }

        let dx = p1x - p0x; // positive
        let dy = p0y - p1y; // positive (going up, abs)

        let fx = p0x - tile_col_to_f24dot8(f24dot8_to_tile_col(p0x));
        let pp = (TILE_W_F24DOT8 - fx) * dy;
        let mut cy = p0y - pp / dx;

        self.push_to_tile(covers, bounds, segment_id, tile_row, col0, p0y, cy);

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
                self.push_to_tile(covers, bounds, segment_id, tile_row, idx, cy, ny);
                cy = ny;
                idx += 1;
            }
        }

        self.push_to_tile(covers, bounds, segment_id, tile_row, col1, cy, p1y);
    }

    // ── Left-Down (←↓) ──

    fn inner_dda_left_down(
        &mut self,
        covers: &mut CoverStorage,
        bounds: &TileBounds,
        segment_id: u32,
        tile_row: usize,
        p0x: i32, p0y: i32,
        p1x: i32, p1y: i32,
    ) {
        let col0 = (f24dot8_to_tile_col(p0x - 1) - bounds.min_col) as usize;
        let col1 = (f24dot8_to_tile_col(p1x) - bounds.min_col) as usize;

        if col0 == col1 {
            self.push_to_tile(covers, bounds, segment_id, tile_row, col0, p0y, p1y);
            return;
        }

        let dx = p0x - p1x; // positive (going left, abs)
        let dy = p1y - p0y; // positive (going down)

        let fx = p0x - tile_col_to_f24dot8(f24dot8_to_tile_col(p0x - 1));
        let pp = fx * dy;
        let mut cy = p0y + pp / dx;

        self.push_to_tile(covers, bounds, segment_id, tile_row, col0, p0y, cy);

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
                self.push_to_tile(covers, bounds, segment_id, tile_row, idx, cy, ny);
                cy = ny;
                idx -= 1;
            }
        }

        self.push_to_tile(covers, bounds, segment_id, tile_row, col1, cy, p1y);
    }

    // ── Left-Up (←↑) ──

    fn inner_dda_left_up(
        &mut self,
        covers: &mut CoverStorage,
        bounds: &TileBounds,
        segment_id: u32,
        tile_row: usize,
        p0x: i32, p0y: i32,
        p1x: i32, p1y: i32,
    ) {
        let col0 = (f24dot8_to_tile_col(p0x - 1) - bounds.min_col) as usize;
        let col1 = (f24dot8_to_tile_col(p1x) - bounds.min_col) as usize;

        if col0 == col1 {
            self.push_to_tile(covers, bounds, segment_id, tile_row, col0, p0y, p1y);
            return;
        }

        let dx = p0x - p1x; // positive (going left, abs)
        let dy = p0y - p1y; // positive (going up, abs)

        let fx = p0x - tile_col_to_f24dot8(f24dot8_to_tile_col(p0x - 1));
        let pp = fx * dy;
        let mut cy = p0y - pp / dx;

        self.push_to_tile(covers, bounds, segment_id, tile_row, col0, p0y, cy);

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
                self.push_to_tile(covers, bounds, segment_id, tile_row, idx, cy, ny);
                cy = ny;
                idx -= 1;
            }
        }

        self.push_to_tile(covers, bounds, segment_id, tile_row, col1, cy, p1y);
    }

    // ========================================================================
    // Stage 5: Push — record crossings + set tag + emit Block
    // ========================================================================

    #[inline]
    fn push_to_tile(
        &mut self,
        covers: &mut CoverStorage,
        bounds: &TileBounds,
        segment_id: u32,
        tile_row: usize,
        tile_col: usize,
        y0: i32,
        y1: i32,
    ) {
        if y0 == y1 {
            return;
        }


        if tile_row >= covers.rows() || tile_col >= covers.cols() {
    return;
}
        // Record per-scanline crossings.
        record_per_scanline_crossings(covers.crossings_at(tile_row, tile_col), y0, y1);

        // Set tag bit.
        covers.set_tag(tile_row, tile_col);

        // Emit Block record for GPU.
        let global_col = (bounds.min_col + tile_col as i32) as u16;
        let global_row = (bounds.min_row + tile_row as i32) as u16;
        self.data.push(Block::new(global_col, global_row, segment_id));
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
// ============================================================================

#[inline(always)]
fn record_per_scanline_crossings(crossings: &mut [i8; TILE_H], y0: i32, y1: i32) {
    if y0 == y1 {
        return;
    }

    let (y_top, y_bot, sign): (i32, i32, i8) = if y0 < y1 {
        (y0, y1, -1) // going down
    } else {
        (y1, y0, 1) // going up
    };

    // Pixel center for scanline s is at: s * 256 + 128
    // Line crosses center s if y_top < center AND center <= y_bot
    for s in 0..TILE_H {
        let center = (s as i32) * 256 + 128;
        if y_top < center && center <= y_bot {
            crossings[s] = crossings[s].saturating_add(sign);
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
