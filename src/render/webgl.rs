#![expect(
    clippy::cast_possible_truncation,
    reason = "We temporarily ignore those because the casts\
only break in edge cases, and some of them are also only related to conversions from f64 to f32."
)]

use crate::render::common::render_tile;

use crate::{
    RenderError, RenderSize, Tile,
    paint::encode::MAX_GRADIENT_LUT_SIZE,
    paint::gradient_cache::{CachedRamp, GradientRampCache},
    paint::{
        EncodedBlurredRoundedRectangle, EncodedGradient, EncodedImage, EncodedKind, EncodedPaint,
        RadialKind,
    },
    render::{
        Config,
        common::{
            GPU_ENCODED_IMAGE_SIZE_TEXELS, GPU_LINEAR_GRADIENT_SIZE_TEXELS,
            GPU_RADIAL_GRADIENT_SIZE_TEXELS, GPU_SWEEP_GRADIENT_SIZE_TEXELS, GpuEncodedImage,
            GpuEncodedPaint, GpuLinearGradient, GpuRadialGradient, GpuSweepGradient,
            pack_image_offset, pack_image_params, pack_image_size, pack_radial_kind_and_swapped,
            pack_texture_width_and_extend_mode,
        },
    },
    scene::{RenderSettings, Scene},
};

use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;
use bytemuck::{Pod, Zeroable};
use kurbo::segments;
use web_sys::wasm_bindgen::{JsCast, JsValue};
use web_sys::{
    HtmlCanvasElement, WebGl2RenderingContext, WebGlBuffer, WebGlFramebuffer, WebGlProgram,
    WebGlTexture, WebGlUniformLocation, WebGlVertexArrayObject,
};

const GPU_PAINT_PLACEHOLDER: GpuEncodedPaint = GpuEncodedPaint::LinearGradient(GpuLinearGradient {
    texture_width_and_extend_mode: 0,
    gradient_start: 0,
    transform: [0.0; 6],
});
/// Query the WebGL context for the max texture size.
fn get_max_texture_dimension_2d(gl: &WebGl2RenderingContext) -> u32 {
    gl.get_parameter(WebGl2RenderingContext::MAX_TEXTURE_SIZE)
        .unwrap()
        .as_f64()
        .unwrap() as u32
}

fn get_max_texture_array_layers(gl: &WebGl2RenderingContext) -> u32 {
    gl.get_parameter(WebGl2RenderingContext::MAX_ARRAY_TEXTURE_LAYERS)
        .unwrap()
        .as_f64()
        .unwrap() as u32
}

#[derive(Debug)]
struct WebGlPrograms {
    tile_program: WebGlProgram,
    tile_uniforms: TileUniforms,
    resources: WebGlResources,
    encoded_paints_data: Vec<u8>,
}

impl WebGlPrograms {
    fn new(gl: WebGl2RenderingContext) -> Self {
        let tile_program = create_shader_program(
            &gl,
            render_tile::VERTEX_SOURCE,
            render_tile::FRAGMENT_SOURCE,
        );
        let tile_uniforms = get_tile_uniforms(&gl, &tile_program);
        let resources = create_webgl_resources(&gl);

        initialize_tile_vao(&gl, &resources);

        let encoded_paints_data = vec![0; (resources.max_texture_dimension_2d << 4) as usize];
        Self {
            tile_program,
            tile_uniforms,
            resources,
            encoded_paints_data,
        }
    }

    fn prepare(
        &mut self,
        gl: &WebGl2RenderingContext,
        encoded_paints: &[GpuEncodedPaint],
        segments: &mut Vec<f32>,
        render_size: &RenderSize,
        paint_idxs: &[u32],
    ) {
        let max_texture_dimension_2d = self.resources.max_texture_dimension_2d;

        self.resize_segments_texture(max_texture_dimension_2d, segments.len());
        self.resize_encoded_paints_texture(max_texture_dimension_2d, paint_idxs);

        self.upload_segments_texture(gl, segments);
        self.upload_encoded_paints_texture(gl, encoded_paints);
    }

