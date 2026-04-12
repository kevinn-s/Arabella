// use web_sys::wasm_bindgen::{JsCast, JsValue};
// use web_sys::js_sys;
// use web_sys::{
//     WebGl2RenderingContext, WebGlBuffer, WebGlFramebuffer, WebGlProgram, WebGlTexture,
//     WebGlUniformLocation, WebGlVertexArrayObject,
// };


// use super::constant::{BAND_FRAGMENT_SOURCE, BAND_VERTEX_SOURCE, 
//     CLEARS_VERTEX_SOURCE, RESOLVE_VERTEX_SOURCE, RESOLVE_FRAGMENT_SOURCE,
    
// };
// use super::common::{ Bands, Edges, LoadOp
// };

// struct WebGlPrograms {
//     band_program: WebGlProgram,
//     band_uniforms: BandUniforms,

//     resolve_program: WebGlProgram,
//     resolve_uniforms: ResolveUniforms,

//     clear_program: WebGlProgram,
//     clear_uniforms: ClearUniforms,
// }

// impl WebGlPrograms {
//     /// Creates programs and initializes resources.
//     fn new(
//         gl: WebGl2RenderingContext,
//     ) -> Self {
//         let band_program = create_shader_program(
//             &gl,
//             BAND_VERTEX_SOURCE,
//             BAND_FRAGMENT_SOURCE,
//         );
//         let band_uniforms= get_band_uniforms(&gl, &band_program);
//         let clear_program = create_shader_program(
//             &gl,
//             CLEARS_VERTEX_SOURCE,
//             ""
//         );
//         let clear_uniforms = get_clear_uniforms(&gl, &clear_program);
//         let resolve_program = create_shader_program(
//             &gl, 
//             RESOLVE_VERTEX_SOURCE,
//             RESOLVE_FRAGMENT_SOURCE 
//         );
//         let resolve_uniforms = get_resolve_uniforms(&gl, &resolve_program);
//         Self {
//             band_program,
//             band_uniforms,
//             resolve_program,
//             resolve_uniforms,
//             clear_program,
//             clear_uniforms
//         }
//     }

//     /// Upload strip data to GPU.
//     fn upload_strips(&mut self, gl: &WebGl2RenderingContext, bands: &[Bands]) {
//         if bands.is_empty() {
//             return;
//         }

//         gl.bind_buffer(
//             WebGl2RenderingContext::ARRAY_BUFFER,
//             Some(&self.resources.strips_buffer),
//         );
//         let strips_data = bytemuck::cast_slice(strips);
//         gl.buffer_data_with_u8_array(
//             WebGl2RenderingContext::ARRAY_BUFFER,
//             strips_data,
//             WebGl2RenderingContext::DYNAMIC_DRAW,
//         );
//     }
// }
// pub struct WebGlRenderer {
//     /// Programs for rendering.
//     programs: WebGlPrograms,
//     /// WebGL context.
//     gl: WebGl2RenderingContext,
// }

// impl WebGlRenderer {
//     /// Creates a new WebGL2 renderer
//     pub fn new(canvas: &web_sys::HtmlCanvasElement) -> Self {
//         let context_options = js_sys::Object::new();
//         // js_sys::Reflect::set(&context_options, &"antialias".into(), &JsValue::FALSE).unwrap();

//         let gl = canvas
//             .get_context_with_context_options("webgl2", &context_options)
//             .expect("WebGL2 context to be available")
//             .unwrap()
//             .dyn_into::<WebGl2RenderingContext>()
//             .expect("Context to be a WebGL2 context");
//         Self {
//             programs: WebGlPrograms::new(gl.clone()),
//             gl
//         }
//     }   

// }

// /// Create a WebGL shader program from vertex and fragment sources.
// fn create_shader_program(
//     gl: &WebGl2RenderingContext,
//     vertex_src: &str,
//     fragment_src: &str,
// ) -> WebGlProgram {
//     // Compile vertex shader.
//     let vertex_shader = gl
//         .create_shader(WebGl2RenderingContext::VERTEX_SHADER)
//         .unwrap();
//     gl.shader_source(&vertex_shader, vertex_src);
//     gl.compile_shader(&vertex_shader);

