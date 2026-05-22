#![expect(
    clippy::cast_possible_truncation,
    reason = "We temporarily ignore those because the casts\
only break in edge cases, and some of them are also only related to conversions from f64 to f32."
)]

use crate::render::common::render_tile;
use crate::{
    RenderError, RenderSize,
    render::{Config},
    scene::{RenderSettings, Scene},
    tile::{Tile,TileMap}
};

use alloc::vec;
use alloc::vec::Vec;
use bytemuck::{Pod, Zeroable};
use web_sys::wasm_bindgen::{JsCast, JsValue};
use web_sys::{
    HtmlCanvasElement, WebGl2RenderingContext, WebGlBuffer, WebGlProgram,
    WebGlTexture, WebGlUniformLocation, WebGlVertexArrayObject,
};

// ============================================================================
// WebGlPrograms
// ============================================================================

#[derive(Debug)]
struct WebGlPrograms {
    tile_program: WebGlProgram,
    tile_uniforms: TileUniforms,
    resources: WebGlResources,
}

impl WebGlPrograms {
    fn new(gl: &WebGl2RenderingContext) -> Self {
        let tile_program = create_shader_program(
            gl,
            render_tile::VERTEX_SOURCE,
            render_tile::FRAGMENT_SOURCE,
        );
        let tile_uniforms = get_tile_uniforms(gl, &tile_program);
        let resources = create_webgl_resources(gl);
        initialize_tile_vao(gl, &resources);

        Self {
            tile_program,
            tile_uniforms,
            resources,
        }
    }

    /// Upload all GPU data from the scene.
    fn prepare(&mut self, gl: &WebGl2RenderingContext, scene: &Scene) {
        let max_w = self.resources.max_texture_dimension_2d;

        // Upload segments texture (Texture 0: curve control points)
        self.upload_segments_texture(gl, scene.segments(), max_w);

        // Upload segment list texture (Texture 1: per-tile segment index lists)
        self.upload_segment_list_texture(gl, scene.segment_list(), max_w);
    }

    fn upload_segments_texture(
        &mut self,
        gl: &WebGl2RenderingContext,
        segments: &[f32],
        max_w: u32,
    ) {
        if segments.is_empty() {
            return;
        }

        // Each segment is 4 floats (ctrl.x, ctrl.y, end.x, end.y) = 1 RGBA texel
        let texel_count = segments.len() / 4;
        let required_height = (texel_count as u32).div_ceil(max_w).max(1);

        if required_height > self.resources.segments_texture_height {
            self.resources.segments_texture_height = required_height;
        }

        // Pad to full texture row width
        let total_floats = (max_w * self.resources.segments_texture_height * 4) as usize;
        let mut texture_data = vec![0.0_f32; total_floats];
        texture_data[..segments.len()].copy_from_slice(segments);

        gl.active_texture(WebGl2RenderingContext::TEXTURE0);
        gl.bind_texture(
            WebGl2RenderingContext::TEXTURE_2D,
            Some(&self.resources.segments_texture),
        );
        upload_data_to_rgba32f_texture(
            gl,
            &texture_data,
            max_w,
            self.resources.segments_texture_height,
        );
    }

    fn upload_segment_list_texture(
        &mut self,
        gl: &WebGl2RenderingContext,
        segment_list: &[u32],
        max_w: u32,
    ) {
        if segment_list.is_empty() {
            return;
        }

        // Each entry is 1 u32 = 1 R32UI texel
        let texel_count = segment_list.len() as u32;
        let required_height = texel_count.div_ceil(max_w).max(1);

        if required_height > self.resources.segment_list_texture_height {
            self.resources.segment_list_texture_height = required_height;
        }

        // Pad to full texture row width
        let total_u32s = (max_w * self.resources.segment_list_texture_height) as usize;
        let mut texture_data = vec![0_u32; total_u32s];
        texture_data[..segment_list.len()].copy_from_slice(segment_list);

        gl.active_texture(WebGl2RenderingContext::TEXTURE1);
        gl.bind_texture(
            WebGl2RenderingContext::TEXTURE_2D,
            Some(&self.resources.segment_list_texture),
        );
        upload_data_to_r32ui_texture(
            gl,
            &texture_data,
            max_w,
            self.resources.segment_list_texture_height,
        );
    }
}

// ============================================================================
// WebGlResources
// ============================================================================

