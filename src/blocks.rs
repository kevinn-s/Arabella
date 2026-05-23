use alloc::vec;
use alloc::vec::Vec;
use lyon_geom::euclid::default::Box2D;

pub const TILE_W: usize = 4;
pub const TILE_H: usize = 4;
pub const TILE_W_F: f32 = TILE_W as f32;
pub const TILE_H_F: f32 = TILE_H as f32;

// ============================================================================
// Block: per (curve, tile) record for GPU upload
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
        let min_col = (bounds.min.x / TILE_W_F).floor() as i32;
        let min_row = (bounds.min.y / TILE_H_F).floor() as i32;
        let max_col = (bounds.max.x / TILE_W_F).ceil() as i32;
        let max_row = (bounds.max.y / TILE_H_F).ceil() as i32;
        Self {
            min_col,
            min_row,
            col_count: (max_col - min_col).max(0) as usize,
            row_count: (max_row - min_row).max(0) as usize,
        }
    }
}

// ============================================================================
// Blocks: container that accumulates Block records via analytical binning
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

    /// After sorting, deduplicate so each (tile, segment_id) pair appears only once.
    pub fn dedup(&mut self) {
        debug_assert!(self.sorted);
        self.data.dedup_by(|a, b| a.x == b.x && a.y == b.y && a.segment_id == b.segment_id);
    }

    // ========================================================================
    // Analytical curve binning (GLLabel-style)
    //
    // For a quadratic bezier (p0, p1, p2), determine which tiles it intersects.
    // Method: intersect the curve with every tile grid line (both horizontal
    // and vertical). Each intersection tells us the curve crosses into an
    // adjacent tile. We also mark the tile containing p0 and p2.
    //
    // This is the same approach as GLLabel's `find_cells_intersections` in
    // vgrid.cpp, generalized to an arbitrary tile grid over the image.
    // ========================================================================

    pub fn bin_curve(
        &mut self,
        bounds: &TileBounds,
        segment_id: u32,
        p0x: f32, p0y: f32,
        p1x: f32, p1y: f32,
        p2x: f32, p2y: f32,
    ) {
        // Skip degenerate curves (zero height = no winding contribution)
        if (p0y - p2y).abs() < 1e-6 && (p0y - p1y).abs() < 1e-6 {
            return;
        }

        let mut any_intersection = false;

        // --- Intersect with every VERTICAL grid line (x = col * TILE_W) ---
        // Including left and right edges of the bounds
        for col_idx in 0..=bounds.col_count {
            let grid_x = (bounds.min_col + col_idx as i32) as f32 * TILE_W_F;

            let mut ty = [0.0_f32; 2];
            let n = solve_quadratic_bezier_component(p0x, p1x, p2x, grid_x, &mut ty);

            for i in 0..n {
                let t = ty[i];
                let cross_y = eval_bezier(p0y, p1y, p2y, t);
                let tile_row = ((cross_y / TILE_H_F).floor() as i32) - bounds.min_row;

                // Mark the tile to the left and right of this grid line
                let left_col = col_idx as i32 - 1;
                let right_col = col_idx as i32;

                if left_col >= 0 && (left_col as usize) < bounds.col_count
                    && tile_row >= 0 && (tile_row as usize) < bounds.row_count
                {
                    let gx = (bounds.min_col + left_col) as u16;
                    let gy = (bounds.min_row + tile_row as i32) as u16;
                    self.data.push(Block::new(gx, gy, segment_id));
                    any_intersection = true;
                }

                if right_col >= 0 && (right_col as usize) < bounds.col_count
                    && tile_row >= 0 && (tile_row as usize) < bounds.row_count
                {
                    let gx = (bounds.min_col + right_col) as u16;
                    let gy = (bounds.min_row + tile_row as i32) as u16;
                    self.data.push(Block::new(gx, gy, segment_id));
                    any_intersection = true;
                }
            }
        }

        // --- Intersect with every HORIZONTAL grid line (y = row * TILE_H) ---
        for row_idx in 0..=bounds.row_count {
            let grid_y = (bounds.min_row + row_idx as i32) as f32 * TILE_H_F;

            let mut tx = [0.0_f32; 2];
            let n = solve_quadratic_bezier_component(p0y, p1y, p2y, grid_y, &mut tx);

            for i in 0..n {
                let t = tx[i];
                let cross_x = eval_bezier(p0x, p1x, p2x, t);
                let tile_col = ((cross_x / TILE_W_F).floor() as i32) - bounds.min_col;

                // Mark tile above and below this grid line
                let above_row = row_idx as i32 - 1;
                let below_row = row_idx as i32;

                if tile_col >= 0 && (tile_col as usize) < bounds.col_count
                    && above_row >= 0 && (above_row as usize) < bounds.row_count
                {
                    let gx = (bounds.min_col + tile_col) as u16;
                    let gy = (bounds.min_row + above_row) as u16;
                    self.data.push(Block::new(gx, gy, segment_id));
                    any_intersection = true;
                }

                if tile_col >= 0 && (tile_col as usize) < bounds.col_count
                    && below_row >= 0 && (below_row as usize) < bounds.row_count
                {
                    let gx = (bounds.min_col + tile_col) as u16;
                    let gy = (bounds.min_row + below_row) as u16;
                    self.data.push(Block::new(gx, gy, segment_id));
                    any_intersection = true;
                }
            }
        }

        // If no grid line was crossed, the curve is fully contained in one tile.
        // Mark that tile.
        if !any_intersection {
            // Use any point on the curve (p0) to find the tile
            let col = ((p0x / TILE_W_F).floor() as i32) - bounds.min_col;
            let row = ((p0y / TILE_H_F).floor() as i32) - bounds.min_row;

            if col >= 0 && (col as usize) < bounds.col_count
                && row >= 0 && (row as usize) < bounds.row_count
            {
                let gx = (bounds.min_col + col) as u16;
                let gy = (bounds.min_row + row) as u16;
                self.data.push(Block::new(gx, gy, segment_id));
            }
        }
    }
}

