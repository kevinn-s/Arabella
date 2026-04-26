#version 300 es
precision highp float;
precision highp int;

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

uniform highp sampler2D segments_texture; 

// ============================================================================
// Varyings
// ============================================================================
flat in uvec2 v_backdrop;
flat in uvec2 v_segment;
flat in uint  v_payload;
flat in uint  v_paint_flag;
flat in uvec2 v_tile_origin_pixels;
in vec2 v_local_xy;

out vec4 fragColor;

#define FILL_RULE_SHIFT    24u
#define FILL_RULE_MASK     0x07000000u

// ============================================================================
// Helper Functions
// ============================================================================
vec4 decode_solid_color(uint payload) {
    return vec4(
        float(payload & 0xFFu),
        float((payload >> 8u) & 0xFFu),
        float((payload >> 16u) & 0xFFu),
        float((payload >> 24u) & 0xFFu)
    ) / 255.0;
}

vec4 fetch_segment_data(uint index) {
    uint tex_width = 1u << u_segments_tex_width_bits;
    uint tex_x = index & (tex_width - 1u);
    uint tex_y = index >> u_segments_tex_width_bits;
    return texelFetch(segments_texture, ivec2(tex_x, tex_y), 0);
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

// ============================================================================
// Quadratic Bezier Raycaster
// ============================================================================
float calculate_quad_winding(vec2 sample_pos, vec2 p0, vec2 p1, vec2 p2) {
    float y = sample_pos.y;
    float winding = 0.0;
    
    // Fast vertical bounds reject
    float min_y = min(p0.y, min(p1.y, p2.y));
    float max_y = max(p0.y, max(p1.y, p2.y));
    if (y < min_y || y >= max_y) return 0.0;
    
    // Solve the bezier quadratic equation: a*t^2 + b*t + c = 0
    float a = p0.y - 2.0 * p1.y + p2.y;
    float b = 2.0 * (p1.y - p0.y);
    float c = p0.y - y;
    
    if (abs(a) < 1e-5) {
        // Linear fallback
        if (abs(b) > 1e-5) {
            float t = -c / b;
            if (t >= 0.0 && t < 1.0) {
                float x = mix(p0.x, p2.x, t);
                if (x <= sample_pos.x) winding += sign(p2.y - p0.y);
            }
        }
    } else {
        // Quadratic roots
        float discriminant = b * b - 4.0 * a * c;
        if (discriminant >= 0.0) {
            float sqrt_d = sqrt(discriminant);
            
            // Root 1
            float t1 = (-b - sqrt_d) / (2.0 * a);
            if (t1 >= 0.0 && t1 < 1.0) {
                float inv_t = 1.0 - t1;
                float x = inv_t * inv_t * p0.x + 2.0 * inv_t * t1 * p1.x + t1 * t1 * p2.x;
                if (x <= sample_pos.x) {
                    float dy = 2.0 * inv_t * (p1.y - p0.y) + 2.0 * t1 * (p2.y - p1.y);
                    winding += sign(dy);
                }
            }
            
            // Root 2
            float t2 = (-b + sqrt_d) / (2.0 * a);
            if (t2 >= 0.0 && t2 < 1.0) {
                float inv_t = 1.0 - t2;
                float x = inv_t * inv_t * p0.x + 2.0 * inv_t * t2 * p1.x + t2 * t2 * p2.x;
                if (x <= sample_pos.x) {
                    float dy = 2.0 * inv_t * (p1.y - p0.y) + 2.0 * t2 * (p2.y - p1.y);
                    winding += sign(dy);
                }
            }
        }
    }
    return winding;
}

// ============================================================================
// Main
// ============================================================================
void main() {
    vec4 base_color = decode_solid_color(v_payload);
    uint segment_count = v_segment.x;
    uint segment_offset = v_segment.y;
    uint fill_rule = (v_paint_flag & FILL_RULE_MASK) >> FILL_RULE_SHIFT;

    int local_y = int(v_local_xy.y - float(v_tile_origin_pixels.y));
    local_y = clamp(local_y, 0, 3);
    int row_backdrop = get_row_backdrop(local_y);

    if (segment_count == 0u) {
        bool filled = (fill_rule == 0u) ? (abs(row_backdrop) > 0) : ((abs(row_backdrop) % 2) != 0);
        if (filled) fragColor = base_color;
        else discard;
        return;
    }

    // 16x MSAA Loop
    float total_coverage = 0.0;
    
    for (int sx = 0; sx < 4; sx++) {
        for (int sy = 0; sy < 4; sy++) {
            vec2 offset = vec2((float(sx) - 1.5) * 0.25, (float(sy) - 1.5) * 0.25);
            vec2 sample_pos = v_local_xy + offset;
            float sample_winding = float(row_backdrop);

            for (uint i = 0u; i < segment_count; ++i) {
                // FIXED: Multiply by 2u to correctly read both texels for this quadratic segment!
                uint tex_base = (segment_offset + i) * 2u;
                vec4 tex0 = fetch_segment_data(tex_base);
                vec4 tex1 = fetch_segment_data(tex_base + 1u);
                
                vec2 p0 = tex0.xy;
                vec2 p1 = tex0.zw;
                vec2 p2 = tex1.xy;

                sample_winding += calculate_quad_winding(sample_pos, p0, p1, p2);
            }

            bool sample_filled = (fill_rule == 0u) ? (abs(sample_winding) > 0.01) : ((int(abs(sample_winding)) % 2) != 0);
            if (sample_filled) total_coverage += 1.0;
        }
    }
    
    total_coverage /= 16.0;

    if (total_coverage < 0.001) discard;
    fragColor = vec4(base_color.rgb, base_color.a * total_coverage);
}