use js_sys::{WebAssembly, Uint32Array};
use wasm_bindgen::JsCast;
use web_sys::{OffscreenCanvas, WebGl2RenderingContext, WebGlProgram, WebGlShader};
use crate::{CalculateEscapeTimes, RenderingState};

pub struct WebGLRenderer {
    canvas: OffscreenCanvas,
    context: WebGl2RenderingContext,
    program: WebGlProgram,
}

impl WebGLRenderer {
    pub fn new() -> WebGLRenderer {
        //let document = web_sys::window().expect("1").document().expect("2");
        //let canvas = document.get_element_by_id("mandelbrot-canvas").expect("3");
        //let canvas = canvas.dyn_into::<web_sys::HtmlCanvasElement>().expect("4");

        let canvas = OffscreenCanvas::new(0, 0).expect("Could not load offscreencanvas");

        let context = canvas.get_context("webgl2").expect("5").unwrap().dyn_into::<WebGl2RenderingContext>().expect("7");

        let vertex_shader = compile_shader(&context, WebGl2RenderingContext::VERTEX_SHADER,
        "#version 300 es

        in vec4 position;

        uniform vec2 screen_size;

        out vec2 view_offset;

        void main() {
            vec2 view_mod = vec2(max(screen_size.x/screen_size.y, 1.0), max(screen_size.y/screen_size.x, 1.0));

            gl_Position = position;
            view_offset = position.xy * view_mod;
        }
        ");

        let fragment_shader = compile_shader(&context, WebGl2RenderingContext::FRAGMENT_SHADER,
        "#version 300 es

        precision highp float;
        precision highp int;

        uniform vec2 view;
        uniform float zoom;
        uniform int max_iterations;

        in vec2 view_offset;

        out highp uint escape_time;

        void main() { 
            bool escapes = false;

            float c_real = view.x + view_offset.x * 1.0/zoom;
            float c_img = view.y + view_offset.y * 1.0/zoom;

            float z_real = 0.0;
            float z_img = 0.0;

            for (int i = 0; i < 1000000000; i += 1) {
                if (i >= max_iterations) {
                    break;
                }
                float new_real = z_real * z_real - z_img * z_img;

                z_img = 2.0 * z_real * z_img;
                z_real = new_real;

                z_real += c_real;
                z_img += c_img;

                if (z_real * z_real + z_img * z_img > 4.0) {
                    escapes = true;
                    escape_time = uint(i);
                    break;
                }
            }
            if (!escapes) {
                escape_time = uint(max_iterations);
            }
        }
        ");

        let program = link_program(&context, &vertex_shader, &fragment_shader);
        context.use_program(Some(&program));

        WebGLRenderer {
            canvas,
            context,
            program,
        }
    }
}

impl CalculateEscapeTimes for WebGLRenderer {
    fn calculate_escape_times(&self, state: &RenderingState) -> Vec<u32> {
        self.context.use_program(Some(&self.program));

        self.canvas.set_width(state.screen_width);
        self.canvas.set_height(state.screen_height);
        self.context.viewport(0, 0, state.screen_width as i32, state.screen_height as i32);

        let view_loc = self.context.get_uniform_location(&self.program, "view").unwrap();
        let zoom_loc = self.context.get_uniform_location(&self.program, "zoom").unwrap();
        let screen_size_loc = self.context.get_uniform_location(&self.program, "screen_size").unwrap();
        let max_iterations_loc = self.context.get_uniform_location(&self.program, "max_iterations").unwrap();

        self.context.uniform1fv_with_f32_array(Some(&zoom_loc), &[state.zoom as f32]);
        self.context.uniform2fv_with_f32_array(Some(&view_loc), &[state.view_x as f32, state.view_y as f32]);
        self.context.uniform2fv_with_f32_array(Some(&screen_size_loc), &[state.screen_width as f32, state.screen_height as f32]);
        self.context.uniform1iv_with_i32_array(Some(&max_iterations_loc), &mut [state.max_iterations as i32]);

        let vertices: [f32; 12] = [
            -1.0, -1.0,
            -1.0, 1.0,
            1.0, -1.0,
            -1.0, 1.0,
            1.0, -1.0,
            1.0, 1.0,
        ];

        let memory_buffer = wasm_bindgen::memory()
            .dyn_into::<WebAssembly::Memory>().expect("8")
            .buffer();
        let vertices_location = vertices.as_ptr() as u32 / 4;
        let vert_array = js_sys::Float32Array::new(&memory_buffer)
            .subarray(vertices_location, vertices_location + vertices.len() as u32);

        let buffer = self.context.create_buffer().expect("9");
        self.context.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&buffer));
        self.context.buffer_data_with_array_buffer_view(
            WebGl2RenderingContext::ARRAY_BUFFER,
            &vert_array,
            WebGl2RenderingContext::STATIC_DRAW,
        );

        self.context.vertex_attrib_pointer_with_i32(0, 2, WebGl2RenderingContext::FLOAT, false, 0, 0);
        self.context.enable_vertex_attrib_array(0);

        //self.context.clear_color(0.0, 0.0, 0.0, 1.0);
        //self.context.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);

        let target_texture = self.context.create_texture().unwrap();

        self.context.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&target_texture));

        let mut texture_data = Uint32Array::new_with_length(state.screen_width * state.screen_height).fill(123,0,1000);

        self.context.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_array_buffer_view_and_src_offset(
            WebGl2RenderingContext::TEXTURE_2D,
            0,
            WebGl2RenderingContext::R32UI as i32,
            state.screen_width as i32,
            state.screen_height as i32,
            0,
            WebGl2RenderingContext::RED_INTEGER,
            WebGl2RenderingContext::UNSIGNED_INT,
            &mut texture_data,
            0
        ).expect("Failed to create texture");

        let framebuffer = self.context.create_framebuffer().unwrap();
        self.context.bind_framebuffer(WebGl2RenderingContext::FRAMEBUFFER, Some(&framebuffer));

        let attachment_point = WebGl2RenderingContext::COLOR_ATTACHMENT0;
        self.context.framebuffer_texture_2d(WebGl2RenderingContext::FRAMEBUFFER, attachment_point, WebGl2RenderingContext::TEXTURE_2D, Some(&target_texture), 0);

        self.context.draw_arrays(
            WebGl2RenderingContext::TRIANGLES,
            0,
            (vertices.len() / 2) as i32,
        );
        self.context.read_pixels_with_opt_array_buffer_view(0, 0, state.screen_width as i32, state.screen_height as i32, WebGl2RenderingContext::RED_INTEGER, WebGl2RenderingContext::UNSIGNED_INT, Some(&texture_data)).unwrap();
        let mut ret_val = vec![1_u32; texture_data.length() as usize];
        texture_data.copy_to(&mut ret_val);
        ret_val
    }
}

