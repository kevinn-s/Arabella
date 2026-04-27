use alloc::vec;
use alloc::vec::Vec;
use bytemuck::{Pod, Zeroable};
use fearless_simd::{Simd, Level, dispatch};
use peniko::{
    BrushRef, Color, Fill,
    kurbo::{Affine, Shape},
};
use crate::Tile;
use crate::path::PathEncoder;

// ============================================================================
// Constants — paint encoding
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

// ============================================================================
// Tile struct (matches your scene.rs definition)
// ============================================================================



// ============================================================================
// Internal binning state (per-path scratch)
// ============================================================================

const TILE_W: f32 = 4.0;
const TILE_H: f32 = 4.0;

/// Settings to apply to the render context.
#[derive(Copy, Clone, Debug)]
pub struct RenderSettings {
    /// The SIMD level that should be used for rendering operations.
    pub level: Level,
}


impl Default for RenderSettings {
    fn default() -> Self {
        Self {
            level: Level::try_detect().unwrap_or(Level::new()),
        }
    }
}
/// Per-tile scratch state during binning. Contains count + per-pixel-row
/// backdrop deltas. Reused across paths via `clear()`.
#[derive(Clone, Copy, Default)]
#[derive(Debug)]
struct ScratchTile {
    count: u32,
    backdrop: [i16; Tile::PIXEL_ROWS],
}
#[derive(Debug)]
/// Working buffers for binning, owned by Scene to avoid per-path allocation.
pub struct BinningScratch {
    /// Per-tile scratch for the full scene grid. Reused per path.
    tiles: Vec<ScratchTile>,
    /// Per-tile offsets after prefix sum on counts. Length = total_tiles.
    offsets: Vec<u32>,
    /// Cursors during Pass 4 fill. Length = total_tiles.
    cursors: Vec<u32>,
    /// Flat segment_refs allocated per path (sized to total_refs).
    segment_refs: Vec<u32>,
    /// Cached scene grid dimensions
    tile_cols: usize,
    tile_rows: usize,
}

impl BinningScratch {
    pub fn new(width: u16, height: u16) -> Self {
        let tile_cols = width.div_ceil(Tile::WIDTH) as usize;
        let tile_rows = height.div_ceil(Tile::HEIGHT) as usize;
        let total_tiles = tile_cols * tile_rows;
        Self {
            tiles: vec![ScratchTile::default(); total_tiles],
            offsets: vec![0; total_tiles],
            cursors: vec![0; total_tiles],
            segment_refs: Vec::new(),
            tile_cols,
            tile_rows,
        }
    }

    fn clear(&mut self) {
        for t in self.tiles.iter_mut() {
            *t = ScratchTile::default();
        }
        self.segment_refs.clear();
    }
}

// ============================================================================
// Backdrop packing helpers
// ============================================================================

/// Pack 4 i16 backdrop values into [u32; 2] for the GPU Tile struct.
#[inline]
fn pack_backdrop(bd: &[i16; Tile::PIXEL_ROWS]) -> [u32; 2] {
    let lo = (bd[0] as u16 as u32) | ((bd[1] as u16 as u32) << 16);
    let hi = (bd[2] as u16 as u32) | ((bd[3] as u16 as u32) << 16);
    [lo, hi]
}

// ============================================================================
// Brush → paint encoding
// ============================================================================