    fn upload_segments_texture(&mut self, gl: &WebGl2RenderingContext, segments: &mut Vec<f32>) {
        if segments.is_empty() {
            return;
        }

        let segments_texture_width = self.resources.max_texture_dimension_2d;
        let segments_texture_height = self.resources.segments_texture_height;
        let total_size = segments_texture_width as usize * segments_texture_height as usize * 16;

        let n_segments = segments.len() / 6;
        let total_texels = n_segments * 2;
        let required_height = (total_texels as u32)
            .div_ceil(segments_texture_width)
            .max(1);
        let original_len = segments.len();

        let mut texture_data = vec![0_f32; (segments_texture_width * required_height * 4) as usize];
        for i in 0..n_segments {
            let src = i * 6;
            let dst = i * 8; // 2 texels = 8 floats
            // Texel 0: p0.x, p0.y, p1.x, p1.y
            texture_data[dst] = segments[src];
            texture_data[dst + 1] = segments[src + 1];
            texture_data[dst + 2] = segments[src + 2];
            texture_data[dst + 3] = segments[src + 3];
            // Texel 1: p2.x, p2.y, 0, 0
            texture_data[dst + 4] = segments[src + 4];
            texture_data[dst + 5] = segments[src + 5];
            texture_data[dst + 6] = 0.0;
            texture_data[dst + 7] = 0.0;
        }

        // // Temporarily pad the length of the alphas to the texture size before uploading.
        // segments.resize(total_size, 0.0);
        gl.active_texture(WebGl2RenderingContext::TEXTURE0);
        gl.bind_texture(
            WebGl2RenderingContext::TEXTURE_2D,
            Some(&self.resources.segments),
        );
        upload_data_to_rgba32f_texture(
            gl,
            &texture_data,
            segments_texture_width,
            segments_texture_height,
        );
    }
    fn upload_encoded_paints_texture(
        &mut self,
        gl: &WebGl2RenderingContext,
        encoded_paints: &[GpuEncodedPaint],
    ) {
        if !encoded_paints.is_empty() {
            let encoded_paints_texture_width = self.resources.max_texture_dimension_2d;
            let encoded_paints_texture_height = self.resources.encoded_paints_texture_height;

            GpuEncodedPaint::serialize_to_buffer(encoded_paints, &mut self.encoded_paints_data);

            gl.active_texture(WebGl2RenderingContext::TEXTURE3);
            gl.bind_texture(
                WebGl2RenderingContext::TEXTURE_2D,
                Some(&self.resources.encoded_paints_texture),
            );

            upload_data_to_rgba32_texture(
                gl,
                bytemuck::cast_slice::<u8, u32>(&self.encoded_paints_data),
                encoded_paints_texture_width,
                encoded_paints_texture_height,
            );
        }
    }
    fn resize_segments_texture(&mut self, max_texture_dimension_2d: u32, segments_len: usize) {
    let n_segments = (segments_len / 6) as u32;
    let total_texels = n_segments * 2;
    let required_height = total_texels.div_ceil(max_texture_dimension_2d).max(1);
    
    if required_height > self.resources.segments_texture_height {
        assert!(
            required_height <= max_texture_dimension_2d,
            "Segments texture height exceeds max texture dimensions"
        );
        self.resources.segments_texture_height = required_height;
    }
}
    fn resize_encoded_paints_texture(&mut self, max_texture_dimension_2d: u32, paint_idxs: &[u32]) {
        let required_texels = paint_idxs.last().unwrap();
        let required_encoded_paints_height = required_texels.div_ceil(max_texture_dimension_2d);
        let current_encoded_paints_height = self.resources.encoded_paints_texture_height;
        if required_encoded_paints_height > current_encoded_paints_height {
            assert!(
                required_encoded_paints_height <= max_texture_dimension_2d,
                "Encoded paints texture height exceeds max texture dimensions"
            );

            let required_encoded_paints_size =
                (max_texture_dimension_2d * required_encoded_paints_height) << 4;
            self.encoded_paints_data
                .resize(required_encoded_paints_size as usize, 0);
            self.resources.encoded_paints_texture_height = required_encoded_paints_height;
        }
    }
}