pub fn init() {
    //let mut renderer = WebGLRenderer::new();
    //renderer.calculate_escape_times(&RenderingState{view_x: 0.0, view_y: 0.0, zoom: 1.0, max_iterations: 300}, &CanvasBuffer{width: 1920, height: 1080, data: vec![]});
}

fn compile_shader(
    context: &WebGl2RenderingContext,
    shader_type: u32,
    source: &str) -> WebGlShader {
    let shader = context.create_shader(shader_type).expect("10");

    context.shader_source(&shader, source);
    context.compile_shader(&shader);

    if context.get_shader_parameter(&shader, WebGl2RenderingContext::COMPILE_STATUS).as_bool().unwrap_or(false) {
        return shader;
    }
    else {
        panic!("Couldn't compile shader! {}", context.get_shader_info_log(&shader).unwrap_or("unknown".into()));
    };
}

fn link_program(
    context: &WebGl2RenderingContext,
    vertex_shader: &WebGlShader,
    fragment_shader: &WebGlShader,
) -> WebGlProgram {
    let program = context.create_program().expect("11");

    context.attach_shader(&program, vertex_shader);
    context.attach_shader(&program, fragment_shader);
    context.link_program(&program);

    if context.get_program_parameter(&program, WebGl2RenderingContext::LINK_STATUS).as_bool().unwrap_or(false) {
        return program;
    }
    else {
        panic!("Couldn't compile program!");
    }
}