#[derive(Debug)]
struct WebGlResources {
    tile_vao: WebGlVertexArrayObject,
    tiles_buffer: WebGlBuffer,
    /// Texture 0: curve control points (RGBA32F)
    segments_texture: WebGlTexture,
    segments_texture_height: u32,
    /// Texture 1: per-tile segment index list (R32UI)
    segment_list_texture: WebGlTexture,
    segment_list_texture_height: u32,
    /// Texture 2: encoded paints (RGBA32UI)
    encoded_paints_texture: WebGlTexture,
    encoded_paints_texture_height: u32,
    /// Texture 3: gradient LUT (RGBA8, LINEAR filtering)
    gradient_texture: WebGlTexture,
    gradient_texture_height: u32,
    max_texture_dimension_2d: u32,
    config_buffer: WebGlBuffer,
}

fn create_webgl_resources(gl: &WebGl2RenderingContext) -> WebGlResources {
    let tile_vao = gl.create_vertex_array().unwrap();
    let tiles_buffer = gl.create_buffer().unwrap();

    // Texture 0: segments (RGBA32F, NEAREST)
    let segments_texture = gl.create_texture().unwrap();
    {
        gl.active_texture(WebGl2RenderingContext::TEXTURE0);
        gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&segments_texture));
        set_nearest_clamp(gl);
    }

    // Texture 1: segment list (R32UI, NEAREST)
    let segment_list_texture = gl.create_texture().unwrap();
    {
        gl.active_texture(WebGl2RenderingContext::TEXTURE1);
        gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&segment_list_texture));
        set_nearest_clamp(gl);
    }

    // Texture 2: encoded paints (RGBA32UI, NEAREST)
    let encoded_paints_texture = gl.create_texture().unwrap();
    {
        gl.active_texture(WebGl2RenderingContext::TEXTURE2);
        gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&encoded_paints_texture));
        set_nearest_clamp(gl);
    }

    // Texture 3: gradient LUT (LINEAR)
    let gradient_texture = gl.create_texture().unwrap();
    {
        gl.active_texture(WebGl2RenderingContext::TEXTURE3);
        gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&gradient_texture));
        gl.tex_parameteri(WebGl2RenderingContext::TEXTURE_2D, WebGl2RenderingContext::TEXTURE_MIN_FILTER, WebGl2RenderingContext::LINEAR as i32);
        gl.tex_parameteri(WebGl2RenderingContext::TEXTURE_2D, WebGl2RenderingContext::TEXTURE_MAG_FILTER, WebGl2RenderingContext::LINEAR as i32);
        gl.tex_parameteri(WebGl2RenderingContext::TEXTURE_2D, WebGl2RenderingContext::TEXTURE_WRAP_S, WebGl2RenderingContext::CLAMP_TO_EDGE as i32);
        gl.tex_parameteri(WebGl2RenderingContext::TEXTURE_2D, WebGl2RenderingContext::TEXTURE_WRAP_T, WebGl2RenderingContext::CLAMP_TO_EDGE as i32);
    }

    let max_texture_dimension_2d = gl
        .get_parameter(WebGl2RenderingContext::MAX_TEXTURE_SIZE)
        .unwrap()
        .as_f64()
        .unwrap() as u32;

    let config_buffer = gl.create_buffer().unwrap();

    WebGlResources {
        tile_vao,
        tiles_buffer,
        segments_texture,
        segments_texture_height: 1,
        segment_list_texture,
        segment_list_texture_height: 1,
        encoded_paints_texture,
        encoded_paints_texture_height: 0,
        gradient_texture,
        gradient_texture_height: 0,
        max_texture_dimension_2d,
        config_buffer,
    }
}

fn set_nearest_clamp(gl: &WebGl2RenderingContext) {
    gl.tex_parameteri(WebGl2RenderingContext::TEXTURE_2D, WebGl2RenderingContext::TEXTURE_MIN_FILTER, WebGl2RenderingContext::NEAREST as i32);
    gl.tex_parameteri(WebGl2RenderingContext::TEXTURE_2D, WebGl2RenderingContext::TEXTURE_MAG_FILTER, WebGl2RenderingContext::NEAREST as i32);
    gl.tex_parameteri(WebGl2RenderingContext::TEXTURE_2D, WebGl2RenderingContext::TEXTURE_WRAP_S, WebGl2RenderingContext::CLAMP_TO_EDGE as i32);
    gl.tex_parameteri(WebGl2RenderingContext::TEXTURE_2D, WebGl2RenderingContext::TEXTURE_WRAP_T, WebGl2RenderingContext::CLAMP_TO_EDGE as i32);
}

