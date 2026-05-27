#version 300 es
precision highp float;
precision highp int;
precision highp usampler2D;

#define TILE_WIDTH                       16u
#define TILE_HEIGHT                      8u

#define COLOR_SOURCE_PAYLOAD             0u
#define COLOR_SOURCE_PAINT               1u
#define PAINT_TYPE_SOLID                 0u

#define COLOR_SOURCE_SHIFT               30u
#define PAINT_TYPE_SHIFT                 27u
#define FILL_RULE_SHIFT                  24u
#define COLOR_SOURCE_MASK                0xC0000000u
#define PAINT_TYPE_MASK                  0x38000000u
#define FILL_RULE_MASK                   0x07000000u

#define FILL_RULE_NONZERO                0u
#define FILL_RULE_EVENODD                1u

// CPU stores per-scanline backdrop in 8.8 fixed-point: 256 = exactly one winding.
#define WINDING_UNIT                     256.0

layout(std140) uniform config {
    uint u_width;
    uint u_height;
    uint u_tile_height;
    uint u_segments_tex_width_bits;
    uint u_segment_list_tex_width_bits;
    uint u_encoded_paints_tex_width_bits;
    uint u_negate_ndc;
    uint _pad0;
};

uniform sampler2D u_segments_texture;

flat in ivec4 v_backdrop_lo; // Per-scanline backdrop rows 0..3 (8.8 fixed point)
flat in ivec4 v_backdrop_hi; // Per-scanline backdrop rows 4..7
flat in uvec2 v_segment;     // .x = line offset in segments tex, .y = line count
flat in uint  v_payload;
flat in uint  v_paint_flag;
flat in uvec2 v_tile_origin_pixels;
flat in uint  v_depth_index;
in vec2       v_local_xy;

out vec4 fragColor;

// ---------------------------------------------------------------------------
// Texture indexing
// ---------------------------------------------------------------------------

ivec2 segments_idx_to_coord(uint idx) {
    return ivec2(
        int(idx & ((1u << u_segments_tex_width_bits) - 1u)),
        int(idx >> u_segments_tex_width_bits)
    );
}

void read_line(uint line_idx, out vec2 p0, out vec2 p1) {
    // 1 flattened line = 1 RGBA32F texel = (p0.x, p0.y, p1.x, p1.y),
    // coords are in TILE-LOCAL pixels.
    vec4 t = texelFetch(u_segments_texture, segments_idx_to_coord(line_idx), 0);
    p0 = t.xy;
    p1 = t.zw;
}

// ---------------------------------------------------------------------------
// Backdrop lookup (per scanline row 0..7 inside this tile)
// ---------------------------------------------------------------------------

int read_backdrop(int row) {
    // Branchless gather across the two ivec4s.
    ivec4 lo = v_backdrop_lo;
    ivec4 hi = v_backdrop_hi;
    int b0 = (row == 0) ? lo.x : ((row == 1) ? lo.y : ((row == 2) ? lo.z : lo.w));
    int b1 = (row == 4) ? hi.x : ((row == 5) ? hi.y : ((row == 6) ? hi.z : hi.w));
    return (row < 4) ? b0 : b1;
}

// ---------------------------------------------------------------------------
// Analytic line-area contribution (BOX filter, radius 0.5)
//
// Returns the signed fraction of the unit-pixel box that lies "to the right"
// of the line within the line's y-range, scaled by the line's winding sign.
// Result is in [-1, +1]. This is the convolution of the line's half-plane
// indicator with a 1×1 box filter centered at `pixel`.
// ---------------------------------------------------------------------------
float line_box(vec2 p0, vec2 p1, vec2 pixel) {
    if (p0.y == p1.y) return 0.0;

    float sign_v;
    vec2 lo, hi;
    if (p0.y < p1.y) { sign_v = -1.0; lo = p0; hi = p1; }
    else             { sign_v = +1.0; lo = p1; hi = p0; }

    // Clip the line's y-range to the pixel's box y-extent [pixel.y ± 0.5].
    float py_lo = pixel.y - 0.5;
    float py_hi = pixel.y + 0.5;
    float y_lo  = max(lo.y, py_lo);
    float y_hi  = min(hi.y, py_hi);
    if (y_hi <= y_lo) return 0.0;

    // x-coordinates of the clipped sub-segment's endpoints.
    float dy   = hi.y - lo.y;
    float t_lo = (y_lo - lo.y) / dy;
    float t_hi = (y_hi - lo.y) / dy;
    float x_lo = mix(lo.x, hi.x, t_lo);
    float x_hi = mix(lo.x, hi.x, t_hi);

    // Trapezoidal area of the pixel-box region to the RIGHT of the line.
    float px_lo = pixel.x - 0.5;
    float px_hi = pixel.x + 0.5;
    float xc_lo = clamp(x_lo, px_lo, px_hi);
    float xc_hi = clamp(x_hi, px_lo, px_hi);
    float avg_x = (xc_lo + xc_hi) * 0.5;
    float h_cov = px_hi - avg_x;

    return sign_v * (y_hi - y_lo) * h_cov;
}

