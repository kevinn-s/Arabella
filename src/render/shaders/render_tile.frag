#version 300 es
precision highp float;
precision highp int;

// ============================================================================
// Constants — must match Rust side
// ============================================================================

#define TILE_WIDTH  4u
#define TILE_HEIGHT 4u
#define TILE_PIXEL_ROWS 4

#define COLOR_SOURCE_PAYLOAD 0u
#define COLOR_SOURCE_PAINT   1u

#define PAINT_TYPE_SOLID            0u
#define PAINT_TYPE_LINEAR_GRADIENT  1u
#define PAINT_TYPE_RADIAL_GRADIENT  2u
#define PAINT_TYPE_SWEEP_GRADIENT   3u
#define PAINT_TYPE_IMAGE            4u

#define COLOR_SOURCE_SHIFT 30u
#define PAINT_TYPE_SHIFT   27u
#define FILL_RULE_SHIFT    24u
#define COLOR_SOURCE_MASK  0xC0000000u
#define PAINT_TYPE_MASK    0x38000000u
#define FILL_RULE_MASK     0x07000000u
#define PAINT_INDEX_MASK   0x00FFFFFFu

#define FILL_RULE_NONZERO 0u
#define FILL_RULE_EVENODD 1u

// ============================================================================
// Uniforms
// ============================================================================

layout(std140) uniform config {
    uint u_width;
    uint u_height;
    uint u_tile_height;
    uint u_segments_tex_width_bits;
    uint u_encoded_paints_tex_width_bits;
    uint u_negate_ndc;
    uint _pad0;
    uint _pad1;
};

uniform sampler2D segments_texture;

// ============================================================================
// Varyings from vertex shader
// ============================================================================

flat in uvec2 v_backdrop;
flat in uvec2 v_segment;
flat in uint  v_payload;
flat in uint  v_paint_flag;
flat in uvec2 v_tile_origin_pixels;
in vec2 v_local_xy;

out vec4 fragColor;

// ============================================================================
// Helpers
// ============================================================================

int unpack_i16(uint word, uint half_idx) {
    uint shifted = (word >> (half_idx * 16u)) & 0xFFFFu;
    if ((shifted & 0x8000u) != 0u) {
        return int(shifted | 0xFFFF0000u);
    } else {
        return int(shifted);
    }
}

int get_backdrop(uvec2 backdrop, int pixel_row) {
    uint word = (pixel_row < 2) ? backdrop.x : backdrop.y;
    uint half_idx = uint(pixel_row & 1);
    return unpack_i16(word, half_idx);
}

void read_segment(uint seg_offset, out vec2 p0, out vec2 p1, out vec2 p2) {
    uint base_texel = seg_offset * 2u;
    uint tex_width = uint(textureSize(segments_texture, 0).x);

    uint t0_x = base_texel & (tex_width - 1u);
    uint t0_y = base_texel >> u_segments_tex_width_bits;
    uint t1_idx = base_texel + 1u;
    uint t1_x = t1_idx & (tex_width - 1u);
    uint t1_y = t1_idx >> u_segments_tex_width_bits;

    vec4 texel0 = texelFetch(segments_texture, ivec2(int(t0_x), int(t0_y)), 0);
    vec4 texel1 = texelFetch(segments_texture, ivec2(int(t1_x), int(t1_y)), 0);

    p0 = texel0.xy;
    p1 = texel0.zw;
    p2 = texel1.xy;
}

// ============================================================================
// Extended Implicit Geometry (Loop-Blinn) Evaluation
// ============================================================================
//
// We replace the quadratic raycast with purely algebraic implicit function 
// testing. By finding the barycentric coordinates of the pixel and checking 
// the sign of (U^2 - V), we can flawlessly determine if the pixel is to the 
// right or left of the curve without any square roots.

