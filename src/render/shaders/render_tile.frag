#version 300 es
precision highp float;
precision highp int;
precision highp usampler2D;

#define TILE_WIDTH  4u
#define TILE_HEIGHT 4u

#define COLOR_SOURCE_PAYLOAD 0u
#define COLOR_SOURCE_PAINT   1u
#define PAINT_TYPE_SOLID     0u

#define COLOR_SOURCE_SHIFT 30u
#define PAINT_TYPE_SHIFT   27u
#define FILL_RULE_SHIFT    24u
#define COLOR_SOURCE_MASK  0xC0000000u
#define PAINT_TYPE_MASK    0x38000000u
#define FILL_RULE_MASK     0x07000000u

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
    uint u_segment_list_tex_width_bits;
    uint u_encoded_paints_tex_width_bits;
    uint u_negate_ndc;
    uint _pad0;
};

// Texture 0: curve control points (RGBA32F)
// Each texel = [from.x, from.y, ctrl.x, ctrl.y] or [to.x, to.y, 0, 0]
// 2 texels per segment: texel0 = (from, ctrl), texel1 = (to, 0)
uniform sampler2D u_segments_texture;

// Texture 1: per-tile segment index list (R32UI)
// Linear array of u32 curve indices
uniform usampler2D u_segment_list_texture;

// ============================================================================
// Varyings from vertex shader
// ============================================================================

flat in ivec2 v_backdrop;
flat in uvec2 v_segment;           // x = offset, y = count
flat in uint  v_payload;
flat in uint  v_paint_flag;
flat in uvec2 v_tile_origin_pixels;
flat in uint  v_depth_index;
in vec2 v_local_xy;

out vec4 fragColor;

// ============================================================================
// Helpers
// ============================================================================

/// Unpack i16 from packed i32 (two i16s per i32)
int unpack_i16(int word, int half_idx) {
    int shifted = (word >> (half_idx * 16)) & 0xFFFF;
    // Sign extend
    if ((shifted & 0x8000) != 0) {
        shifted |= ~0xFFFF;
    }
    return shifted;
}

/// Get backdrop winding for a pixel row (0-3) within the tile
int get_row_backdrop(int local_y) {
    int word = (local_y < 2) ? v_backdrop.x : v_backdrop.y;
    int half_idx = local_y & 1;
    return unpack_i16(word, half_idx);
}

/// Convert linear index to 2D texture coordinate
ivec2 segments_idx_to_coord(uint idx) {
    return ivec2(
        int(idx & ((1u << u_segments_tex_width_bits) - 1u)),
        int(idx >> u_segments_tex_width_bits)
    );
}

ivec2 segment_list_idx_to_coord(uint idx) {
    return ivec2(
        int(idx & ((1u << u_segment_list_tex_width_bits) - 1u)),
        int(idx >> u_segment_list_tex_width_bits)
    );
}

/// Fetch a curve index from the segment list texture
uint get_curve_index(uint list_offset, uint i) {
    uint idx = list_offset + i;
    return texelFetch(u_segment_list_texture, segment_list_idx_to_coord(idx), 0).r;
}

/// Read curve control points from segments texture
/// Each curve occupies 2 RGBA32F texels:
///   texel 0: (from.x, from.y, ctrl.x, ctrl.y)
///   texel 1: (to.x, to.y, 0, 0)
void read_curve(uint curve_idx, out vec2 p0, out vec2 p1, out vec2 p2) {
    uint base_texel = curve_idx * 2u;
    vec4 texel0 = texelFetch(u_segments_texture, segments_idx_to_coord(base_texel), 0);
    vec4 texel1 = texelFetch(u_segments_texture, segments_idx_to_coord(base_texel + 1u), 0);
    p0 = texel0.xy;  // from
    p1 = texel0.zw;  // ctrl
    p2 = texel1.xy;  // to
}

// ============================================================================
// Loop-Blinn Implicit Curve Evaluation
// ============================================================================

