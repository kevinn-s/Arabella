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
// Add this line to your Fragment Shader
flat in uint v_depth_index;
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


int get_row_backdrop(int local_y) {
    uint b = (local_y < 2) ? v_backdrop.x : v_backdrop.y;
    uint shift = ((local_y & 1) == 0) ? 0u : 16u;
    int winding = int((b >> shift) & 0xFFFFu);
    if ((winding & 0x8000) != 0) {
        winding |= ~0xFFFF;
    }
    return winding;
}

vec4 fetch_segment_data(uint index) {
    uint tex_width = 1u << u_segments_tex_width_bits;
    uint tex_x = index & (tex_width - 1u);
    uint tex_y = index >> u_segments_tex_width_bits;
    return texelFetch(segments_texture, ivec2(tex_x, tex_y), 0);
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

// ============================================================================
// Extended Implicit Geometry (Strict Bounding)
// ============================================================================
float quad_implicit_contribution(vec2 p0, vec2 p1, vec2 p2, vec2 pixel) {
    // --- 1. Direction of traversal ---
    float y_min, y_max, sign_v;
    if (p2.y > p0.y) {
        sign_v = 1.0; y_min = p0.y; y_max = p2.y;
    } else if (p2.y < p0.y) {
        sign_v = -1.0; y_min = p2.y; y_max = p0.y;
    } else {
        return 0.0;
    }

    // --- 2. Y-bounds (half-open to avoid double counting at segment joins) ---
    if (pixel.y < y_min || pixel.y >= y_max) {
        return 0.0;
    }

    // --- 3. Compute x on chord and on active control leg at pixel.y ---
    float t_chord = (pixel.y - p0.y) / (p2.y - p0.y);
    float x_chord = p0.x + t_chord * (p2.x - p0.x);

    float x_leg;
    bool on_first_leg = (sign_v > 0.0) ? (pixel.y < p1.y) : (pixel.y > p1.y);

    if (on_first_leg) {
        // Active leg: P0 → P1
        float t_leg = (pixel.y - p0.y) / (p1.y - p0.y);
        x_leg = p0.x + t_leg * (p1.x - p0.x);
    } else {
        // Active leg: P1 → P2
        if (abs(p2.y - p1.y) < 1e-6) {
            x_leg = p2.x;
        } else {
            float t_leg = (pixel.y - p1.y) / (p2.y - p1.y);
            x_leg = p1.x + t_leg * (p2.x - p1.x);
        }
    }

    // // --- 4. Strict triangle bounds: handle T2 ∪ T3 region ---
    float x_min_tri = min(x_chord, x_leg);
    float x_max_tri = max(x_chord, x_leg);

    if (pixel.x > x_max_tri) {
        return sign_v; // in T2 ∪ T3 → inside
    } else if (pixel.x < x_min_tri) {
        return 0.0;    // left of entire control triangle → outside
    }

    // --- 5. Inside T1: implicit Loop-Blinn test ---
    // Loop-Blinn texture coordinates:
    //   u = 0.5 * w1 + w2
    //   v = w2
    //   f(u, v) = u² - v
    //
    // P0 → (u=0, v=0) → f = 0    (on curve)
    // P1 → (u=0.5, v=0) → f = 0.25 (P1 is on the f > 0 side = bulge side)
    // P2 → (u=1, v=1) → f = 0    (on curve)
    //
    // f > 0 ⟺ pixel is on the BULGE side (P1 side) of the curve
    // f < 0 ⟺ pixel is on the CHORD side (wedge side) of the curve

    float denom = (p1.y - p2.y) * (p0.x - p2.x) + (p2.x - p1.x) * (p0.y - p2.y);
    if (abs(denom) < 1e-6) {
        // Degenerate triangle — treat as line, default to chord comparison
        return (pixel.x > x_chord) ? sign_v : 0.0;
    }

    float inv_denom = 1.0 / denom;
    float w0 = ((p1.y - p2.y) * (pixel.x - p2.x) + (p2.x - p1.x) * (pixel.y - p2.y)) * inv_denom;
    float w1 = ((p2.y - p0.y) * (pixel.x - p2.x) + (p0.x - p2.x) * (pixel.y - p2.y)) * inv_denom;
    float w2 = 1.0 - w0 - w1;

    float u = 0.5 * w1 + w2;
    float v = w2;
    float f = u * u - v;


    // --- 6. Choose the correct half of T1 based on bulge direction ---
    bool bulges_right = x_leg > x_chord;

    if (bulges_right) {
        // Right-bulge: WEDGE half (chord side, f < 0) is inside.
        return (f > 0.0) ? sign_v : 0.0;
    } else {
        // Left-bulge: BULGE-CHORD WEDGE (P1 side, f > 0) is inside.
        return (f < 0.0) ? sign_v : 0.0;
    }
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

 

    float coverage = 0.0;


    coverage /= 16.0;

    // if (coverage < 0.1) {
    //     discard;
    // }
    int local_y = int(v_local_xy.y - float(v_tile_origin_pixels.y));
  
    int row_winding_start = get_row_backdrop(local_y);

    float implicit_contrb = 0.0;
     for (uint s = 0u; s < seg_count; s++) {
                uint tex_base = (seg_offset + s) * 2u;
                vec4 tex0 = fetch_segment_data(tex_base);
                vec4 tex1 = fetch_segment_data(tex_base + 1u);
            
                implicit_contrb += quad_implicit_contribution(tex0.xy, tex0.zw, tex1.xy, v_local_xy); 
          
            }

    if(float(row_winding_start) / 256.0 > 0.0){
        if(implicit_contrb == 0.0) {
        fragColor = vec4(0.0,1.0,0.5,1.0);
        return;
        }
    }



    // Resolve Paint (Simplified for Solid)
    vec4 paint = unpack_rgba8(v_payload);
  
    // Apply coverage as alpha (Assuming premultiplied or simple blend)
    fragColor = paint * coverage;
}