//     if !gl
//         .get_shader_parameter(&vertex_shader, WebGl2RenderingContext::COMPILE_STATUS)
//         .as_bool()
//         .unwrap_or(false)
//     {
//         let info = gl
//             .get_shader_info_log(&vertex_shader)
//             .unwrap_or_else(|| "Unknown error creating vertex shader".into());
//         panic!("Failed to compile vertex shader: {info}");
//     }

//     // Compile fragment shader.
//     let fragment_shader = gl
//         .create_shader(WebGl2RenderingContext::FRAGMENT_SHADER)
//         .unwrap();
//     gl.shader_source(&fragment_shader, fragment_src);
//     gl.compile_shader(&fragment_shader);

//     if !gl
//         .get_shader_parameter(&fragment_shader, WebGl2RenderingContext::COMPILE_STATUS)
//         .as_bool()
//         .unwrap_or(false)
//     {
//         let info = gl
//             .get_shader_info_log(&fragment_shader)
//             .unwrap_or_else(|| "Unknown error creating fragment shader".into());
//         panic!("Failed to compile fragment shader: {info}");
//     }

//     // Create and link the program.
//     let program = gl.create_program().unwrap();
//     gl.attach_shader(&program, &vertex_shader);
//     gl.attach_shader(&program, &fragment_shader);
//     gl.link_program(&program);

//     if !gl
//         .get_program_parameter(&program, WebGl2RenderingContext::LINK_STATUS)
//         .as_bool()
//         .unwrap_or(false)
//     {
//         let info = gl
//             .get_program_info_log(&program)
//             .unwrap_or_else(|| "Unknown error creating program".into());
//         panic!("Failed to link program: {info}");
//     }

//     gl.delete_shader(Some(&vertex_shader));
//     gl.delete_shader(Some(&fragment_shader));

//     program
// }

// struct WebGlRendererContext<'a> {
//     programs: &'a mut WebGlPrograms,
//     gl: &'a WebGl2RenderingContext,
// }

// impl WebGlRendererContext<'_> {
//     /// Render strips to the specified render target.
//     fn _do_bands_render_pass( 
//         &mut self,
//         bands: &[super::common::Bands],
//         load: LoadOp,
//     ){
//         if bands.is_empty() {
//             return;
//         }

//         // Clear framebuffer if requested.
//         if matches!(load, LoadOp::Clear) {
//             self.gl.clear_color(0.0, 0.0, 0.0, 0.0);
//             self.gl.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);
//         }

//         self.gl.use_program(Some(&self.programs.band_program));

//         self.gl.draw_arrays_instanced(
//             WebGl2RenderingContext::TRIANGLE_STRIP,
//             0,
//             4,
//             bands.len() as i32,
//         );
//         self.gl.bind_vertex_array(None);
//     }  
//     fn upload_bands () {

//     }
// }
// #[derive(Debug)]
// struct BandUniforms {
//     config_block_index: u32,
// }
// #[derive(Debug)]
// struct ResolveUniforms {
//     config_block_index: u32,
//     atlas_texture: WebGlUniformLocation,
// }
// #[derive(Debug)]
// struct ClearUniforms {
//     config_block_index: u32,
// }
// fn get_band_uniforms(gl: &WebGl2RenderingContext, program: &WebGlProgram) -> BandUniforms {
//     let config_vs_name = "Config";
//     let config_block_index = gl.get_uniform_block_index(program, config_vs_name);
//     BandUniforms {
//         config_block_index: config_block_index
//     }
// }
// fn get_clear_uniforms(gl: &WebGl2RenderingContext, program: &WebGlProgram) -> ClearUniforms {
//     let config_vs_name = "Config";
//     let config_block_index = gl.get_uniform_block_index(program, config_vs_name);
//     ClearUniforms {
//         config_block_index: config_block_index
//     }
// }
// fn get_resolve_uniforms(gl: &WebGl2RenderingContext, program: &WebGlProgram) -> ResolveUniforms {
//     let config_vs_name = "Config";
//     let config_block_index = gl.get_uniform_block_index(program, config_vs_name);
    