/// Convert a brush into (payload, paint_and_rect_flag) words.
/// For now only handles solid colors. Gradients/images return placeholder.
fn encode_paint(brush: BrushRef<'_>) -> (u32, u32) {
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


// ============================================================================
// Curve evaluation helpers
// ============================================================================

#[inline]
fn eval_quad_x_at_t(p0x: f32, p1x: f32, p2x: f32, t: f32) -> f32 {
    let mt = 1.0 - t;
    mt * mt * p0x + 2.0 * mt * t * p1x + t * t * p2x
}

#[inline]
fn eval_quad_y_at_t(p0y: f32, p1y: f32, p2y: f32, t: f32) -> f32 {
    let mt = 1.0 - t;
    mt * mt * p0y + 2.0 * mt * t * p1y + t * t * p2y
}

/// Solve y(t) = y_target for a y-monotonic quadratic, return x(t).
fn eval_quad_x_at_y(
    p0x: f32, p0y: f32, p1x: f32, p1y: f32, p2x: f32, p2y: f32, y: f32,
) -> f32 {
    let a = p0y - 2.0 * p1y + p2y;
    let b = 2.0 * (p1y - p0y);
    let c = p0y - y;

    let t = if a.abs() < 1e-6 {
        if b.abs() < 1e-4 { 0.0 } else { -c / b }
    } else {
        let disc = (b * b - 4.0 * a * c).max(0.0);
        let sqrt_disc = disc.sqrt();
        let t1 = (-b + sqrt_disc) / (2.0 * a);
        let t2 = (-b - sqrt_disc) / (2.0 * a);
        if t1 >= 0.0 && t1 <= 1.0 { t1 } else { t2 }
    }
    .clamp(0.0, 1.0);

    eval_quad_x_at_t(p0x, p1x, p2x, t)
}

/// Solve x(t) = x_target for a quadratic, return y(t).
fn solve_quad_y_at_x(
    p0x: f32, p0y: f32, p1x: f32, p1y: f32, p2x: f32, p2y: f32, x: f32,
) -> f32 {
    let a = p0x - 2.0 * p1x + p2x;
    let b = 2.0 * (p1x - p0x);
    let c = p0x - x;

    let t = if a.abs() < 1e-6 {
        if b.abs() < 1e-6 { 0.0 } else { -c / b }
    } else {
        let disc = (b * b - 4.0 * a * c).max(0.0);
        let sqrt_disc = disc.sqrt();
        let t1 = (-b + sqrt_disc) / (2.0 * a);
        let t2 = (-b - sqrt_disc) / (2.0 * a);
        if t1 >= 0.0 && t1 <= 1.0 { t1 } else { t2 }
    }
    .clamp(0.0, 1.0);

    eval_quad_y_at_t(p0y, p1y, p2y, t)
}

// ============================================================================
// Pass 1 + 4: per-segment tile-row sweep
// ============================================================================

/// Compute (col_start, col_end, x_min_clamped, x_max_clamped, px_start, px_end)
/// for a line in one tile row. Returns None if culled.
#[inline]
fn line_x_range(
    p0x: f32, p0y: f32, p1x: f32, p1y: f32,
    line_left_x: f32, line_right_x: f32,
    tile_row: usize, y_min: f32, y_max: f32,
    tile_cols: usize, viewport_w: f32,
) -> Option<(usize, usize, [f32; Tile::PIXEL_ROWS], f32, f32)> {
    let y_top = (tile_row as f32 * TILE_H).max(y_min);
    let y_bot = ((tile_row + 1) as f32 * TILE_H).min(y_max);

    if y_bot <= y_top { return None; }

    let dy = p1y - p0y;
    if dy.abs() < 1e-6 { return None; }
    let x_slope = (p1x - p0x) / dy;

    let x_top = p0x + (y_top - p0y) * x_slope;
    let x_bot = p0x + (y_bot - p0y) * x_slope;

    let x_min = x_top.min(x_bot).max(line_left_x).max(0.0);
    let x_max = x_top.max(x_bot).min(line_right_x).min(viewport_w);

    if x_max < x_min { return None; }

    let col_start = (x_min / TILE_W) as usize;
    let col_end = ((x_max / TILE_W) as usize).min(tile_cols.saturating_sub(1));

    // Calculate exact sub-pixel Delta Y for each pixel row
    let tile_row_y = tile_row as f32 * TILE_H;
    let mut coverage = [0.0f32; Tile::PIXEL_ROWS];
    let mut current_y = y_top;
    
    while current_y < y_bot {
        let px = ((current_y - tile_row_y).floor() as usize).min(Tile::PIXEL_ROWS - 1);
        let pixel_bottom = tile_row_y + (px + 1) as f32;
        let next_y = pixel_bottom.min(y_bot);
        let segment_dy = next_y - current_y;

        coverage[px] += segment_dy;
        current_y = next_y;
    }

    Some((col_start, col_end, coverage, x_min, x_max))
}

/// Compute (col_start, col_end, ...) for a y-monotonic quadratic in one tile row.
#[inline]
fn quad_x_range(
    p0x: f32, p0y: f32, p1x: f32, p1y: f32, p2x: f32, p2y: f32,
    quad_left_x: f32, quad_right_x: f32,
    tile_row: usize, y_min: f32, y_max: f32,
    tile_cols: usize, viewport_w: f32,
) -> Option<(usize, usize, [f32; Tile::PIXEL_ROWS], f32, f32)> { // <-- Updated return type
    let y_top = (tile_row as f32 * TILE_H).max(y_min);
    let y_bot = ((tile_row + 1) as f32 * TILE_H).min(y_max);

    if y_bot <= y_top { return None; }

    let x_top = eval_quad_x_at_y(p0x, p0y, p1x, p1y, p2x, p2y, y_top);
    let x_bot = eval_quad_x_at_y(p0x, p0y, p1x, p1y, p2x, p2y, y_bot);

    let mut x_min = x_top.min(x_bot);
    let mut x_max = x_top.max(x_bot);

    // X-extremum check
    let denom = p0x - 2.0 * p1x + p2x;
    if denom.abs() > 1e-4 {
        let t_ext = (p0x - p1x) / denom;
        if t_ext > 0.0 && t_ext < 1.0 {
            let y_ext = eval_quad_y_at_t(p0y, p1y, p2y, t_ext);
            if y_ext >= y_top && y_ext <= y_bot {
                let x_ext = eval_quad_x_at_t(p0x, p1x, p2x, t_ext);
                x_min = x_min.min(x_ext);
                x_max = x_max.max(x_ext);
            }
        }
    }

    // Layer 1 + Layer 2 clamping
    x_min = x_min.max(quad_left_x).max(0.0);
    x_max = x_max.min(quad_right_x).min(viewport_w);

    if x_max < x_min { return None; }

    let col_start = (x_min / TILE_W) as usize;
    let col_end = ((x_max / TILE_W) as usize).min(tile_cols.saturating_sub(1));

    // --- NEW: Fractional Sub-pixel Coverage Calculation ---
    let tile_row_y = tile_row as f32 * TILE_H;
    let mut coverage = [0.0f32; Tile::PIXEL_ROWS];
    let mut current_y = y_top;
    
    // Slice the bounding y_top to y_bot into exact pixel rows
    while current_y < y_bot {
        let px = ((current_y - tile_row_y).floor() as usize).min(Tile::PIXEL_ROWS - 1);
        let pixel_bottom = tile_row_y + (px + 1) as f32;
        let next_y = pixel_bottom.min(y_bot);
        let segment_dy = next_y - current_y;

        coverage[px] += segment_dy;
        current_y = next_y;
    }

    Some((col_start, col_end, coverage, x_min, x_max))
}

// ============================================================================
// Main binning entry point
// ============================================================================

pub struct BinResult {
    /// Number of tile instances appended to the GPU tile vec.
    pub n_tiles_emitted: u32,
}

/// Bin one path's segments into tile instances and append to scene buffers.
///
/// - `lines`: f32 SoA, stride 4 (p0x, p0y, p1x, p1y)
/// - `quads`: f32 SoA, stride 6 (p0x, p0y, p1x, p1y, p2x, p2y), y-monotonic
/// - `gpu_segments`: scene-wide segment texture buffer, will be appended
/// - `gpu_tiles`: scene-wide tile vec, will be appended
/// - `scratch`: reused per-path working memory
pub fn bin_path(
    lines: &[u32],
    quads: &[u32],
    fill_rule: Fill,
    paint_payload: u32,
    paint_flag: u32,
    depth_index: u32,
    width: u16,
    height: u16,
    gpu_segments: &mut Vec<f32>,
    gpu_tiles: &mut Vec<Tile>,
    scratch: &mut BinningScratch,
) -> BinResult {
    if width == 0 || height == 0 {
        return BinResult { n_tiles_emitted: 0 };
    }

    // Reset scratch but keep allocations
    scratch.clear();

    let tile_cols = scratch.tile_cols;
    let tile_rows = scratch.tile_rows;
    let total_tiles = tile_cols * tile_rows;
    let viewport_w = width as f32;
    let viewport_h = height as f32;

    let n_lines = lines.len() / 4;
    let n_quads = quads.len() / 6;

    let fill_rule_word = match fill_rule {
        Fill::NonZero => FILL_RULE_NONZERO,
        Fill::EvenOdd => FILL_RULE_EVENODD,
    };

    // ── Pass 1: count + backdrop deltas ──

    for i in 0..n_lines {
        let li = i * 4;
        let p0x = f32::from_bits(lines[li]);
        let p0y = f32::from_bits(lines[li + 1]);
        let p1x = f32::from_bits(lines[li + 2]);
        let p1y = f32::from_bits(lines[li + 3]);

        if (p1y - p0y).abs() < 1e-6 { continue; }

        let winding: i16 = if p1y > p0y { 1 } else { -1 };
        let y_min = p0y.min(p1y).max(0.0);
        let y_max = p0y.max(p1y).min(viewport_h);
        if y_min >= y_max { continue; }

        let line_left_x = p0x.min(p1x);
        let line_right_x = p0x.max(p1x);

        let row_start = ((y_min / TILE_H) as usize).min(tile_rows.saturating_sub(1));
        let row_end = ((y_max / TILE_H) as usize).min(tile_rows.saturating_sub(1));

        for tile_row in row_start..=row_end {
            if let Some((col_start, col_end,cov_deltas, _, _)) = line_x_range(
                p0x, p0y, p1x, p1y,
                line_left_x, line_right_x,
                tile_row, y_min, y_max,
                tile_cols, viewport_w,
            ) {
                // Count
                for col in col_start..=col_end {
        scratch.tiles[tile_row * tile_cols + col].count += 1;
    }
    let exit_col = col_end + 1;
    if exit_col < tile_cols {
        let idx = tile_row * tile_cols + exit_col;
        
        // Direction factor: positive going down, negative going up
        let sign = if p1y > p0y { 1.0 } else { -1.0 };
        
        for px in 0..Tile::PIXEL_ROWS {
            if cov_deltas[px] > 0.0 {
                if cov_deltas[px] > 0.0 {
        // Use the fractional delta (8.8 fixed point)
        let delta = (cov_deltas[px] * sign * 256.0).round() as i16;
        scratch.tiles[idx].backdrop[px] += delta; // Add the fraction, not the whole 1
    }
            }
        }
    }
            }
        }
    }

    for i in 0..n_quads {
        let qi = i * 6;
        let p0x = f32::from_bits(quads[qi]);
        let p0y = f32::from_bits(quads[qi + 1]);
        let p1x = f32::from_bits(quads[qi + 2]);
        let p1y = f32::from_bits(quads[qi + 3]);
        let p2x = f32::from_bits(quads[qi + 4]);
        let p2y = f32::from_bits(quads[qi + 5]);

        if (p2y - p0y).abs() < 1e-6 { continue; }

        let winding: i16 = if p2y > p0y { 1 } else { -1 };
        let y_min = p0y.min(p2y).max(0.0);
        let y_max = p0y.max(p2y).min(viewport_h);
        if y_min >= y_max { continue; }

        let quad_left_x = p0x.min(p1x).min(p2x);
        let quad_right_x = p0x.max(p1x).max(p2x);

        let row_start = ((y_min / TILE_H) as usize).min(tile_rows.saturating_sub(1));
        let row_end = ((y_max / TILE_H) as usize).min(tile_rows.saturating_sub(1));

        for tile_row in row_start..=row_end {
            if let Some((col_start, col_end, cov_deltas, _, _)) = quad_x_range(
                p0x, p0y, p1x, p1y, p2x, p2y,
                quad_left_x, quad_right_x,
                tile_row, y_min, y_max,
                tile_cols, viewport_w,
            ) {
                for col in col_start..=col_end {
        scratch.tiles[tile_row * tile_cols + col].count += 1;
    }
    let exit_col = col_end + 1;
    if exit_col < tile_cols {
        let idx = tile_row * tile_cols + exit_col;
        
        // Quads are y-monotonic, so evaluate total direction from p0 to p2
        let sign = if p2y > p0y { 1.0 } else { -1.0 };
        
        for px in 0..Tile::PIXEL_ROWS {
            if cov_deltas[px] > 0.0 {
        // Use the fractional delta (8.8 fixed point)
        let delta = (cov_deltas[px] * sign * 256.0).round() as i16;
        scratch.tiles[idx].backdrop[px] += delta; // Add the fraction, not the whole 1
    }
        }
    }
            }
        }
    }

    // ── Pass 2: prefix sum backdrop (left to right per tile row, per pixel row) ──

    for row in 0..tile_rows {
        let mut acc = [0i16; Tile::PIXEL_ROWS];
        for col in 0..tile_cols {
            let idx = row * tile_cols + col;
            for px in 0..Tile::PIXEL_ROWS {
                acc[px] += scratch.tiles[idx].backdrop[px];
                scratch.tiles[idx].backdrop[px] = acc[px];
            }
        }
    }

    // ── Pass 3: prefix sum counts → offsets ──

    let mut acc: u32 = 0;
    for i in 0..total_tiles {
        scratch.offsets[i] = acc;
        acc += scratch.tiles[i].count;
    }
    let total_refs = acc as usize;

    // ── Pass 4: fill segment_refs (re-sweep) ──

  // ── Pass 4: fill segment_refs (re-sweep) ──

scratch.segment_refs.clear();
scratch.segment_refs.resize(total_refs, 0);
scratch.cursors.copy_from_slice(&scratch.offsets);

// Lines
for i in 0..n_lines {
    let li = i * 4;
    let p0x = f32::from_bits(lines[li]);
    let p0y = f32::from_bits(lines[li + 1]);
    let p1x = f32::from_bits(lines[li + 2]);
    let p1y = f32::from_bits(lines[li + 3]);

    if (p1y - p0y).abs() < 1e-6 { continue; }
    let y_min = p0y.min(p1y).max(0.0);
    let y_max = p0y.max(p1y).min(viewport_h);
    if y_min >= y_max { continue; }

    let line_left_x = p0x.min(p1x);
    let line_right_x = p0x.max(p1x);
    let row_start = ((y_min / TILE_H) as usize).min(tile_rows.saturating_sub(1));
    let row_end = ((y_max / TILE_H) as usize).min(tile_rows.saturating_sub(1));

    let seg_idx = i as u32;

    for tile_row in row_start..=row_end {
        if let Some((col_start, col_end, _, _, _)) = line_x_range(
            p0x, p0y, p1x, p1y,
            line_left_x, line_right_x,
            tile_row, y_min, y_max,
            tile_cols, viewport_w,
        ) {
            // WRITE segment_refs using cursors — NO count increment, NO backdrop
            for col in col_start..=col_end {
                let tile_idx = tile_row * tile_cols + col;
                let cursor = scratch.cursors[tile_idx] as usize;
                scratch.segment_refs[cursor] = seg_idx;
                scratch.cursors[tile_idx] += 1;
            }
        }
    }
}

// Quads
for i in 0..n_quads {
    let qi = i * 6;
    let p0x = f32::from_bits(quads[qi]);
    let p0y = f32::from_bits(quads[qi + 1]);
    let p1x = f32::from_bits(quads[qi + 2]);
    let p1y = f32::from_bits(quads[qi + 3]);
    let p2x = f32::from_bits(quads[qi + 4]);
    let p2y = f32::from_bits(quads[qi + 5]);

    if (p2y - p0y).abs() < 1e-6 { continue; }
    let y_min = p0y.min(p2y).max(0.0);
    let y_max = p0y.max(p2y).min(viewport_h);
    if y_min >= y_max { continue; }

    let quad_left_x = p0x.min(p1x).min(p2x);
    let quad_right_x = p0x.max(p1x).max(p2x);
    let row_start = ((y_min / TILE_H) as usize).min(tile_rows.saturating_sub(1));
    let row_end = ((y_max / TILE_H) as usize).min(tile_rows.saturating_sub(1));

    let seg_idx = (n_lines + i) as u32;

    for tile_row in row_start..=row_end {
        if let Some((col_start, col_end, _, _, _)) = quad_x_range(
            p0x, p0y, p1x, p1y, p2x, p2y,
            quad_left_x, quad_right_x,
            tile_row, y_min, y_max,
            tile_cols, viewport_w,
        ) {
            // WRITE segment_refs using cursors — NO count increment, NO backdrop
            for col in col_start..=col_end {
                let tile_idx = tile_row * tile_cols + col;
                let cursor = scratch.cursors[tile_idx] as usize;
                scratch.segment_refs[cursor] = seg_idx;
                scratch.cursors[tile_idx] += 1;
            }
        }
    }
} 
    // ── Pass 5: pack GPU segments (uniform 6-float quads) ──

    let gpu_seg_base = (gpu_segments.len() / 6) as u32;
    gpu_segments.reserve(total_refs * 6);

    for &seg_idx in scratch.segment_refs.iter() {
        if (seg_idx as usize) < n_lines {
            // Line → degenerate quad with midpoint
            let li = seg_idx as usize * 4;
            let p0x = f32::from_bits(lines[li]);
            let p0y = f32::from_bits(lines[li + 1]);
            let p1x = f32::from_bits(lines[li + 2]);
            let p1y = f32::from_bits(lines[li + 3]);
            gpu_segments.extend_from_slice(&[
                p0x, p0y,
                (p0x + p1x) * 0.5, (p0y + p1y) * 0.5,
                p1x, p1y,
            ]);
        } else {
            // Quad → as-is
            let qi = (seg_idx as usize - n_lines) * 6;
            for j in 0..6 {
                gpu_segments.push(f32::from_bits(quads[qi + j]));
            }
        }
    }

    // ── Pass 6: classify + emit GPU tiles ──

    let mut n_emitted: u32 = 0;
    let final_paint_flag = paint_flag | (fill_rule_word << 24);

    for row in 0..tile_rows {
        for col in 0..tile_cols {
            let idx = row * tile_cols + col;
            let scratch_t = scratch.tiles[idx];
            let count = scratch_t.count;
            let bd = scratch_t.backdrop;

            // Classify per fill rule
            let mut all_outside = true;
            let mut all_inside = true;
            for px in 0..Tile::PIXEL_ROWS {
                let inside = match fill_rule {
                    Fill::EvenOdd => (bd[px].unsigned_abs() & 1) != 0,
                    Fill::NonZero => bd[px] != 0,
                };
                if inside { all_outside = false; }
                else { all_inside = false; }
            }

            if count == 0 && all_outside {
                continue; // EMPTY — skip
            }

            let backdrop_packed = pack_backdrop(&bd);
            let segment_pair = if count == 0 && all_inside {
                // SOLID — no segments needed
                [0u32, 0u32]
            } else {
                // PARTIAL — has segments
                let local_offset = scratch.offsets[idx];
                let global_offset = gpu_seg_base + local_offset;
                [count, global_offset]
            };

            gpu_tiles.push(Tile {
                x: col as u16,
                y: row as u16,
                width: Tile::WIDTH as u8,
                height: Tile::HEIGHT as u8,
                _pad: [0; 2],
                backdrop: backdrop_packed,
                segment: segment_pair,
                payload: paint_payload,
                paint_and_rect_flag: final_paint_flag,
                depth_index,
                _final_pad: 0
            });
            n_emitted += 1;
        }
    }

    BinResult { n_tiles_emitted: n_emitted }
}

// ============================================================================
// Scene integration
// ============================================================================

const COLOR_SOURCE_SHIFT: u32 = 30;
const PAINT_TYPE_SHIFT: u32 = 27;
const FILL_RULE_SHIFT: u32 = 24;

use crate::paint::EncodedPaint;
use core::cell::RefCell;
use core::cell::{Ref, RefMut};

#[derive(Debug)]
pub struct Scene {
    pub(crate) width: u16,
    pub(crate) height: u16,
 
    pub(crate) segments: RefCell<Vec<f32>>,
    pub(crate) tiles: RefCell<Vec<Tile>>,
    pub(crate) encoded_paints: RefCell<Vec<EncodedPaint>>,
    binning_scratch: RefCell<BinningScratch>,
 
    encoder_lines: RefCell<Vec<u32>>,
    encoder_quads: RefCell<Vec<u32>>,
    encoder_n_segments: RefCell<u32>,
    encoder_n_paths: RefCell<u32>,
 
    next_depth: i32,
 
    level: Level,
}

impl Scene {
    pub fn new(width: u16, height: u16) -> Self {
        let level = Level::new(); // auto-detect best SIMD level
        Self {
            width,
            height,
            segments: RefCell::new(Vec::new()),
            tiles: RefCell::new(Vec::new()),
            encoded_paints: RefCell::new(Vec::new()),
            binning_scratch: RefCell::new(BinningScratch::new(width, height)),
            encoder_lines: RefCell::new(Vec::new()),
            encoder_quads: RefCell::new(Vec::new()),
            encoder_n_segments: RefCell::new(0),
            encoder_n_paths: RefCell::new(0),
            next_depth: 0,
            level,
        }
    }
 
    pub fn fill<'b>(
        &mut self,
        style: Fill,
        transform: Affine,
        brush: impl Into<BrushRef<'b>>,
        brush_transform: Option<Affine>,
        shape: &impl Shape,
    ) {
        let brush_ref: BrushRef = brush.into();
        let (payload, paint_flag) = encode_paint(brush_ref);
 
        // Inject the fill rule into the flag
        let fill_rule_word = match style {
            Fill::NonZero => FILL_RULE_NONZERO,
            Fill::EvenOdd => FILL_RULE_EVENODD,
        };
        let final_paint_flag = paint_flag | (fill_rule_word << FILL_RULE_SHIFT);
 
        let depth = self.next_depth as u32;
        self.next_depth += 1;
        // Dispatch into SIMD-specific implementation
        dispatch!(self.level, simd => self.fill_impl(
            simd, style, transform, payload, final_paint_flag, depth, shape
        ));
 
        let _ = brush_transform; // unused for now
    }


    pub fn depth(&mut self) -> i32{
        self.next_depth
    }
 
    #[inline(always)]
    fn fill_impl<S: Simd>(
        &self,
        simd: S,
        style: Fill,
        transform: Affine,
        payload: u32,
        paint_flag: u32,
        depth: u32,
        shape: &impl Shape,
    ) {
        // ── Encode path into reusable line/quad buffers ──
        let mut lines = self.encoder_lines.borrow_mut();
        let mut quads = self.encoder_quads.borrow_mut();
        let mut n_segs = self.encoder_n_segments.borrow_mut();
        let mut n_paths = self.encoder_n_paths.borrow_mut();
 
        lines.clear();
        quads.clear();
 
        {
            let mut encoder = PathEncoder::new(
                &mut lines,
                &mut quads,
                &mut n_segs,
                &mut n_paths,
                true, // is_fill
                transform,
                simd,
            );
            encoder.shape(shape);
        }
 

        // ── Bin and emit GPU tiles ──
        let mut tiles = self.tiles.borrow_mut();
        let mut segments = self.segments.borrow_mut();
        let mut scratch = self.binning_scratch.borrow_mut();
 
        let result = bin_path(
            &lines,
            &quads,
            style,
            payload,
            paint_flag,
            depth,
            self.width,
            self.height,
            &mut segments,
            &mut tiles,
            &mut scratch,
        );
 
        log::debug!("Emitted {} GPU tiles", result.n_tiles_emitted);
    }
 
    pub fn clear(&mut self) {
        self.segments.borrow_mut().clear();
        self.tiles.borrow_mut().clear();
        self.next_depth = 0;
    }
 
    pub fn tiles(&self) -> Ref<'_, Vec<Tile>> {
        self.tiles.borrow()
    }
 
    pub fn segments(&self) -> Ref<'_, Vec<f32>> {
        self.segments.borrow()
    }
 
    pub fn segments_mut(&self) -> RefMut<'_, Vec<f32>> {
        self.segments.borrow_mut()
    }
 
    pub fn width(&self) -> u16 {
        self.width
    }
 
    pub fn height(&self) -> u16 {
        self.height
    }
}
