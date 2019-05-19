mod utils;
mod webgl;
mod cpu;

use cpu::CPURenderer;
use webgl::WebGLRenderer;
use cfg_if::cfg_if;
use std::os::raw::c_void;
use wasm_bindgen::prelude::*;
use palette::{ Hsl, Srgb };

cfg_if! {
    // When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
    // allocator.
    if #[cfg(feature = "wee_alloc")] {
        extern crate wee_alloc;
        #[global_allocator]
        static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
    }
}

#[wasm_bindgen(start)]
pub fn initialize() {
    utils::set_panic_hook();
    webgl::init();
}

#[wasm_bindgen]
pub struct CanvasBuffer {
    width: usize,
    height: usize,
    data: Vec<u8>,
}

#[wasm_bindgen]
impl CanvasBuffer {
    pub fn new(width: usize, height: usize) -> CanvasBuffer {
        CanvasBuffer {
            width,
            height,
            data: vec![0; width * height],
        }
    }

    pub fn resize(&mut self, new_width: usize, new_height: usize) {
        self.width = new_width;
        self.height = new_height;
        self.data.resize(new_width * new_height * 4, 0);
        for (i, data) in self.data.iter_mut().enumerate() {
            match i % 4 {
                3 => *data = 255,
                0 => *data = 0,
                1 => *data = 0,
                2 => *data = 0,
                _ => (),
            }
        }
    }

    pub fn get_buffer(&mut self) -> *mut c_void {
        self.data.as_mut_ptr() as *mut c_void
    }

    pub fn get_buffer_length(&self) -> usize {
        self.data.len()
    }
}

trait CalculateEscapeTimes {
    fn calculate_escape_times(&self, rendering_state: &RenderingState, buffer: &CanvasBuffer) -> Vec<u32>;
}

struct RenderingState {
    zoom: f64,
    view_x: f64,
    view_y: f64,
    max_iterations: u32,
}

#[wasm_bindgen]
pub struct MandelbrotRenderer {
    state: RenderingState,
    escape_time_renderer: Box<dyn CalculateEscapeTimes>,
}

#[wasm_bindgen]
impl MandelbrotRenderer {
    pub fn new(zoom: f64, view_x: f64, view_y: f64, max_iterations: u32) -> MandelbrotRenderer {
        MandelbrotRenderer {
            state: RenderingState {
                zoom,
                view_x,
                view_y,
                max_iterations,
            },
            escape_time_renderer: Box::new(WebGLRenderer::new()),
        }
    }

    pub fn change_zoom(&mut self, factor: f64) {
        self.state.zoom *= factor;
    }

    pub fn change_view(&mut self, delta_x: f64, delta_y: f64) {
        self.state.view_x += delta_x * 1.0 / self.state.zoom;
        self.state.view_y += delta_y * 1.0 / self.state.zoom;
    }

    pub fn set_max_iterations(&mut self, new_value: u32) {
        self.state.max_iterations = new_value;
    }

    pub fn render(&mut self, buffer: &mut CanvasBuffer) {
        let escape_times = self.escape_time_renderer.calculate_escape_times(&self.state, &buffer);
        //self.calculate_colors(buffer, escape_times);
    }
/*
    fn calculate_escape_times(&self, buffer: &CanvasBuffer) -> Vec<u32> {
        let mut width = 2.0 / self.zoom;
        let mut height = 2.0 / self.zoom;

        if buffer.width > buffer.height {
            width *= buffer.width as f64 / buffer.height as f64;
        } else {
            height *= buffer.height as f64 / buffer.width as f64
        }
        
        let pos_x = self.view_x - width / 2.0;
        let pos_y = self.view_y - height / 2.0;

        let mut escape_times = vec![0_u32; buffer.width*buffer.height];

        for y in 0..buffer.height {
            for x in 0..buffer.width {
                let real = pos_x + x as f64 * (width / buffer.width as f64);
                let img = pos_y + y as f64 * (height / buffer.height as f64);

                let c = Complex::new(real, img);
                let mut z = Complex::new(0.0, 0.0);

                escape_times[y * buffer.width + x] = self.max_iterations;

                for i in 0..self.max_iterations {
                    z = z * z + c;
                    if z.norm_sqr() > 4.0 {
                        escape_times[y * buffer.width + x] = i;
                        break;
                    }
                }
            }
        }

        escape_times
    }
*/
    fn calculate_colors(&self, buffer: &mut CanvasBuffer, escape_times: Vec<u32>) {
        let mut total = 0.0;
        let mut histogram = vec![0_u32; self.state.max_iterations as usize + 1];

        for escape_time in escape_times.iter() {
            if *escape_time < self.state.max_iterations {
                total += 1.0;
            }
            histogram[*escape_time as usize] += 1;
        }

        for y in 0..buffer.height {
            for x in 0..buffer.width {
                let mut hue = 0.0;
                let escape_time = escape_times[y * buffer.width + x];
                for i in 0..=escape_time {
                    hue += histogram[i as usize] as f64 / total;
                }


                if escape_time < self.state.max_iterations {
                    let color = Srgb::from(Hsl::new(hue * 360.0, 1.0, 0.5));

                    buffer.data[(y * buffer.width + x) * 4 + 0] = (color.red * 255.0) as u8;
                    buffer.data[(y * buffer.width + x) * 4 + 1] = (color.green * 255.0) as u8;
                    buffer.data[(y * buffer.width + x) * 4 + 2] = (color.blue * 255.0) as u8;
                    buffer.data[(y * buffer.width + x) * 4 + 3] = 255;

                    //buffer.data[(y * buffer.width + x) * 4 + 0] = 0;
                    //buffer.data[(y * buffer.width + x) * 4 + 1] = (hue * 255.0) as u8;
                    //buffer.data[(y * buffer.width + x) * 4 + 2] = 0;
                    //buffer.data[(y * buffer.width + x) * 4 + 3] = 255;
                } else {
                    buffer.data[(y * buffer.width + x) * 4 + 0] = 0;
                    buffer.data[(y * buffer.width + x) * 4 + 1] = 0;
                    buffer.data[(y * buffer.width + x) * 4 + 2] = 0;
                    buffer.data[(y * buffer.width + x) * 4 + 3] = 255;
                }
            }
        }
    }
}