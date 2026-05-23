#version 300 es
precision highp float;
precision highp int;

// ============================================================================
// Instanced Loop-Blinn Fragment Shader
//
// Evaluates the quadratic implicit f = u² - v to determine if a pixel
// is inside or outside the curve boundary within the control triangle.
//
// The sign convention:
//   f > 0 → pixel is on the bulge side (P1 side) of the curve
//   f < 0 → pixel is on the chord side (wedge side) of the curve
//   f = 0 → pixel is exactly on the curve
//
// Whether "inside" means f > 0 or f < 0 depends on whether the curve
// bulges to the right or left relative to the chord direction (P0→P2).
// ============================================================================

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
// Varyings from vertex shader
// ============================================================================

in vec2 v_lb_uv;
flat in uint v_payload;
flat in uint v_paint_flag;
flat in vec2 v_p0;
flat in vec2 v_p1;
flat in vec2 v_p2;

out vec4 fragColor;

// ============================================================================
// Helpers
// ============================================================================

vec4 unpack_rgba8(uint packed) {
    float r = float((packed >>  0u) & 0xFFu) / 255.0;
    float g = float((packed >>  8u) & 0xFFu) / 255.0;
    float b = float((packed >> 16u) & 0xFFu) / 255.0;
    float a = float((packed >> 24u) & 0xFFu) / 255.0;
    return vec4(r, g, b, a);
}

// Determine if the curve bulges to the right of the chord (P0→P2).
// Uses the cross product of chord direction with (P1 - P0).
bool curve_bulges_right(vec2 p0, vec2 p1, vec2 p2) {
    vec2 chord = p2 - p0;
    vec2 to_ctrl = p1 - p0;
    // Cross product: positive means P1 is to the right of chord
    float cross_val = chord.x * to_ctrl.y - chord.y * to_ctrl.x;
    return cross_val < 0.0;
}

// ============================================================================
// Main
// ============================================================================

void main() {
    // Evaluate Loop-Blinn implicit function
    float u = v_lb_uv.x;
    float v = v_lb_uv.y;
    float f = u * u - v;

    // Determine curve orientation to decide discard condition
    bool bulges_right = curve_bulges_right(v_p0, v_p1, v_p2);

    // Antialiasing: use screen-space derivatives of f for smooth edges
    float fw = fwidth(f);
    float alpha;

    if (bulges_right) {
        // Curve bulges right: pixels on the chord side (f < 0) are inside the fill.
        // Discard pixels on the bulge side (f > 0).
        alpha = 1.0 - smoothstep(-fw, fw, f);
    } else {
        // Curve bulges left: pixels on the bulge side (f > 0) are inside the fill.
        // Discard pixels on the chord side (f < 0).
        alpha = smoothstep(-fw, fw, f);
    }

    if (alpha < 0.004) {
        discard;
    }

    // Resolve paint color
    vec4 paint = unpack_rgba8(v_payload);

    // Output with coverage-based alpha
    fragColor = paint * alpha;
}