// ---------------------------------------------------------------------------
// Tent-filter line contribution (TENSOR TENT, radius 1)
//
// The 2D tensor tent has support 2×2 pixels: it extends 1 pixel above, below,
// left, and right of the pixel center, with weight tapering linearly to zero
// at the edges. Mathematically:
//
//     tent(s) = max(1 - |s|, 0)
//     h(x, y) = tent(x - px) · tent(y - py)
//
// We approximate the tent in y as a 5-sample weighted sum {0.04, 0.24, 0.44,
// 0.24, 0.04} at offsets {-0.8, -0.4, 0, +0.4, +0.8}, while keeping the
// x-direction analytic (using `line_box` at each y-sample). This is *not*
// the paper's full closed-form tent integral — that's a piecewise-polynomial
// sub-interval integration ~80 lines of GLSL — but it captures the tent's
// y-falloff with O(5×) the cost of `line_box`.
//
// !! IMPORTANT: this filter has radius 1 in y, so a pixel at the bottom of
// !! a tile reads samples up to +0.8 below itself — which can lie in the
// !! NEXT tile down. Without halo binning, lines from that neighbor tile
// !! are missing from `seg_count`, producing visible seams on tile bottom
// !! and top rows. That seam is exactly what this experiment is meant to
// !! reveal.
// ---------------------------------------------------------------------------

float line_tent(vec2 p0, vec2 p1, vec2 pixel) {
    // Discrete tent kernel (sampled in y, analytic in x).
    // Weights normalized so the sum equals 1.0.
    return
        0.04 * line_box(p0, p1, pixel + vec2(0.0, -0.8)) +
        0.24 * line_box(p0, p1, pixel + vec2(0.0, -0.4)) +
        0.44 * line_box(p0, p1, pixel) +
        0.24 * line_box(p0, p1, pixel + vec2(0.0, +0.4)) +
        0.04 * line_box(p0, p1, pixel + vec2(0.0, +0.8));
}

// ---------------------------------------------------------------------------
// Paint
// ---------------------------------------------------------------------------

vec4 unpack_rgba8(uint packed) {
    float r = float((packed >>  0u) & 0xFFu) / 255.0;
    float g = float((packed >>  8u) & 0xFFu) / 255.0;
    float b = float((packed >> 16u) & 0xFFu) / 255.0;
    float a = float((packed >> 24u) & 0xFFu) / 255.0;
    return vec4(r, g, b, a);
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

void main() {
    uint  fill_rule  = (v_paint_flag & FILL_RULE_MASK) >> FILL_RULE_SHIFT;
    uint  seg_offset = v_segment.x;
    uint  seg_count  = v_segment.y;

    // ── Pixel position in TILE-LOCAL pixel coordinates ──
    // `v_local_xy` is interpolated from the four tile-corner vertices, which
    // were emitted in INTEGER pixel-space. At a fragment in the middle of
    // pixel (X, Y), rasterization places gl_FragCoord at (X+0.5, Y+0.5), and
    // the interpolated v_local_xy lands at the same point. So v_local_xy
    // already IS the pixel-center position — no further +0.5 offset is
    // needed (and adding one introduces a half-pixel shift that mis-reads
    // the per-scanline backdrop).
    vec2 pixel        = v_local_xy - vec2(v_tile_origin_pixels);
    int  scanline_row = int(clamp(floor(pixel.y), 0.0, float(int(TILE_HEIGHT) - 1)));

    // ── Initial winding from per-scanline backdrop ──
    // CPU stored 8.8 fixed-point area where ±256 = exactly one winding.
    // The propagated backdrop is the running sum of all crossings to the
    // LEFT of this tile's column on this scanline.
    float backdrop = float(read_backdrop(scanline_row)) / WINDING_UNIT;
    float winding  = backdrop;

    // ── Add this tile's per-line contributions ──
    // Switching `line_box` ↔ `line_tent` here selects the AA filter:
    //   - line_box  : 1×1 support, sharp, no neighbor dependency.
    //   - line_tent : 2×2 support, softer, REQUIRES halo binning to be
    //                 artifact-free (we don't have it yet — see seams).
    for (uint s = 0u; s < seg_count; s++) {
        uint line_idx = seg_offset + s;
        vec2 p0, p1;
        read_line(line_idx, p0, p1);
        winding += line_box(p0, p1, pixel);
    }

    // ── Fill rule → coverage ──
    // The winding is now a continuous value, so the threshold becomes a clamp.
    float coverage;
    if (fill_rule == FILL_RULE_NONZERO) {
        // Inside if winding ≠ 0; coverage scales with how "full" the pixel is.
        coverage = clamp(abs(winding), 0.0, 1.0);
    } else {
        // EvenOdd: triangle-wave on |winding| → 0 at evens, 1 at odds.
        float w = abs(winding);
        coverage = 1.0 - abs(mod(w, 2.0) - 1.0);
    }

    if (coverage <= 0.0) {
        discard;
    }

    vec4 paint = unpack_rgba8(v_payload);
    fragColor  = vec4(paint.rgb * paint.a * coverage, paint.a * coverage);
}