fn get_tile_uniforms(gl: &WebGl2RenderingContext, program: &WebGlProgram) -> TileUniforms {
    // Get uniform block indices for config
    let config_vs_name = "config"; // Your vertex shader config uniform name  
    let config_vs_block_index = gl.get_uniform_block_index(program, config_vs_name);

    let config_fs_name = "config"; // Your fragment shader config uniform name    
    let config_fs_block_index = gl.get_uniform_block_index(program, config_fs_name);

    debug_assert_ne!(
        config_vs_block_index,
        WebGl2RenderingContext::INVALID_INDEX,
        "invalid uniform index"
    );
    debug_assert_ne!(
        config_fs_block_index,
        WebGl2RenderingContext::INVALID_INDEX,
        "invalid uniform index"
    );

    // Bind uniform blocks to binding points
    gl.uniform_block_binding(program, config_vs_block_index, 0);
    gl.uniform_block_binding(program, config_fs_block_index, 0);

    // Get texture uniform locations for your tile shader
    let segments_texture_name = "segments_texture"; // Your segment texture uniform name  
    let atlas_texture_array_name = "atlas_texture_array";
    let encoded_paints_texture_fs_name = "encoded_paints_texture";
    let encoded_paints_texture_vs_name = "encoded_paints_texture";
    let gradient_texture_name = "gradient_texture";

    TileUniforms {
        config_vs_block_index,
        config_fs_block_index,
        segments: gl
            .get_uniform_location(program, segments_texture_name)
            .unwrap(),
        atlas_texture_array: gl.get_uniform_location(program, atlas_texture_array_name),
        encoded_paints_texture_fs: gl.get_uniform_location(program, encoded_paints_texture_fs_name),
        encoded_paints_texture_vs: gl
            .get_uniform_location(program, encoded_paints_texture_vs_name)
            ,
        gradient_texture: gl
            .get_uniform_location(program, gradient_texture_name)
            ,
    }
}

#[derive(Debug)]
struct WebGlResources {
    tile_vao: WebGlVertexArrayObject,
    tiles_buffer: WebGlBuffer,
    segments: WebGlTexture,
    segments_texture_height: u32,
    encoded_paints_texture: WebGlTexture,
    encoded_paints_texture_height: u32,
    gradient_texture: WebGlTexture,
    gradient_texture_height: u32,
    max_texture_dimension_2d: u32,
    config_buffer: WebGlBuffer,
}

fn create_webgl_resources(gl: &WebGl2RenderingContext) -> WebGlResources {
    let tile_vao = gl.create_vertex_array().unwrap();
    let tiles_buffer = gl.create_buffer().unwrap();
    let segments = gl.create_texture().unwrap();
    {
        gl.active_texture(WebGl2RenderingContext::TEXTURE0);
        gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&segments));
        gl.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_MIN_FILTER,
            WebGl2RenderingContext::NEAREST as i32,
        );
        gl.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_MAG_FILTER,
            WebGl2RenderingContext::NEAREST as i32,
        );
        gl.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_WRAP_S,
            WebGl2RenderingContext::CLAMP_TO_EDGE as i32,
        );
        gl.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_WRAP_T,
            WebGl2RenderingContext::CLAMP_TO_EDGE as i32,
        );
    }
    let encoded_paints_texture = gl.create_texture().unwrap();
    {
        gl.active_texture(WebGl2RenderingContext::TEXTURE1);
        gl.bind_texture(
            WebGl2RenderingContext::TEXTURE_2D,
            Some(&encoded_paints_texture),
        );
        gl.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_MIN_FILTER,
            WebGl2RenderingContext::NEAREST as i32,
        );
        gl.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_MAG_FILTER,
            WebGl2RenderingContext::NEAREST as i32,
        );
        gl.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_WRAP_S,
            WebGl2RenderingContext::CLAMP_TO_EDGE as i32,
        );
        gl.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_WRAP_T,
            WebGl2RenderingContext::CLAMP_TO_EDGE as i32,
        );
    }
    let gradient_texture = gl.create_texture().unwrap();
    {
        gl.active_texture(WebGl2RenderingContext::TEXTURE2);
        gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&gradient_texture));
        gl.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_MIN_FILTER,
            WebGl2RenderingContext::LINEAR as i32,
        );
        gl.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_MAG_FILTER,
            WebGl2RenderingContext::LINEAR as i32,
        );
        gl.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_WRAP_S,
            WebGl2RenderingContext::CLAMP_TO_EDGE as i32,
        );
        gl.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_WRAP_T,
            WebGl2RenderingContext::CLAMP_TO_EDGE as i32,
        );
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
        segments,
        segments_texture_height: 0,
        encoded_paints_texture,
        encoded_paints_texture_height: 0,
        gradient_texture,
        gradient_texture_height: 0,
        max_texture_dimension_2d,
        config_buffer,
    }
}