float quad_implicit_contribution(vec2 p0, vec2 p1, vec2 p2, vec2 pixel) {
    float y_min, y_max, sign_v;
    if (p2.y > p0.y) {
        sign_v = 1.0; y_min = p0.y; y_max = p2.y;
    } else if (p2.y < p0.y) {
        sign_v = -1.0; y_min = p2.y; y_max = p0.y;
    } else {
        return 0.0;
    }

    if (pixel.y < y_min || pixel.y >= y_max) {
        return 0.0;
    }

    // 1. Where is the pixel relative to the straight Chord (P0 -> P2)?
    float t_chord = (pixel.y - p0.y) / (p2.y - p0.y);
    float x_chord = p0.x + t_chord * (p2.x - p0.x);
    bool right_of_chord = pixel.x > x_chord;

    // 2. Degenerate triangle check (Area ~ 0)
    // If the control points form a straight line, just use the chord test.
    float denom = (p1.y - p2.y) * (p0.x - p2.x) + (p2.x - p1.x) * (p0.y - p2.y);
    if (abs(denom) < 1e-6) {
        return right_of_chord ? sign_v : 0.0;
    }

    // 3. Barycentric Coordinates
    float inv_denom = 1.0 / denom;
    float w0 = ((p1.y - p2.y) * (pixel.x - p2.x) + (p2.x - p1.x) * (pixel.y - p2.y)) * inv_denom;
    float w1 = ((p2.y - p0.y) * (pixel.x - p2.x) + (p0.x - p2.x) * (pixel.y - p2.y)) * inv_denom;
    float w2 = 1.0 - w0 - w1;

    // 4. Loop-Blinn Implicit Function (U^2 - V)
    float u = 0.5 * w1 + w2;
    float v = w2;
    float f = u * u - v; // f < 0 means inside the parabola

    // 5. Does the curve bulge right or left of the chord?
    float x_chord_p1 = p0.x + ((p1.y - p0.y) / (p2.y - p0.y)) * (p2.x - p0.x);
    bool bulges_right = p1.x > x_chord_p1;

    // 6. The Boolean Union!
    bool right_of_curve = false;
    if (bulges_right) {
        // Curve is right of chord. To be right of curve, must be right of chord AND outside parabola.
        right_of_curve = right_of_chord && (f > 0.0);
    } else {
        // Curve is left of chord. To be right of curve, must be right of chord OR inside parabola.
        right_of_curve = right_of_chord || (f < 0.0);
    }

    return right_of_curve ? sign_v : 0.0;
}

// ============================================================================
// Paint resolution
// ============================================================================

vec4 unpack_rgba8(uint packed) {
    float r = float((packed >>  0u) & 0xFFu) / 255.0;
    float g = float((packed >>  8u) & 0xFFu) / 255.0;
    float b = float((packed >> 16u) & 0xFFu) / 255.0;
    float a = float((packed >> 24u) & 0xFFu) / 255.0;
    return vec4(r, g, b, a);
}

vec4 resolve_paint() {
    uint source = (v_paint_flag & COLOR_SOURCE_MASK) >> COLOR_SOURCE_SHIFT;
    uint paint_type = (v_paint_flag & PAINT_TYPE_MASK) >> PAINT_TYPE_SHIFT;

    if (source == COLOR_SOURCE_PAYLOAD && paint_type == PAINT_TYPE_SOLID) {
        return unpack_rgba8(v_payload);
    }

    return vec4(1.0, 0.0, 1.0, 1.0);
}

// ============================================================================
// Main
// ============================================================================

void main() {
    uint fill_rule = (v_paint_flag & FILL_RULE_MASK) >> FILL_RULE_SHIFT;
    uint seg_count  = v_segment.x;
    uint seg_offset = v_segment.y;

    vec2 pixel_xy = gl_FragCoord.xy;
    if (u_negate_ndc == 0u) {
        pixel_xy.y = float(u_height) - pixel_xy.y;
    }

    int pixel_row = int(pixel_xy.y) - int(v_tile_origin_pixels.y);
    pixel_row = clamp(pixel_row, 0, TILE_PIXEL_ROWS - 1);

    // FIXED: Decode backdrop to a float properly
    float backdrop = float(get_backdrop(v_backdrop, pixel_row)) / 256.0;

    // ── Solid tile fast path ──
    if (seg_count == 0u) {
        float solid_coverage = 0.0;
        if (fill_rule == FILL_RULE_EVENODD) {
            solid_coverage = 1.0 - abs(mod(backdrop, 2.0) - 1.0);
        } else {
            solid_coverage = clamp(abs(backdrop), 0.0, 1.0);
        }
        
        if (solid_coverage > 0.0) {
            vec4 paint = resolve_paint();
            fragColor = vec4(paint.rgb, paint.a * solid_coverage);
            return;
        } else {
            discard;
        }
    }

    // ── Partial tile: Single Sample (No AA) ──
    // Center the sample exactly in the middle of the pixel
    vec2 sample_pos = vec2(floor(pixel_xy.x) + 0.5, floor(pixel_xy.y) + 0.5);
    float winding = backdrop;

    for (uint s = 0u; s < seg_count; s++) {
        vec2 p0, p1, p2;
        read_segment(seg_offset + s, p0, p1, p2);
        winding += quad_implicit_contribution(p0, p1, p2, sample_pos);
    }

    // ── Fill Rule Test ──
    float coverage = 0.0;
    if (fill_rule == FILL_RULE_EVENODD) {
        coverage = 1.0 - abs(mod(winding, 2.0) - 1.0);
    } else { // NONZERO
        coverage = clamp(abs(winding), 0.0, 1.0);
    }

    if (coverage <= 0.0) {
        discard;
    }

    vec4 paint = resolve_paint();
    fragColor = vec4(paint.rgb, paint.a * coverage);
}