float quad_implicit_contribution(vec2 p0, vec2 p1, vec2 p2, vec2 pixel) {
    // Direction of traversal
    float y_min, y_max, sign_v;
    if (p2.y > p0.y) {
        sign_v = 1.0; y_min = p0.y; y_max = p2.y;
    } else if (p2.y < p0.y) {
        sign_v = -1.0; y_min = p2.y; y_max = p0.y;
    } else {
        return 0.0;
    }

    // Y-bounds (half-open interval)
    if (pixel.y < y_min || pixel.y >= y_max) {
        return 0.0;
    }

    // X on chord at pixel.y
    float t_chord = (pixel.y - p0.y) / (p2.y - p0.y);
    float x_chord = p0.x + t_chord * (p2.x - p0.x);

    // X on active control leg at pixel.y
    float x_leg;
    bool on_first_leg = (sign_v > 0.0) ? (pixel.y < p1.y) : (pixel.y > p1.y);

    if (on_first_leg) {
        float denom = p1.y - p0.y;
        if (abs(denom) < 1e-6) {
            x_leg = p0.x;
        } else {
            float t_leg = (pixel.y - p0.y) / denom;
            x_leg = p0.x + t_leg * (p1.x - p0.x);
        }
    } else {
        float denom = p2.y - p1.y;
        if (abs(denom) < 1e-6) {
            x_leg = p2.x;
        } else {
            float t_leg = (pixel.y - p1.y) / denom;
            x_leg = p1.x + t_leg * (p2.x - p1.x);
        }
    }

    // Strict triangle bounds
    float x_min_tri = min(x_chord, x_leg);
    float x_max_tri = max(x_chord, x_leg);

    if (pixel.x > x_max_tri) {
        return sign_v;
    } else if (pixel.x < x_min_tri) {
        return 0.0;
    }

    // Inside control triangle: Loop-Blinn implicit test
    float denom = (p1.y - p2.y) * (p0.x - p2.x) + (p2.x - p1.x) * (p0.y - p2.y);
    if (abs(denom) < 1e-6) {
        return (pixel.x > x_chord) ? sign_v : 0.0;
    }

    float inv_denom = 1.0 / denom;
    float w0 = ((p1.y - p2.y) * (pixel.x - p2.x) + (p2.x - p1.x) * (pixel.y - p2.y)) * inv_denom;
    float w1 = ((p2.y - p0.y) * (pixel.x - p2.x) + (p0.x - p2.x) * (pixel.y - p2.y)) * inv_denom;
    float w2 = 1.0 - w0 - w1;

    float u = 0.5 * w1 + w2;
    float v = w2;
    float f = u * u - v;

    bool bulges_right = x_leg > x_chord;

    if (bulges_right) {
        return (f > 0.0) ? sign_v : 0.0;
    } else {
        return (f < 0.0) ? sign_v : 0.0;
    }
}

// ============================================================================
// Paint
// ============================================================================

vec4 unpack_rgba8(uint packed) {
    float r = float((packed >>  0u) & 0xFFu) / 255.0;
    float g = float((packed >>  8u) & 0xFFu) / 255.0;
    float b = float((packed >> 16u) & 0xFFu) / 255.0;
    float a = float((packed >> 24u) & 0xFFu) / 255.0;
    return vec4(r, g, b, a);
}

// ============================================================================
// Main
// ============================================================================

void main() {
    uint fill_rule = (v_paint_flag & FILL_RULE_MASK) >> FILL_RULE_SHIFT;
    uint seg_offset = v_segment.x;
    uint seg_count  = v_segment.y;

    // Pixel row within tile (0-3)
    int local_y = clamp(int(v_local_xy.y) - int(v_tile_origin_pixels.y), 0, 3);

    // Start with backdrop winding for this row
    float winding = float(get_row_backdrop(local_y));

    // Pixel center
    vec2 pixel_center = v_local_xy + vec2(0.5);

    // Accumulate winding from all curves in this tile
    for (uint s = 0u; s < seg_count; s++) {
        // Look up which curve this tile references
        uint curve_idx = get_curve_index(seg_offset, s);

        // Fetch curve control points
        vec2 p0, p1, p2;
        read_curve(curve_idx, p0, p1, p2);

        // Evaluate Loop-Blinn implicit
        winding += quad_implicit_contribution(p0, p1, p2, pixel_center);
    }

    // Apply fill rule
    float coverage;
    if (fill_rule == FILL_RULE_NONZERO) {
        coverage = clamp(abs(winding), 0.0, 1.0);
    } else {
        // Even-odd
        coverage = abs(mod(winding, 2.0));
        if (coverage > 1.0) coverage = 2.0 - coverage;
    }

    if (coverage < 0.004) {
        discard;
    }

    // Resolve paint (solid color from payload)
    vec4 paint = unpack_rgba8(v_payload);

    // Premultiplied alpha output
    fragColor = paint * coverage;
}