// ============================================================================
// TileUniforms
// ============================================================================

#[derive(Debug)]
struct TileUniforms {
    config_block_index: u32,
    segments_texture: WebGlUniformLocation,
    segment_list_texture: WebGlUniformLocation,
    encoded_paints_texture: Option<WebGlUniformLocation>,
    gradient_texture: Option<WebGlUniformLocation>,
}

fn get_tile_uniforms(gl: &WebGl2RenderingContext, program: &WebGlProgram) -> TileUniforms {
    let config_block_index = gl.get_uniform_block_index(program, "config");
    debug_assert_ne!(config_block_index, WebGl2RenderingContext::INVALID_INDEX);
    gl.uniform_block_binding(program, config_block_index, 0);

    TileUniforms {
        config_block_index,
        segments_texture: gl.get_uniform_location(program, "u_segments_texture").unwrap(),
        segment_list_texture: gl.get_uniform_location(program, "u_segment_list_texture").unwrap(),
        encoded_paints_texture: gl.get_uniform_location(program, "u_encoded_paints_texture"),
        gradient_texture: gl.get_uniform_location(program, "u_gradient_texture"),
    }
}

// ============================================================================
// WebGlRenderer
// ============================================================================

#[derive(Debug)]
pub struct WebGlRenderer {
    programs: WebGlPrograms,
    gl: WebGl2RenderingContext,
}

impl WebGlRenderer {
    pub fn new(canvas: &HtmlCanvasElement) -> Self {
        let context_options = js_sys::Object::new();
        js_sys::Reflect::set(&context_options, &"antialias".into(), &JsValue::FALSE).unwrap();
        js_sys::Reflect::set(&context_options, &"depth".into(), &JsValue::TRUE).unwrap();

        let gl = canvas
            .get_context_with_context_options("webgl2", &context_options)
            .expect("WebGL2 context to be available")
            .unwrap()
            .dyn_into::<WebGl2RenderingContext>()
            .expect("Context to be a WebGL2 context");

        assert!(
            gl.get_parameter(WebGl2RenderingContext::DEPTH_BITS)
                .unwrap()
                .as_f64()
                .unwrap() >= 24.0,
            "Depth buffer must be at least 24 bits"
        );

        let programs = WebGlPrograms::new(&gl);

        Self { programs, gl }
    }

    pub fn render(&mut self, scene: &Scene, render_size: &RenderSize) {
        // Upload textures from scene
        self.programs.prepare(&self.gl, scene);

        // Clear
        self.gl.viewport(0, 0, render_size.width as i32, render_size.height as i32);
        self.gl.clear_color(1.0, 1.0, 1.0, 1.0);
        self.gl.clear_depth(1.0);
        self.gl.clear(
            WebGl2RenderingContext::COLOR_BUFFER_BIT | WebGl2RenderingContext::DEPTH_BUFFER_BIT,
        );

        // Draw
        self.gl.use_program(Some(&self.programs.tile_program));
        self.render_tiles(scene.tiles(), render_size);
    }