// ============================================================================
// Analytical backdrop computation
//
// For each tile, compute the winding number "entering" from the left.
// For each pixel row (scanline) within a tile row, cast a horizontal ray
// from x = tile_left_edge. Count how many times each curve crosses that
// ray to the LEFT of tile_left_edge.
//
// Method: for each curve, for each tile column boundary x:
//   solve bezier_x(t) = x for t ∈ [0, 1)
//   for each valid t, evaluate bezier_y(t) to find which scanline row it hits
//   sign from bezier_y_derivative(t)
//
// This gives us the exact winding at each tile's left edge, matching what
// the shader will compute for crossings to the RIGHT of the pixel.
// ============================================================================

/// Compute per-tile backdrop for all tiles in a row.
/// Returns a 2D array: backdrop[col][scanline_row] = entering winding.
///
/// `segments` is the flat f32 array (8 floats per curve).
/// `curve_count` is total number of curves.
/// `tile_row` is which tile row (in global tile coords) we're computing for.
/// `bounds` gives the tile extent.
pub fn compute_row_backdrops(
    segments: &[f32],
    curve_count: usize,
    tile_row: i32,
    bounds: &TileBounds,
) -> Vec<[i16; TILE_H]> {
    let num_cols = bounds.col_count;
    let mut backdrops = vec![[0i16; TILE_H]; num_cols];

    // For each tile column, the left edge x coordinate
    // Backdrop for column C = winding at x = (bounds.min_col + C) * TILE_W
    //
    // Equivalent: for each curve, find all crossings with a horizontal ray
    // at each scanline Y (pixel_center_y), count those to the LEFT of each
    // column's left edge.
    //
    // Optimized approach: for each curve, for each scanline in this tile row,
    // solve bezier_y(t) = scanline_y, then evaluate bezier_x(t).
    // If bezier_x(t) < column_left_edge, that crossing contributes to
    // that column's backdrop.
    //
    // Even more efficient: compute all X crossings per scanline, sort them,
    // then do a prefix sum across columns.

    let row_top_y = (tile_row as f32) * TILE_H_F;

    for scanline in 0..TILE_H {
        // Pixel center Y for this scanline
        let pixel_y = row_top_y + scanline as f32 + 0.5;

        // Collect all (x_crossing, sign) pairs for this scanline
        let mut crossings: Vec<(f32, i16)> = Vec::new();

        for curve_idx in 0..curve_count {
            let base = curve_idx * 8;
            let p0x = segments[base];
            let p0y = segments[base + 1];
            let p1x = segments[base + 2];
            let p1y = segments[base + 3];
            let p2x = segments[base + 4];
            let p2y = segments[base + 5];

            // Quick Y-bounds check
            let y_lo = p0y.min(p1y).min(p2y);
            let y_hi = p0y.max(p1y).max(p2y);
            if pixel_y < y_lo || pixel_y >= y_hi {
                continue;
            }

            // Solve bezier_y(t) = pixel_y
            let mut roots = [0.0_f32; 2];
            let n = solve_quadratic_bezier_component(p0y, p1y, p2y, pixel_y, &mut roots);

            for i in 0..n {
                let t = roots[i];
                let cx = eval_bezier(p0x, p1x, p2x, t);

                // Direction at this crossing
                let dy = eval_bezier_deriv(p0y, p1y, p2y, t);
                let sign: i16 = if dy > 0.0 { 1 } else if dy < 0.0 { -1 } else { 0 };

                if sign != 0 {
                    crossings.push((cx, sign));
                }
            }
        }

        // Sort crossings by x
        crossings.sort_unstable_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(core::cmp::Ordering::Equal));

        // For each column, backdrop = sum of all crossings with x < column_left_edge
        // This is a prefix sum: walk crossings left-to-right, accumulate into columns
        let mut crossing_idx = 0;
        let mut acc: i16 = 0;

        for col in 0..num_cols {
            let col_left_x = (bounds.min_col + col as i32) as f32 * TILE_W_F;

            // Consume all crossings to the left of this column's left edge
            while crossing_idx < crossings.len() && crossings[crossing_idx].0 < col_left_x {
                acc += crossings[crossing_idx].1;
                crossing_idx += 1;
            }

            backdrops[col][scanline] = acc;
        }
    }

    backdrops
}

