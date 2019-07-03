mod utils;
//mod webgl;
mod cpu;

use cpu::CPURenderer;
//use webgl::WebGLRenderer;
use std::os::raw::c_void;
use wasm_bindgen::prelude::*;
use palette::{ Hsl, Srgb };
use num::BigInt;

const BITS_PER_DIGIT: u32 = 16;
const ZOOM_FACTOR: f64 = 1.125;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
//#[cfg(feature = "wee_alloc")]
//#[global_allocator]
//static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen(start)]
pub fn initialize() {
    //webgl::init();
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
        self.data.clear();
        self.data.resize(new_width * new_height * 4, 0);
    }

    pub fn get_buffer(&mut self) -> *mut c_void {
        self.data.as_mut_ptr() as *mut c_void
    }

    pub fn get_buffer_length(&self) -> usize {
        self.data.len()
    }
}

trait CalculateEscapeTimes {
    fn calculate_escape_times(&self, rendering_state: &RenderingState) -> Vec<u32>;
}

struct RenderingState {
    zoom: BigInt,
    view_x: BigInt,
    view_y: BigInt,
    max_iterations: u32,
    screen_width: u32,
    screen_height: u32,
    digits: u32,
}

#[wasm_bindgen]
pub struct MandelbrotRenderer {
    state: RenderingState,
    escape_time_renderer: Box<dyn CalculateEscapeTimes>,
}

#[wasm_bindgen]
impl MandelbrotRenderer {
    pub fn new(zoom: f64, view_x: f64, view_y: f64, max_iterations: u32, digits: u32) -> MandelbrotRenderer {
    utils::set_panic_hook();
        MandelbrotRenderer {
            state: RenderingState {
                zoom: float_to_bigint(zoom, digits),
                view_x: float_to_bigint(view_x, digits),
                view_y: float_to_bigint(view_y, digits),
                max_iterations,
                screen_width: 0,
                screen_height: 0,
                digits: if digits == 0 { 1 } else { digits },
            },
            escape_time_renderer: Box::new(CPURenderer::new()),
        }
    }

    pub fn change_zoom(&mut self, zoom_in: bool) {
        let factor = float_to_bigint(if zoom_in {1.0/ZOOM_FACTOR} else {ZOOM_FACTOR}, self.state.digits);
        self.state.zoom = mul_bigint(&self.state.zoom, &factor, self.state.digits);
    }

    pub fn change_view(&mut self, delta_x: f64, delta_y: f64) {
        let delta_x = float_to_bigint(delta_x, self.state.digits);
        let delta_y = float_to_bigint(delta_y, self.state.digits);
        self.state.view_x += mul_bigint(&delta_x, &self.state.zoom, self.state.digits);
        self.state.view_y += mul_bigint(&delta_y, &self.state.zoom, self.state.digits);
    }

    pub fn set_max_iterations(&mut self, new_value: u32) {
        self.state.max_iterations = new_value;
    }

    pub fn render(&mut self, buffer: &mut CanvasBuffer) {
        self.state.screen_width = buffer.width as u32;
        self.state.screen_height = buffer.height as u32;
        let escape_times = self.escape_time_renderer.calculate_escape_times(&self.state);
        self.calculate_colors(buffer, escape_times);
    }

    fn calculate_colors(&self, buffer: &mut CanvasBuffer, escape_times: Vec<u32>) {
        /*let mut total = 0.0;
        let mut histogram = vec![0_u32; self.state.max_iterations as usize + 1];

        for escape_time in escape_times.iter() {
            if *escape_time < self.state.max_iterations {
                total += 1.0;
            }
            histogram[*escape_time as usize] += 1;
        }*/

        for y in 0..buffer.height {
            for x in 0..buffer.width {
                let escape_time = escape_times[y * buffer.width + x];
                /*let mut hue = 0.0;
                for i in 0..=escape_time {
                    hue += histogram[i as usize] as f64 / total;
                }*/


                if escape_time < self.state.max_iterations {
                    //let color = Srgb::from(Hsl::new(hue * 360.0, 1.0, 0.5));

                    //buffer.data[(y * buffer.width + x) * 4 + 0] = (color.red * 255.0) as u8;
                    //buffer.data[(y * buffer.width + x) * 4 + 1] = (color.green * 255.0) as u8;
                    //buffer.data[(y * buffer.width + x) * 4 + 2] = (color.blue * 255.0) as u8;
                    //buffer.data[(y * buffer.width + x) * 4 + 3] = 255;

                    buffer.data[(y * buffer.width + x) * 4 + 0] = 0;
                    buffer.data[(y * buffer.width + x) * 4 + 1] = 255 as u8;
                    buffer.data[(y * buffer.width + x) * 4 + 2] = 0;
                    buffer.data[(y * buffer.width + x) * 4 + 3] = 255;
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

fn mul_bigint(x: &BigInt, y: &BigInt, digits: u32) -> BigInt {
    (x * y) >> ((digits-1) * BITS_PER_DIGIT) as usize
}

fn float_to_bigint(x: f64, digits: u32) -> BigInt {
    BigInt::from((x * f64::powi(2.0, BITS_PER_DIGIT as i32)) as i128) << ((digits-2) * BITS_PER_DIGIT) as usize
}