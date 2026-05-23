#version 300 es
precision highp float;
precision highp int;

// ============================================================================
// Instanced Loop-Blinn Vertex Shader
//
// Renders one control triangle per quadratic curve instance.
// 3 vertices per instance (gl_VertexID 0,1,2) → P0, P1, P2
// Loop-Blinn texture coordinates assigned per vertex:
//   P0 → (0, 0)
//   P1 → (0.5, 0)
//   P2 → (1, 1)
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

// Per-instance attributes
layout(location = 0) in uint a_curve_index;
layout(location = 1) in uint a_payload;
layout(location = 2) in uint a_paint_flag;
layout(location = 3) in uint a_depth_index;

// Outputs to fragment shader
out vec2 v_lb_uv;
flat out uint v_payload;
flat out uint v_paint_flag;
flat out vec2 v_p0;
flat out vec2 v_p1;
flat out vec2 v_p2;

// ============================================================================
// Helpers
// ============================================================================

ivec2 segments_idx_to_coord(uint idx) {
    return ivec2(
        int(idx & ((1u << u_segments_tex_width_bits) - 1u)),
        int(idx >> u_segments_tex_width_bits)
    );
}

void main() {
    // Fetch control points from segments texture
    uint base_texel = a_curve_index * 2u;
    vec4 texel0 = texelFetch(u_segments_texture, segments_idx_to_coord(base_texel), 0);
    vec4 texel1 = texelFetch(u_segments_texture, segments_idx_to_coord(base_texel + 1u), 0);

    vec2 p0 = texel0.xy;  // from
    vec2 p1 = texel0.zw;  // ctrl
    vec2 p2 = texel1.xy;  // to

    // Select position and Loop-Blinn UV based on vertex ID within the triangle
    vec2 pos;
    vec2 lb_uv;
    int vid = gl_VertexID % 3;

    if (vid == 0) {
        pos = p0;
        lb_uv = vec2(0.0, 0.0);
    } else if (vid == 1) {
        pos = p1;
        lb_uv = vec2(0.5, 0.0);
    } else {
        pos = p2;
        lb_uv = vec2(1.0, 1.0);
    }

    // Convert pixel position to NDC
    vec2 ndc = vec2(
        (pos.x / float(u_width))  * 2.0 - 1.0,
        (pos.y / float(u_height)) * 2.0 - 1.0
    );
    if (u_negate_ndc != 0u) {
        ndc.y = -ndc.y;
    }

    // Depth from instance
    float depth = 1.0 - (float(a_depth_index) / 10000.0) * 2.0;

    gl_Position = vec4(ndc, 0.0, 1.0);

    v_lb_uv = lb_uv;
    v_payload = a_payload;
    v_paint_flag = a_paint_flag;
    v_p0 = p0;
    v_p1 = p1;
    v_p2 = p2;
}