fn upload_data_to_rgba32_texture(
    gl: &WebGl2RenderingContext,
    data: &[u32],
    texture_width: u32,
    texture_height: u32,
) {
    // Safety: This calling `Uint32Array::view` is unsafe because it provides a view into
    // WASM linear memory, and any additional allocations might invalidate that view.
    // In our case, this is not an issue because we only use this view once for uploading
    // data to the GPU below, and no allocations happen between that.
    // The `tex_image_2d` method is synchronous in the sense that once it returns, it is guaranteed
    // that all necessary data has already been read, so any allocations that happen
    // after this block don't affect this anymore.
    //
    // See also: https://wikis.khronos.org/opengl/Synchronization
    // >> There are several OpenGL functions that can pull data directly from client-side memory,
    // >> or push data directly into client-side memory. Functions like `glTexSubImage2D`,
    // >> `glReadPixels`, `glBufferSubData` and so forth.
    //
    // >> Because OpenGL is defined to be synchronous, when any of these functions have
    // >> returned, they must have finished with the client memory. When `glReadPixels` returns,
    // >> the pixel data is in your client memory (unless you are reading into a buffer object).
    // >> When `glBufferSubData` returns, you can immediately modify or delete whatever memory
    // >> pointer you gave it, as OpenGL has already read as much as it wants.
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

fn upload_data_to_rgba32f_texture(
    gl: &WebGl2RenderingContext,
    data: &[f32],
    texture_width: u32,
    texture_height: u32,
) {
    // Safety: This calling `Uint32Array::view` is unsafe because it provides a view into
    // WASM linear memory, and any additional allocations might invalidate that view.
    // In our case, this is not an issue because we only use this view once for uploading
    // data to the GPU below, and no allocations happen between that.
    // The `tex_image_2d` method is synchronous in the sense that once it returns, it is guaranteed
    // that all necessary data has already been read, so any allocations that happen
    // after this block don't affect this anymore.
    //
    // See also: https://wikis.khronos.org/opengl/Synchronization
    // >> There are several OpenGL functions that can pull data directly from client-side memory,
    // >> or push data directly into client-side memory. Functions like `glTexSubImage2D`,
    // >> `glReadPixels`, `glBufferSubData` and so forth.
    //
    // >> Because OpenGL is defined to be synchronous, when any of these functions have
    // >> returned, they must have finished with the client memory. When `glReadPixels` returns,
    // >> the pixel data is in your client memory (unless you are reading into a buffer object).
    // >> When `glBufferSubData` returns, you can immediately modify or delete whatever memory
    // >> pointer you gave it, as OpenGL has already read as much as it wants.
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

#[derive(Debug)]
pub struct WebGlRenderer {
    /// Programs for rendering.  
    programs: WebGlPrograms,
    /// WebGL context.  
    gl: WebGl2RenderingContext,
    /// Encoded paints for storing encoded paints.  
    encoded_paints: Vec<GpuEncodedPaint>,
    /// Stores the index (offset) of the encoded paints in the encoded paints texture.  
    paint_idxs: Vec<u32>,
    gradient_cache: GradientRampCache,
}
impl WebGlRenderer {
    pub fn new(canvas: &web_sys::HtmlCanvasElement) -> Self {
        Self::new_with(canvas, RenderSettings::default())
    }
    pub fn new_with(canvas: &web_sys::HtmlCanvasElement, settings: RenderSettings) -> Self {
        // We do our own anti-aliasing, so no need to enable it in the WebGL
        // context.
        let context_options = js_sys::Object::new();
        js_sys::Reflect::set(&context_options, &"antialias".into(), &JsValue::FALSE).unwrap();
        // Vello only supports 24+ bit depth buffers. If the hardware falls back to a 16 bit depth buffer,
        // correctness issues will arise. For all intents and purposes, a device manufactured in the past 10 years
        // should support 24+ bit depth buffers (certainly those within the realm of what we consider "supported" devices)
        // but:
        //
        // Relevant code for default depth buffer behaviour can be found here:
        // - Chromium defaults to 24 bit with no fallback: https://github.com/chromium/chromium/blob/86bafb3aab8e999690d310b201d0b5489f512b08/third_party/blink/renderer/platform/graphics/gpu/drawing_buffer.cc#L1376-L1400
        // - Firefox defaults to 24 bit with no fallback: https://github.com/mozilla/gecko-dev/blob/5836a062726f715fda621338a17b51aff30d0a8c/gfx/gl/MozFramebuffer.cpp#L155-L161
        // - Safari defaults to 24 bit _with 16 bit_ fallback: https://github.com/WebKit/WebKit/blob/a6d6c154bbee0643f5ad1e55c071558c0df9aef7/Source/WebCore/platform/graphics/angle/GraphicsContextGLANGLE.cpp#L393-L416
        //
        // TODO: The above understanding is encoded in a below assertion, but this should be encapsulated within a
        // "this device can run Vello correctly" check function.
        js_sys::Reflect::set(&context_options, &"depth".into(), &JsValue::TRUE).unwrap();

        let gl = canvas
            .get_context_with_context_options("webgl2", &context_options)
            .expect("WebGL2 context to be available")
            .unwrap()
            .dyn_into::<WebGl2RenderingContext>()
            .expect("Context to be a WebGL2 context");

        // Note: It is not entirely clear whether we really _have_ to ensure anti-aliasing is disabled.
        // This code is inherited from a similar snippet in wgpu
        // (https://github.com/gfx-rs/wgpu/blob/56e4a389ddd02403e232beef3d3ff305625e6485/wgpu-hal/src/gles/web.rs#L101-L106),
        // which itself seems to have been copied from the older `gfx` crate, where it was first introduced
        // in https://github.com/gfx-rs/gfx/pull/2554/changes#diff-a47711d61df7a43fe6dd99c39b936d17ff817cbc2238d7e3ae6698ffde9b88f7R79,
        // without any comment on why.
        // From my (Laurenz) testing, tests seem to work even when anti-aliasing is enabled,
        // but Andrew previously got errors similar to the ones outlined in
        // https://github.com/gfx-rs/wgpu/issues/5263. Therefore, we just leave it as is for now.
       

        let mut settings = settings;
        let max_texture_dimension_2d = get_max_texture_dimension_2d(&gl);
        // normalize_atlas_config(
        //     &mut settings.atlas_config,
        //     max_texture_dimension_2d,
        //     get_max_texture_array_layers(&gl),
        //     1,
        // );
        let total_slots: usize = (max_texture_dimension_2d / u32::from(Tile::HEIGHT)) as usize;
        assert!(
            gl.get_parameter(WebGl2RenderingContext::DEPTH_BITS)
                .unwrap()
                .as_f64()
                .unwrap()
                >= 24.0,
            "Depth buffer must be at least 24 bits"
        );
        let max_gradient_cache_size =
            max_texture_dimension_2d * max_texture_dimension_2d / MAX_GRADIENT_LUT_SIZE as u32;
        let gradient_cache = GradientRampCache::new(max_gradient_cache_size, settings.level);
        Self {
            programs: WebGlPrograms::new(gl.clone()),
            gl,
            encoded_paints: Vec::new(),
            paint_idxs: Vec::new(),
            gradient_cache,
        }
    }

    pub fn render(&mut self, scene: &mut Scene, render_size: &RenderSize) {
        let encoded_paints = scene.encoded_paints.borrow_mut();

        self.prepare_gpu_encoded_paints(&encoded_paints);

        self.programs.prepare(
            &self.gl,
            &self.encoded_paints,
            &mut scene.segments.get_mut(),
            render_size,
            &self.paint_idxs,
        );

        self.gl
            .viewport(0, 0, render_size.width as i32, render_size.height as i32);
        self.gl.clear_color(1.0, 1.0, 1.0, 1.0); // white background
        self.gl.clear_depth(1.0);
        self.gl.clear(
            WebGl2RenderingContext::COLOR_BUFFER_BIT | WebGl2RenderingContext::DEPTH_BUFFER_BIT,
        );

        self.gl.use_program(Some(&self.programs.tile_program));

        let mut ctx = WebGlRendererContext {
            programs: &mut self.programs,
            gl: &self.gl,
        };
        let tiles = &scene.tiles.borrow();
        ctx.render_tiles(tiles, &render_size);
        self.gradient_cache.maintain();

        // Ok(());
    }

    fn prepare_gpu_encoded_paints(&mut self, encoded_paints: &[EncodedPaint]) {
        self.encoded_paints
            .resize_with(encoded_paints.len(), || GPU_PAINT_PLACEHOLDER);
        self.paint_idxs.resize(encoded_paints.len() + 1, 0);

        let mut current_idx = 0;
        for (encoded_paint_idx, paint) in encoded_paints.iter().enumerate() {
            self.paint_idxs[encoded_paint_idx] = current_idx;
            match paint {
                EncodedPaint::Image(img) => {
                    // deal with this later..
                }
                EncodedPaint::Gradient(gradient) => {
                    let (gradient_start, gradient_width) =
                        self.gradient_cache.get_or_create_ramp(gradient);
                    let gpu_gradient =
                        self.encode_gradient_paint(gradient, gradient_width, gradient_start);
                    let gradient_size_texels = match &gpu_gradient {
                        GpuEncodedPaint::LinearGradient(_) => GPU_LINEAR_GRADIENT_SIZE_TEXELS,
                        GpuEncodedPaint::RadialGradient(_) => GPU_RADIAL_GRADIENT_SIZE_TEXELS,
                        GpuEncodedPaint::SweepGradient(_) => GPU_SWEEP_GRADIENT_SIZE_TEXELS,
                        _ => unreachable!("encode_gradient_for_gpu only returns gradient types"),
                    };
                    self.encoded_paints[encoded_paint_idx] = gpu_gradient;
                    current_idx += gradient_size_texels;
                }
                EncodedPaint::BlurredRoundedRect(_blurred_rect) => {
                    // TODO: Blurred rounded rectangles are not yet supported
                    log::warn!(
                        "Blurred rounded rectangles are not yet supported in sparse strips hybrid renderer"
                    );
                }
            }
        }
        self.paint_idxs[encoded_paints.len()] = current_idx;
    }

    fn encode_gradient_paint(
        &self,
        gradient: &EncodedGradient,
        gradient_width: u32,
        gradient_start: u32,
    ) -> GpuEncodedPaint {
        let transform = gradient.transform.as_coeffs().map(|x| x as f32);
        let extend_mode = match gradient.extend {
            peniko::Extend::Pad => 0,
            peniko::Extend::Repeat => 1,
            peniko::Extend::Reflect => 2,
        };
        let texture_width_and_extend_mode =
            pack_texture_width_and_extend_mode(gradient_width, extend_mode);

        match &gradient.kind {
            EncodedKind::Linear(_) => GpuEncodedPaint::LinearGradient(GpuLinearGradient {
                texture_width_and_extend_mode,
                gradient_start,
                transform,
            }),
            EncodedKind::Radial(radial) => {
                let (kind, bias, scale, fp0, fp1, fr1, f_focal_x, f_is_swapped, scaled_r0_squared) =
                    match radial {
                        RadialKind::Radial { bias, scale } => {
                            (0, *bias, *scale, 0.0, 0.0, 0.0, 0.0, 0, 0.0)
                        }
                        RadialKind::Strip { scaled_r0_squared } => {
                            (1, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0, *scaled_r0_squared)
                        }
                        RadialKind::Focal {
                            focal_data,
                            fp0,
                            fp1,
                        } => (
                            2,
                            *fp0,
                            *fp1,
                            *fp0,
                            *fp1,
                            focal_data.fr1,
                            focal_data.f_focal_x,
                            focal_data.f_is_swapped as u32,
                            0.0,
                        ),
                    };
                GpuEncodedPaint::RadialGradient(GpuRadialGradient {
                    texture_width_and_extend_mode,
                    gradient_start,
                    transform,
                    kind_and_f_is_swapped: pack_radial_kind_and_swapped(kind, f_is_swapped),
                    bias,
                    scale,
                    fp0,
                    fp1,
                    fr1,
                    f_focal_x,
                    scaled_r0_squared,
                })
            }
            EncodedKind::Sweep(sweep) => GpuEncodedPaint::SweepGradient(GpuSweepGradient {
                texture_width_and_extend_mode,
                gradient_start,
                transform,
                start_angle: sweep.start_angle,
                inv_angle_delta: sweep.inv_angle_delta,
                _padding: [0, 0],
            }),
        }
    }
}
#[derive(Debug)]
struct WebGlRendererContext<'a> {
    programs: &'a mut WebGlPrograms,
    gl: &'a WebGl2RenderingContext,
}
impl WebGlRendererContext<'_> {
    fn render_tiles(&mut self, tiles: &[Tile], render_size: &RenderSize) {
        if tiles.is_empty() {
            return;
        };
        log::info!("depth is{:}", tiles[0].depth_index);
        let config = Config {
            width: render_size.width,
            height: render_size.height,
            tile_height: u32::from(Tile::HEIGHT),
            segments_tex_width_bits: self
                .programs
                .resources
                .max_texture_dimension_2d
                .trailing_zeros(),
            encoded_paints_tex_width_bits: self
                .programs
                .resources
                .max_texture_dimension_2d
                .trailing_zeros(),
            negate_ndc: 0,
            _pad0: 0,
            _pad1: 0,
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
            self.programs.tile_uniforms.config_vs_block_index,
            Some(&self.programs.resources.config_buffer),
        );
        self.gl.bind_buffer_base(
            WebGl2RenderingContext::UNIFORM_BUFFER,
            self.programs.tile_uniforms.config_fs_block_index,
            Some(&self.programs.resources.config_buffer),
        );
        self.gl.active_texture(WebGl2RenderingContext::TEXTURE1);
        self.gl.bind_texture(
            WebGl2RenderingContext::TEXTURE_2D,
            Some(&self.programs.resources.segments),
        );
        self.gl
            .uniform1i(Some(&self.programs.tile_uniforms.segments), 1);

        if let Some(loc) = &self.programs.tile_uniforms.encoded_paints_texture_vs {
            self.gl.active_texture(WebGl2RenderingContext::TEXTURE3);
            self.gl.bind_texture(
                WebGl2RenderingContext::TEXTURE_2D,
                Some(&self.programs.resources.encoded_paints_texture),
            );
            self.gl.uniform1i(Some(loc), 3);
        }

        if let Some(loc) = &self.programs.tile_uniforms.gradient_texture {
            self.gl.active_texture(WebGl2RenderingContext::TEXTURE4);
            self.gl.bind_texture(
                WebGl2RenderingContext::TEXTURE_2D,
                Some(&self.programs.resources.gradient_texture),
            );
            self.gl.uniform1i(Some(&loc), 4);
        }

        let tiles_len = tiles.len();
        let tiles = bytemuck::cast_slice(tiles);

        self.gl.bind_buffer(
            WebGl2RenderingContext::ARRAY_BUFFER,
            Some(&self.programs.resources.tiles_buffer),
        );
        self.gl.buffer_data_with_u8_array(
            WebGl2RenderingContext::ARRAY_BUFFER,
            tiles,
            WebGl2RenderingContext::DYNAMIC_DRAW,
        );

        self.gl.bind_vertex_array(Some(&self.programs.resources.tile_vao));

        if tiles_len as i32 > 0 {

            self.gl.enable(WebGl2RenderingContext::DEPTH_TEST);
            self.gl.depth_func(WebGl2RenderingContext::LEQUAL); // Closer or equal depth wins
            
            // 2. Clear the depth buffer for this set of tiles
            // If you call render_tiles multiple times per frame, 
            // you might want to clear this at the very start of the frame instead.
            self.gl.clear_depth(1.0);
            self.gl.clear(WebGl2RenderingContext::DEPTH_BUFFER_BIT);
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
                tiles_len as i32,
            );
        }

        self.gl.bind_vertex_array(None);
        self.gl.enable(WebGl2RenderingContext::BLEND);
self.gl.blend_func(
    WebGl2RenderingContext::SRC_ALPHA,
    WebGl2RenderingContext::ONE_MINUS_SRC_ALPHA,
);
    }
}
#[derive(Debug)]
struct TileUniforms {
    config_vs_block_index: u32,
    config_fs_block_index: u32,
    segments: WebGlUniformLocation,
    atlas_texture_array: Option<WebGlUniformLocation>,
    encoded_paints_texture_fs: Option<WebGlUniformLocation>,
    encoded_paints_texture_vs: Option<WebGlUniformLocation>,
    gradient_texture: Option<WebGlUniformLocation>,
}

