
#[repr(C)]
#[derive(Debug, Clone, Copy)]
// - x: texel position at x-axis in atlas buffer
// - y: texel position at y-axis in atlas buffer
// - direction: determine to either increment or decrement in texel
pub struct Edges {
    pub x: u16,
    pub y: u16,
    pub direction: u8,

}
// `flag` bit layout:
//   - Bit  31:    `RECT_STRIP_FLAG`  0 = normal strip, 1 = rect strip
//   - Bits 29-30: `color_source`     0 = use payload, 1 = use slot texture, 2 = blend mode
//   - Bits 0-28:  Usage depends on color_source:
//
//     When color_source = 0 (COLOR_SOURCE_PAYLOAD):
//       - Bits 26-28: `paint_type` (0 = solid, 1 = image, 2 = linear_gradient, 3 = radial_gradient, 4 = sweep_gradient)
//       - Bits 0-25:
//         - If paint_type = 0: unused
//         - If paint_type >= 1: `paint_texture_idx`
//
//     When color_source = 1 (COLOR_SOURCE_SLOT):
//       - Bits 0-7: opacity (0-255)
//       - Bits 8-28: unused
//
//     When color_source = 2 (COLOR_SOURCE_BLEND):
//       - Bits 16-28: `dest_slot` (14 bits)
//       - Bits 8-15: `mix_mode` (8 bits)
//       - Bits 0-7: `compose_mode` (8 bits)
// Decision tree for paint/payload interpretation:
//
// color_source = 0 (COLOR_SOURCE_PAYLOAD) - Use payload data directly
// ├── paint_type = 0 (PAINT_TYPE_SOLID) - Solid color rendering
// │   └── payload = [r, g, b, a] RGBA (packed as u8s)
// │
// ├── paint_type = 1 (PAINT_TYPE_IMAGE) - Image rendering
// │   └── payload = packed image parameters
// │
// ├── paint_type = 2 (PAINT_TYPE_LINEAR_GRADIENT) - Linear gradient rendering
// ├── paint_type = 3 (PAINT_TYPE_RADIAL_GRADIENT) - Radial gradient (with kind discriminator)
// └── paint_type = 4 (PAINT_TYPE_SWEEP_GRADIENT) - Sweep gradient rendering
//     ├── payload = [x, y] scene coordinates (packed as u16s)
//     └── bits 0-25 = paint_texture_idx
//
// color_source = 1 (COLOR_SOURCE_SLOT) - Use slot texture
// ├── payload = slot_index (u32)
// └── bits 0-7 = opacity (0-255, where 255 = fully opaque)
//
// color_source = 2 (COLOR_SOURCE_BLEND) - Blend two slots
// ├── payload = [src_slot, dest_slot] slot indices (packed as u16s)
// │   ├── bits 0-15 = src_slot (source slot to blend)
// │   └── bits 16-31 = dest_slot (destination slot to blend with)
// └── paint bits 0-23:
//     ├── bits 16-23 = opacity (0-255, applied to blend result)
//     ├── bits 8-15 = mix_mode (blend mixing mode)
//     └── bits 0-7 = compose_mode (compositing operation)
pub struct Bands {
    pub width: u32,
    pub flag: u32,
    pub fill_rule: bool,
    pub payload: u32,
    pub backdrop: u16,
}

pub(crate) enum LoadOp {
    Load,
    Clear,
}