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
// Each curve = 2 texels: texel0 = (from.x, from.y, ctrl.x, ctrl.y), texel1 = (to.x, to.y, 0, 0)
uniform sampler2D u_segments_texture;

// Texture 1: per-tile segment index list (R32UI)
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

int unpack_i16(int word, int half_idx) {
    int shifted = (word >> (half_idx * 16)) & 0xFFFF;
    if ((shifted & 0x8000) != 0) {
        shifted |= ~0xFFFF;
    }
    return shifted;
}

int get_row_backdrop(int local_y) {
    int word = (local_y < 2) ? v_backdrop.x : v_backdrop.y;
    int half_idx = local_y & 1;
    return unpack_i16(word, half_idx);
}

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

uint get_curve_index(uint list_offset, uint i) {
    uint idx = list_offset + i;
    return texelFetch(u_segment_list_texture, segment_list_idx_to_coord(idx), 0).r;
}

void read_curve(uint curve_idx, out vec2 p0, out vec2 p1, out vec2 p2) {
    uint base_texel = curve_idx * 2u;
    vec4 texel0 = texelFetch(u_segments_texture, segments_idx_to_coord(base_texel), 0);
    vec4 texel1 = texelFetch(u_segments_texture, segments_idx_to_coord(base_texel + 1u), 0);
    p0 = texel0.xy;
    p1 = texel0.zw;
    p2 = texel1.xy;
}

// ============================================================================
// GLLabel-style Root-Finding Curve Evaluation
// ============================================================================
//
// For each quadratic bezier, we ask: "does a rightward ray from the pixel
// cross this curve?" We solve for t where bezier_y(t) = pixel.y, then check
// if bezier_x(t) > pixel.x. Each valid crossing contributes +1 or -1 to the
// winding number based on the curve's Y-direction at that t.
//
// This approach:
//   - Handles non-monotonic curves (finds 0, 1, or 2 crossings per curve)
//   - No y-monotonization required on CPU
//   - No control-triangle / barycentric math
//   - Robust at curve extrema (both roots found naturally)
//
// Based on Dobbie's vector texture technique as used in GLLabel.

// Solve quadratic: finds t values where bezier component = 0.
// For bezier B(t) = (1-t)^2 * p0 + 2t(1-t) * p1 + t^2 * p2,
// B(t) - target = 0 expands to: (p0 - 2*p1 + p2)*t^2 + 2*(p1-p0)*t + (p0-target) = 0
//
// Returns number of valid roots in [0, 1). Results stored in `t`.
int solve_quadratic_bezier(float p0, float p1, float p2, float target, out vec2 t) {
    // Coefficients of the quadratic in t:
    //   a*t^2 + b*t + c = 0
    float c = p0 - target;
    float b = 2.0 * (p1 - p0);
    float a = p0 - 2.0 * p1 + p2;

    // Nearly-linear case (a ≈ 0): solve b*t + c = 0
    if (abs(a) < 1e-6) {
        if (abs(b) < 1e-6) {
            return 0; // Constant — no crossing
        }
        float root = -c / b;
        // Half-open interval [0, 1) to avoid double-counting at endpoints
        if (root >= 0.0 && root < 1.0) {
            t[0] = root;
            return 1;
        }
        return 0;
    }

    // Full quadratic: discriminant
    float disc = b * b - 4.0 * a * c;
    if (disc < 0.0) {
        return 0;
    }

    float sqrt_disc = sqrt(disc);
    float inv_2a = 0.5 / a;

    // Two candidate roots
    float r0 = (-b + sqrt_disc) * inv_2a;
    float r1 = (-b - sqrt_disc) * inv_2a;

    int count = 0;

    // Half-open interval [0, 1) — standard convention for winding rules.
    // This ensures that at segment joins (where one curve ends and the next
    // begins at the same point), the crossing is counted exactly once.
    if (r0 >= 0.0 && r0 < 1.0) {
        t[count] = r0;
        count++;
    }
    if (r1 >= 0.0 && r1 < 1.0) {
        t[count] = r1;
        count++;
    }

    return count;
}