// ============================================================================
// Quadratic bezier math utilities
// ============================================================================

/// Solve: (p0 - 2*p1 + p2)*t^2 + 2*(p1 - p0)*t + (p0 - target) = 0
/// Returns number of roots in [0, 1). Roots stored in `out`.
fn solve_quadratic_bezier_component(p0: f32, p1: f32, p2: f32, target: f32, out: &mut [f32; 2]) -> usize {
    let c = p0 - target;
    let b = 2.0 * (p1 - p0);
    let a = p0 - 2.0 * p1 + p2;

    // Nearly-linear case
    if a.abs() < 1e-7 {
        if b.abs() < 1e-7 {
            return 0;
        }
        let root = -c / b;
        if root >= 0.0 && root < 1.0 {
            out[0] = root;
            return 1;
        }
        return 0;
    }

    // Quadratic formula
    let disc = b * b - 4.0 * a * c;
    if disc < 0.0 {
        return 0;
    }

    let sqrt_disc = disc.sqrt();
    let inv_2a = 0.5 / a;

    let r0 = (-b + sqrt_disc) * inv_2a;
    let r1 = (-b - sqrt_disc) * inv_2a;

    let mut count = 0;

    if r0 >= 0.0 && r0 < 1.0 {
        out[count] = r0;
        count += 1;
    }
    if r1 >= 0.0 && r1 < 1.0 {
        out[count] = r1;
        count += 1;
    }

    count
}

/// Evaluate quadratic bezier at parameter t.
#[inline(always)]
fn eval_bezier(p0: f32, p1: f32, p2: f32, t: f32) -> f32 {
    let mt = 1.0 - t;
    mt * mt * p0 + 2.0 * t * mt * p1 + t * t * p2
}

/// Evaluate quadratic bezier derivative at parameter t.
#[inline(always)]
fn eval_bezier_deriv(p0: f32, p1: f32, p2: f32, t: f32) -> f32 {
    2.0 * (1.0 - t) * (p1 - p0) + 2.0 * t * (p2 - p1)
}