fn create_shader_program(
    gl: &WebGl2RenderingContext,
    vertex_src: &str,
    fragment_src: &str,
) -> WebGlProgram {
    // Compile vertex shader.
    let vertex_shader = gl
        .create_shader(WebGl2RenderingContext::VERTEX_SHADER)
        .unwrap();
    gl.shader_source(&vertex_shader, vertex_src);
    gl.compile_shader(&vertex_shader);

    if !gl
        .get_shader_parameter(&vertex_shader, WebGl2RenderingContext::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        let info = gl
            .get_shader_info_log(&vertex_shader)
            .unwrap_or_else(|| "Unknown error creating vertex shader".into());
        panic!("Failed to compile vertex shader: {info}");
    }

    // Compile fragment shader.
    let fragment_shader = gl
        .create_shader(WebGl2RenderingContext::FRAGMENT_SHADER)
        .unwrap();
    gl.shader_source(&fragment_shader, fragment_src);
    gl.compile_shader(&fragment_shader);

    if !gl
        .get_shader_parameter(&fragment_shader, WebGl2RenderingContext::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        let info = gl
            .get_shader_info_log(&fragment_shader)
            .unwrap_or_else(|| "Unknown error creating fragment shader".into());
        panic!("Failed to compile fragment shader: {info}");
    }

    // Create and link the program.
    let program = gl.create_program().unwrap();
    gl.attach_shader(&program, &vertex_shader);
    gl.attach_shader(&program, &fragment_shader);
    gl.link_program(&program);

    if !gl
        .get_program_parameter(&program, WebGl2RenderingContext::LINK_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        let info = gl
            .get_program_info_log(&program)
            .unwrap_or_else(|| "Unknown error creating program".into());
        panic!("Failed to link program: {info}");
    }

    gl.delete_shader(Some(&vertex_shader));
    gl.delete_shader(Some(&fragment_shader));

    program
}
fn initialize_tile_vao(gl: &WebGl2RenderingContext, resources: &WebGlResources) {
    gl.bind_vertex_array(Some(&resources.tile_vao));
    gl.bind_buffer(
        WebGl2RenderingContext::ARRAY_BUFFER,
        Some(&resources.tiles_buffer),
    );

    let stride = core::mem::size_of::<Tile>() as i32;

    // Attribute 0: x, y (packed as u16s)
    gl.enable_vertex_attrib_array(0);
    gl.vertex_attrib_i_pointer_with_i32(
        0,
        1, // 1 u32 containing packed x,y
        WebGl2RenderingContext::UNSIGNED_INT,
        stride,
        0, // offset of x,y field
    );
    gl.vertex_attrib_divisor(0, 1);

    // Attribute 1: width, height (packed as u8s in u32)
    gl.enable_vertex_attrib_array(1);
    gl.vertex_attrib_i_pointer_with_i32(
        1,
        1, // 1 u32 containing packed width,height
        WebGl2RenderingContext::UNSIGNED_INT,
        stride,
        4, // offset after x,y
    );
    gl.vertex_attrib_divisor(1, 1);

    // Attribute 2: backdrop[0], backdrop[1] (2 u32s)
    gl.enable_vertex_attrib_array(2);
    gl.vertex_attrib_i_pointer_with_i32(
        2,
        2, // 2 u32s for backdrop array
        WebGl2RenderingContext::UNSIGNED_INT,
        stride,
        8, // offset after x,y,width,height
    );
    gl.vertex_attrib_divisor(2, 1);

    // Attribute 3: segment[0], segment[1] (2 u32s)
    gl.enable_vertex_attrib_array(3);
    gl.vertex_attrib_i_pointer_with_i32(
        3,
        2, // 2 u32s for segment array
        WebGl2RenderingContext::UNSIGNED_INT,
        stride,
        16, // offset after backdrop (8 + 8 bytes)
    );
    gl.vertex_attrib_divisor(3, 1);

    // Attribute 4: payload, paint_and_rect_flag, depth_index (3 u32s)
    gl.enable_vertex_attrib_array(4);
    gl.vertex_attrib_i_pointer_with_i32(
        4,
        3, // 3 u32s for remaining fields
        WebGl2RenderingContext::UNSIGNED_INT,
        stride,
        24, // offset after segment (16 + 8 bytes)
    );
    gl.vertex_attrib_divisor(4, 1);

    gl.bind_vertex_array(None);
}