// Evaluate bezier position at parameter t: B(t) = (1-t)^2*p0 + 2t(1-t)*p1 + t^2*p2
float bezier_at(float p0, float p1, float p2, float t) {
    float mt = 1.0 - t;
    return mt * mt * p0 + 2.0 * t * mt * p1 + t * t * p2;
}

// Evaluate bezier derivative at parameter t: B'(t) = 2(1-t)(p1-p0) + 2t(p2-p1)
float bezier_deriv_at(float p0, float p1, float p2, float t) {
    return 2.0 * (1.0 - t) * (p1 - p0) + 2.0 * t * (p2 - p1);
}

// Compute the winding contribution of one quadratic bezier at a pixel.
//
// Casts a rightward ray from pixel and counts signed crossings.
// For each t where bezier_y(t) = pixel.y AND bezier_x(t) > pixel.x,
// the crossing contributes +1 if the curve is going down at that t,
// or -1 if going up.
float quad_winding_contribution(vec2 p0, vec2 p1, vec2 p2, vec2 pixel) {
    // Quick Y-bounds rejection: if pixel.y is outside the curve's Y range,
    // no crossing is possible.
    float y_min = min(p0.y, min(p1.y, p2.y));
    float y_max = max(p0.y, max(p1.y, p2.y));

    if (pixel.y < y_min || pixel.y >= y_max) {
        return 0.0;
    }

    // Find all t where bezier_y(t) = pixel.y
    vec2 t_roots;
    int root_count = solve_quadratic_bezier(p0.y, p1.y, p2.y, pixel.y, t_roots);

    if (root_count == 0) {
        return 0.0;
    }

    float winding = 0.0;

    for (int i = 0; i < 2; i++) {
        if (i >= root_count) break;

        float t = t_roots[i];

        // Evaluate X at this t — is the crossing to the right of the pixel?
        float curve_x = bezier_at(p0.x, p1.x, p2.x, t);

        if (curve_x > pixel.x) {
            // The curve crosses our ray. Determine direction.
            float dy_dt = bezier_deriv_at(p0.y, p1.y, p2.y, t);

            // dy_dt > 0 means curve going down at crossing → winding +1
            // dy_dt < 0 means curve going up at crossing → winding -1
            if (dy_dt > 0.0) {
                winding += 1.0;
            } else if (dy_dt < 0.0) {
                winding -= 1.0;
            }
            // dy_dt == 0 means we're at a Y-extremum. The root was found
            // at the turning point. Since we use [0,1) interval, at most
            // one of the two adjacent monotonic halves will claim this point.
            // In practice dy_dt is rarely exactly 0 due to floating point,
            // but if it is, we skip it (contributes 0 net winding).
        }
    }

    return winding;
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

    // Start with backdrop winding for this row.
    // The backdrop represents the winding number accumulated from all curves
    // whose crossings occur strictly to the LEFT of this tile's left edge.
    float winding = float(get_row_backdrop(local_y));

    // Pixel center in world coordinates
    vec2 pixel_center = v_local_xy + vec2(0.5);

    // Accumulate winding from all curves assigned to this tile.
    // Each curve is evaluated via root-finding: solve bezier_y(t) = pixel.y,
    // then check if bezier_x(t) is to the right of the pixel (rightward ray test).
    for (uint s = 0u; s < seg_count; s++) {
        uint curve_idx = get_curve_index(seg_offset, s);

        vec2 p0, p1, p2;
        read_curve(curve_idx, p0, p1, p2);

        winding += quad_winding_contribution(p0, p1, p2, pixel_center);
    }

    // Apply fill rule to convert winding number → coverage
    float coverage;
    if (fill_rule == FILL_RULE_NONZERO) {
        coverage = clamp(abs(winding), 0.0, 1.0);
    } else {
        // Even-odd: coverage = 1 when winding is odd, 0 when even
        float w = abs(winding);
        coverage = 1.0 - abs(mod(w, 2.0) - 1.0);
    }

    if (coverage < 0.004) {
        discard;
    }

    // Resolve paint (solid color from payload)
    vec4 paint = unpack_rgba8(v_payload);

    // Premultiplied alpha output
    fragColor = paint * coverage;
}
