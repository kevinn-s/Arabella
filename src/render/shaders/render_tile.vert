#version 300 es
precision highp float;
precision highp int;

#define TILE_WIDTH  4u
#define TILE_HEIGHT 4u

// Tile struct layout (36 bytes):
//   offset  0: x(u16), y(u16)           → attr 0 as 1 uint
//   offset  4: width(u8), height(u8), pad[2] → attr 1 as 1 uint
//   offset  8: backdrop[2] (2 × i32)    → attr 2 as ivec2
//   offset 16: segments[2] (2 × u32)    → attr 3 as uvec2
//   offset 24: payload(u32), paint_flag(u32), depth_index(u32) → attr 4 as uvec3

layout(location = 0) in uint  a_xy;
layout(location = 1) in uint  a_size;
layout(location = 2) in ivec2 a_backdrop;
layout(location = 3) in uvec2 a_segment;
layout(location = 4) in uvec3 a_misc;

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

flat out ivec2 v_backdrop;
flat out uvec2 v_segment;           // x = offset into segment_list, y = count
flat out uint  v_payload;
flat out uint  v_paint_flag;
flat out uvec2 v_tile_origin_pixels;
flat out uint  v_depth_index;
out vec2 v_local_xy;

void main() {
    uint tile_x_idx = a_xy & 0xFFFFu;
    uint tile_y_idx = a_xy >> 16u;

    uint tile_w = a_size & 0xFFu;
    uint tile_h = (a_size >> 8u) & 0xFFu;

    // Generate quad corner from gl_VertexID (TRIANGLE_STRIP)
    vec2 corner = vec2(float(gl_VertexID & 1), float((gl_VertexID >> 1) & 1));

    vec2 pixel_pos = vec2(
        float(tile_x_idx * TILE_WIDTH) + corner.x * float(tile_w),
        float(tile_y_idx * TILE_HEIGHT) + corner.y * float(tile_h)
    );

    vec2 ndc = vec2(
        (pixel_pos.x / float(u_width))  * 2.0 - 1.0,
        (pixel_pos.y / float(u_height)) * 2.0 - 1.0
    );
    if (u_negate_ndc != 0u) {
        ndc.y = -ndc.y;
    }

    float depth = 1.0 - (float(a_misc.z) / 10000.0) * 2.0;
    gl_Position = vec4(ndc, 0.0, 1.0);

    v_backdrop = a_backdrop;
    v_segment  = a_segment;  // x = offset (from f32::from_bits), y = count
    v_payload  = a_misc.x;
    v_paint_flag = a_misc.y;
    v_tile_origin_pixels = uvec2(
        tile_x_idx * TILE_WIDTH,
        tile_y_idx * TILE_HEIGHT
    );
    v_local_xy = pixel_pos;
    v_depth_index = a_misc.z;
}