//     ResolveUniforms {
//         config_block_index: config_block_index ,
//         atlas_texture: gl
//             .get_uniform_location(program, "u_atlas")
//             .unwrap(),
//     }
// }

// fn create_texture(gl: &WebGl2RenderingContext) -> WebGlTexture {
//     let texture = gl.create_texture().unwrap();
//     gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&texture));
//     gl.tex_parameteri(
//         WebGl2RenderingContext::TEXTURE_2D,
//         WebGl2RenderingContext::TEXTURE_MIN_FILTER,
//         WebGl2RenderingContext::NEAREST as i32,
//     );
//     gl.tex_parameteri(
//         WebGl2RenderingContext::TEXTURE_2D,
//         WebGl2RenderingContext::TEXTURE_MAG_FILTER,
//         WebGl2RenderingContext::NEAREST as i32,
//     );
//     gl.tex_parameteri(
//         WebGl2RenderingContext::TEXTURE_2D,
//         WebGl2RenderingContext::TEXTURE_WRAP_S,
//         WebGl2RenderingContext::CLAMP_TO_EDGE as i32,
//     );
//     gl.tex_parameteri(
//         WebGl2RenderingContext::TEXTURE_2D,
//         WebGl2RenderingContext::TEXTURE_WRAP_T,
//         WebGl2RenderingContext::CLAMP_TO_EDGE as i32,
//     );
//     gl.tex_parameteri(
//         WebGl2RenderingContext::TEXTURE_2D,
//         WebGl2RenderingContext::TEXTURE_MAX_LEVEL,
//         0,
//     );
//     texture
// }

// fn create_framebuffer_for_texture(
//     gl: &WebGl2RenderingContext,
//     texture: &WebGlTexture,
// ) -> WebGlFramebuffer {
//     let framebuffer = gl.create_framebuffer().unwrap();
//     gl.bind_framebuffer(WebGl2RenderingContext::FRAMEBUFFER, Some(&fb));
//     gl.framebuffer_texture_2d(
//         WebGl2RenderingContext::FRAMEBUFFER,
//         WebGl2RenderingContext::COLOR_ATTACHMENT0,
//         WebGl2RenderingContext::TEXTURE_2D,
//         Some(texture),
//         0,
//     );
//     framebuffer
// }
// struct WebGlResources {
//     // === VAOs (vertex layout configurations) ===
//     edges_vao: WebGlVertexArrayObject,
//     // Buffer for [Band] data
//     edges_buffer: WebGlBuffer,
//     bands_vao: WebGlVertexArrayObject,
//     bands_buffer: WebGlBuffer,
//     clear_vao: WebGlVertexArrayObject,
//     atlas_texture: WebGlTexture,
//     /// Framebuffer wrapping atlas_texture for rendering into
//     atlas_framebuffer: WebGlFramebuffer,
//     atlas_width: u32,
//     atlas_height: u32,
// }

// fn create_webgl_resources(
//     gl: &WebGl2RenderingContext,
// ) -> WebGlResources {
//     let edges_vao = gl.create_vertex_array().unwrap();
//     let bands_vao = gl.create_vertex_array().unwrap();
//     let edges_buffer = gl.create_buffer().unwrap();
//     let bands_buffer = gl.create_buffer().unwrap();
//     let clear_vao = gl.create_vertex_array().unwrap();
    
//     // Create and configure atlas texture.
//     let atlas_texture = create_texture(gl);
//     let atlas_framebuffer = create_framebuffer_for_texture(gl, &atlas_texture);
//     let atlas_width = 4096;
//     let atlas_height = 4096;

//     WebGlResources { 
//         edges_vao, 
//         edges_buffer, 
//         bands_vao, 
//         bands_buffer, 
//         clear_vao, 
//         atlas_texture, 
//         atlas_framebuffer, 
//         atlas_width, 
//         atlas_height
//     }
// }