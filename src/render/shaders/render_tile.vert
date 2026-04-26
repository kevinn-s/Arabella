    #version 300 es
    precision highp float;
    precision highp int;

    // ============================================================================
    // Constants — must match Rust side
    // ============================================================================

    #define TILE_WIDTH  4u
    #define TILE_HEIGHT 4u

    // Color source modes
    #define COLOR_SOURCE_PAYLOAD 0u
    #define COLOR_SOURCE_PAINT   1u

    // Paint types
    #define PAINT_TYPE_SOLID            0u
    #define PAINT_TYPE_LINEAR_GRADIENT  1u
    #define PAINT_TYPE_RADIAL_GRADIENT  2u
    #define PAINT_TYPE_SWEEP_GRADIENT   3u
    #define PAINT_TYPE_IMAGE            4u

    // Paint flag bit layout (matches Rust encode_paint):
    //   bits 30-31: color source  (COLOR_SOURCE_*)
    //   bits 27-29: paint type    (PAINT_TYPE_*)
    //   bits 24-26: fill rule     (0 = nonzero, 1 = even-odd)
    //   bits  0-23: payload index (e.g. into encoded_paints texture)
    #define COLOR_SOURCE_SHIFT 30u
    #define PAINT_TYPE_SHIFT   27u
    #define FILL_RULE_SHIFT    24u
    #define COLOR_SOURCE_MASK  0xC0000000u
    #define PAINT_TYPE_MASK    0x38000000u
    #define FILL_RULE_MASK     0x07000000u
    #define PAINT_INDEX_MASK   0x00FFFFFFu

    // ============================================================================
    // Vertex attributes (per-instance, one Tile per instance)
    // ============================================================================

    // Tile struct layout (40 bytes, stride 40):
    //   x, y         : u16 u16        (4 bytes)  → attribute 0 as 1 u32
    //   width, h     : u8 u8 + 2 pad  (4 bytes)  → attribute 1 as 1 u32
    //   backdrop[2]  : u32 u32        (8 bytes)  → attribute 2 as uvec2
    //   segment[2]   : u32 u32        (8 bytes)  → attribute 3 as uvec2
    //   payload      : u32            (4 bytes)
    //   paint_flag   : u32            (4 bytes)
    //   depth_index  : u32            (4 bytes)  → attribute 4 as uvec3
    layout(location = 0) in uint  a_xy;
    layout(location = 1) in uint  a_size;
    layout(location = 2) in uvec2 a_backdrop;
    layout(location = 3) in uvec2 a_segment;
    layout(location = 4) in uvec3 a_misc;        // payload, paint_flag, depth_index

    // ============================================================================
    // Uniform block
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

    // ============================================================================
    // Varyings to fragment shader
    // ============================================================================

    flat out uvec2 v_backdrop;          // packed 4x i16 winding
    flat out uvec2 v_segment;            // [count, offset]
    flat out uint  v_payload;            // packed RGBA8 (solid color) or paint index
    flat out uint  v_paint_flag;         // source/type/fill_rule packed
    flat out uvec2 v_tile_origin_pixels; // top-left pixel of this tile
    out vec2 v_local_xy;                 // pixel position within the screen

    void main() {
        // ── Unpack tile attributes ──
        uint tile_x_idx = a_xy & 0xFFFFu;
        uint tile_y_idx = a_xy >> 16u;

        uint tile_w = a_size & 0xFFu;
        uint tile_h = (a_size >> 8u) & 0xFFu;

        // ── Generate quad corner from gl_VertexID for TRIANGLE_STRIP ──
        // VertexID: 0 → (0,0), 1 → (1,0), 2 → (0,1), 3 → (1,1)
        vec2 corner = vec2(float(gl_VertexID & 1), float((gl_VertexID >> 1) & 1));

        // ── Compute screen pixel position for this corner ──
        vec2 pixel_pos = vec2(
            float(tile_x_idx) * float(TILE_WIDTH)  + corner.x * float(tile_w),
            float(tile_y_idx) * float(TILE_HEIGHT) + corner.y * float(tile_h)
        );

        // ── Convert pixel position to NDC ──
        vec2 ndc = vec2(
            (pixel_pos.x / float(u_width))  * 2.0 - 1.0,
            (pixel_pos.y / float(u_height)) * 2.0 - 1.0
        );
        if (u_negate_ndc != 0u) {
            ndc.y = -ndc.y;
        }

        // ── Depth for painter's algorithm ──
        // Lower depth_index = drawn first = behind.
        // GL depth is [-1, +1], with +1 = farthest. Map small index to high depth.
        // Use a generous range; assume up to ~4M paths.
        float depth = 1.0 - (float(a_misc.z) + 1.0) / 16777216.0;

        gl_Position = vec4(ndc, depth, 1.0);

        // ── Pass to fragment shader ──
        v_backdrop = a_backdrop;
        v_segment  = a_segment;
        v_payload  = a_misc.x;
        v_paint_flag = a_misc.y;
        v_tile_origin_pixels = uvec2(
            tile_x_idx * TILE_WIDTH,
            tile_y_idx * TILE_HEIGHT
        );
        v_local_xy = pixel_pos;
    }
