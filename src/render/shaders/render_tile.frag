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
// Line→ray winding contribution
//
// Returns ±1 if the horizontal ray going right from `pixel` crosses the line.
// Sign matches the CPU-side rule used in `record_per_scanline_crossings`:
//   line going DOWN  (p0.y < p1.y) → -1
//   line going UP    (p0.y > p1.y) → +1
// Half-open Y interval [y_min, y_max) matches the CPU side exactly.
// ---------------------------------------------------------------------------

float line_contribution(vec2 p0, vec2 p1, vec2 pixel) {
    if (p0.y == p1.y) return 0.0;

    float sign_v = (p0.y < p1.y) ? -1.0 : 1.0;
    float y_min  = min(p0.y, p1.y);
    float y_max  = max(p0.y, p1.y);

    if (pixel.y <  y_min) return 0.0;
    if (pixel.y >= y_max) return 0.0;

    float t      = (pixel.y - p0.y) / (p1.y - p0.y);
    float x_at_t = mix(p0.x, p1.x, t);

    return (pixel.x >= x_at_t) ? sign_v : 0.0;
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
    for (uint s = 0u; s < seg_count; s++) {
        uint line_idx = seg_offset + s;
        vec2 p0, p1;
        read_line(line_idx, p0, p1);
        winding += line_contribution(p0, p1, pixel);
    }

    // ── Fill rule ──
    float coverage;
    if (fill_rule == FILL_RULE_NONZERO) {
        // Inside if winding ≠ 0.  Use a comfortably-low threshold to absorb
        // any 8.8 quantization noise (½ unit = ~0.002 in winding space).
        coverage = (abs(winding) > 0.5) ? 1.0 : 0.0;
    } else {
        // EvenOdd: round to nearest int and check parity.
        int w = int(floor(abs(winding) + 0.5));
        coverage = ((w & 1) == 1) ? 1.0 : 0.0;
    }

    if (coverage <= 0.0) {
        discard;
    }

    vec4 paint = unpack_rgba8(v_payload);
    fragColor  = vec4(paint.rgb * paint.a * coverage, paint.a * coverage);
}