    fn render_tiles(&self, tiles: &[Tile], render_size: &RenderSize) {
        if tiles.is_empty() {
            return;
        }

        let max_w = self.programs.resources.max_texture_dimension_2d;

        // Upload config UBO
       let config = Config {
    width: render_size.width,
    height: render_size.height,
    tile_height: 4u32,
    segments_tex_width_bits: max_w.trailing_zeros(),
    segment_list_tex_width_bits: max_w.trailing_zeros(),
    encoded_paints_tex_width_bits: max_w.trailing_zeros(),
    negate_ndc: 0,
    _pad0: 0,
};

        self.gl.bind_buffer(
            WebGl2RenderingContext::UNIFORM_BUFFER,
            Some(&self.programs.resources.config_buffer),
        );
        self.gl.buffer_data_with_u8_array(
            WebGl2RenderingContext::UNIFORM_BUFFER,
            bytemuck::bytes_of(&config),
            WebGl2RenderingContext::DYNAMIC_DRAW,
        );
        self.gl.bind_buffer_base(
            WebGl2RenderingContext::UNIFORM_BUFFER,
            0,
            Some(&self.programs.resources.config_buffer),
        );

        // Bind Texture 0: segments
        self.gl.active_texture(WebGl2RenderingContext::TEXTURE0);
        self.gl.bind_texture(
            WebGl2RenderingContext::TEXTURE_2D,
            Some(&self.programs.resources.segments_texture),
        );
        self.gl.uniform1i(Some(&self.programs.tile_uniforms.segments_texture), 0);

        // Bind Texture 1: segment list
        self.gl.active_texture(WebGl2RenderingContext::TEXTURE1);
        self.gl.bind_texture(
            WebGl2RenderingContext::TEXTURE_2D,
            Some(&self.programs.resources.segment_list_texture),
        );
        self.gl.uniform1i(Some(&self.programs.tile_uniforms.segment_list_texture), 1);

        // Bind Texture 2: encoded paints (if available)
        if let Some(loc) = &self.programs.tile_uniforms.encoded_paints_texture {
            self.gl.active_texture(WebGl2RenderingContext::TEXTURE2);
            self.gl.bind_texture(
                WebGl2RenderingContext::TEXTURE_2D,
                Some(&self.programs.resources.encoded_paints_texture),
            );
            self.gl.uniform1i(Some(loc), 2);
        }

        // Bind Texture 3: gradient LUT (if available)
        if let Some(loc) = &self.programs.tile_uniforms.gradient_texture {
            self.gl.active_texture(WebGl2RenderingContext::TEXTURE3);
            self.gl.bind_texture(
                WebGl2RenderingContext::TEXTURE_2D,
                Some(&self.programs.resources.gradient_texture),
            );
            self.gl.uniform1i(Some(loc), 3);
        }

        // Upload tiles to VBO
        let tiles_bytes: &[u8] = bytemuck::cast_slice(tiles);
        self.gl.bind_buffer(
            WebGl2RenderingContext::ARRAY_BUFFER,
            Some(&self.programs.resources.tiles_buffer),
        );
        self.gl.buffer_data_with_u8_array(
            WebGl2RenderingContext::ARRAY_BUFFER,
            tiles_bytes,
            WebGl2RenderingContext::DYNAMIC_DRAW,
        );

        // Draw instanced
        self.gl.bind_vertex_array(Some(&self.programs.resources.tile_vao));

        self.gl.enable(WebGl2RenderingContext::DEPTH_TEST);
self.gl.depth_func(WebGl2RenderingContext::LESS);
self.gl.depth_mask(true);
        self.gl.enable(WebGl2RenderingContext::BLEND);
        self.gl.blend_func(
            WebGl2RenderingContext::SRC_ALPHA,
            WebGl2RenderingContext::ONE_MINUS_SRC_ALPHA,
        );

        self.gl.draw_arrays_instanced(
            WebGl2RenderingContext::TRIANGLE_STRIP,
            0,
            4,
            tiles.len() as i32,
        );

        self.gl.bind_vertex_array(None);
    }
}

// ============================================================================
// Texture upload helpers
// ============================================================================

fn upload_data_to_rgba32f_texture(
    gl: &WebGl2RenderingContext,
    data: &[f32],
    texture_width: u32,
    texture_height: u32,
) {
    let packed_array = unsafe { js_sys::Float32Array::view(data) };
    gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_array_buffer_view(
        WebGl2RenderingContext::TEXTURE_2D,
        0,
        WebGl2RenderingContext::RGBA32F as i32,
        texture_width as i32,
        texture_height as i32,
        0,
        WebGl2RenderingContext::RGBA,
        WebGl2RenderingContext::FLOAT,
        Some(&packed_array),
    )
    .unwrap();
}

fn upload_data_to_r32ui_texture(
    gl: &WebGl2RenderingContext,
    data: &[u32],
    texture_width: u32,
    texture_height: u32,
) {
    let packed_array = unsafe { js_sys::Uint32Array::view(data) };
    gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_array_buffer_view(
        WebGl2RenderingContext::TEXTURE_2D,
        0,
        WebGl2RenderingContext::R32UI as i32,
        texture_width as i32,
        texture_height as i32,
        0,
        WebGl2RenderingContext::RED_INTEGER,
        WebGl2RenderingContext::UNSIGNED_INT,
        Some(&packed_array),
    )
    .unwrap();
}

fn upload_data_to_rgba32ui_texture(
    gl: &WebGl2RenderingContext,
    data: &[u32],
    texture_width: u32,
    texture_height: u32,
) {
    let packed_array = unsafe { js_sys::Uint32Array::view(data) };
    gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_array_buffer_view(
        WebGl2RenderingContext::TEXTURE_2D,
        0,
        WebGl2RenderingContext::RGBA32UI as i32,
        texture_width as i32,
        texture_height as i32,
        0,
        WebGl2RenderingContext::RGBA_INTEGER,
        WebGl2RenderingContext::UNSIGNED_INT,
        Some(&packed_array),
    )
    .unwrap();
}

// ============================================================================
// Shader compilation
// ============================================================================

fn create_shader_program(
    gl: &WebGl2RenderingContext,
    vertex_src: &str,
    fragment_src: &str,
) -> WebGlProgram {
    let vertex_shader = gl.create_shader(WebGl2RenderingContext::VERTEX_SHADER).unwrap();
    gl.shader_source(&vertex_shader, vertex_src);
    gl.compile_shader(&vertex_shader);
    if !gl.get_shader_parameter(&vertex_shader, WebGl2RenderingContext::COMPILE_STATUS).as_bool().unwrap_or(false) {
        let info = gl.get_shader_info_log(&vertex_shader).unwrap_or_default();
        panic!("Failed to compile vertex shader: {info}");
    }

    let fragment_shader = gl.create_shader(WebGl2RenderingContext::FRAGMENT_SHADER).unwrap();
    gl.shader_source(&fragment_shader, fragment_src);
    gl.compile_shader(&fragment_shader);
    if !gl.get_shader_parameter(&fragment_shader, WebGl2RenderingContext::COMPILE_STATUS).as_bool().unwrap_or(false) {
        let info = gl.get_shader_info_log(&fragment_shader).unwrap_or_default();
        panic!("Failed to compile fragment shader: {info}");
    }

    let program = gl.create_program().unwrap();
    gl.attach_shader(&program, &vertex_shader);
    gl.attach_shader(&program, &fragment_shader);
    gl.link_program(&program);
    if !gl.get_program_parameter(&program, WebGl2RenderingContext::LINK_STATUS).as_bool().unwrap_or(false) {
        let info = gl.get_program_info_log(&program).unwrap_or_default();
        panic!("Failed to link program: {info}");
    }

    gl.delete_shader(Some(&vertex_shader));
    gl.delete_shader(Some(&fragment_shader));
    program
}

// ============================================================================
// VAO setup
// ============================================================================

fn initialize_tile_vao(gl: &WebGl2RenderingContext, resources: &WebGlResources) {
    gl.bind_vertex_array(Some(&resources.tile_vao));
    gl.bind_buffer(
        WebGl2RenderingContext::ARRAY_BUFFER,
        Some(&resources.tiles_buffer),
    );

    let stride = core::mem::size_of::<Tile>() as i32;

    // Attribute 0: x, y (2 × u16 packed as 1 u32)
    gl.enable_vertex_attrib_array(0);
    gl.vertex_attrib_i_pointer_with_i32(0, 1, WebGl2RenderingContext::UNSIGNED_INT, stride, 0);
    gl.vertex_attrib_divisor(0, 1);

    // Attribute 1: width, height, _pad (4 × u8 packed as 1 u32)
    gl.enable_vertex_attrib_array(1);
    gl.vertex_attrib_i_pointer_with_i32(1, 1, WebGl2RenderingContext::UNSIGNED_INT, stride, 4);
    gl.vertex_attrib_divisor(1, 1);

    // Attribute 2: backdrop[2] (2 × i32)
    gl.enable_vertex_attrib_array(2);
    gl.vertex_attrib_i_pointer_with_i32(2, 2, WebGl2RenderingContext::INT, stride, 8);
    gl.vertex_attrib_divisor(2, 1);

    // Attribute 3: segments[2] (2 × f32 stored as u32 via from_bits)
    gl.enable_vertex_attrib_array(3);
    gl.vertex_attrib_i_pointer_with_i32(3, 2, WebGl2RenderingContext::UNSIGNED_INT, stride, 16);
    gl.vertex_attrib_divisor(3, 1);

    // Attribute 4: payload, paint_and_rect_flag, depth_index (3 × u32)
    gl.enable_vertex_attrib_array(4);
    gl.vertex_attrib_i_pointer_with_i32(4, 3, WebGl2RenderingContext::UNSIGNED_INT, stride, 24);
    gl.vertex_attrib_divisor(4, 1);

    gl.bind_vertex_array(None);
}
