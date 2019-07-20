use std::mem::swap;
use std::sync::Mutex;

#[macro_use]
extern crate lazy_static;

lazy_static! {
    static ref RENDERING_STATE: Mutex<RenderingState> = Mutex::new(RenderingState {
        zoom: float_to_bigint(1.0, 1),
        view_x: float_to_bigint(-0.3, 1),
        view_y: float_to_bigint(0.0, 1),
        max_iterations: 30,
        screen_width: 0,
        screen_height: 0,
        digits: 1,
        use_gpu: false,
        requesting_render: false,
    });
}

lazy_static! {
    static ref CANVAS_BUFFER: Mutex<CanvasBuffer> = Mutex::new(CanvasBuffer {
        width: 0,
        height: 0,
        data: vec![0; 0],
    });
}

mod utils;
//mod webgl;
mod cpu;

use cpu::CPURenderer;
//use webgl::WebGLRenderer;
use num::BigInt;
//use palette::{Hsl, Srgb};
use std::os::raw::c_void;
use wasm_bindgen::prelude::*;

const BITS_PER_DIGIT: u32 = 16;
const ZOOM_FACTOR: f64 = 1.125;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
//#[cfg(feature = "wee_alloc")]
//#[global_allocator]
//static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen(start)]
pub fn rust_main() {
    utils::set_panic_hook();
    //panic!("panic hook");
    render_loop();
    //webgl::init();
}

#[wasm_bindgen]
pub struct CanvasBuffer {
    pub width: usize,
    pub height: usize,
    data: Vec<u8>,
}

#[wasm_bindgen]
impl CanvasBuffer {
    pub fn get_ptr(&mut self) -> *mut c_void {
        self.data.as_mut_ptr() as *mut c_void
    }
}

#[derive(Clone)]
struct RenderingState {
    zoom: BigInt,
    view_x: BigInt,
    view_y: BigInt,
    max_iterations: u32,
    screen_width: u32,
    screen_height: u32,
    digits: u32,
    use_gpu: bool,
    requesting_render: bool,
}

#[wasm_bindgen]
pub fn request_render() {
    let mut state = RENDERING_STATE.lock().unwrap();
    state.requesting_render = true;
}

#[wasm_bindgen]
pub fn resize_canvas(new_width: usize, new_height: usize) {
    let mut buffer = CANVAS_BUFFER.lock().unwrap();
    buffer.width = new_width;
    buffer.height = new_height;
    buffer.data.clear();
    buffer.data.resize(new_width * new_height * 4, 0);
}

#[wasm_bindgen]
pub fn get_canvas_buffer() -> CanvasBuffer {
    let mut full_buffer = CANVAS_BUFFER.lock().unwrap();
    let mut empty_buffer = CanvasBuffer {
        width: 0,
        height: 0,
        data: vec![],
    };
    swap(&mut (*full_buffer), &mut empty_buffer);
    empty_buffer
}

trait CalculateEscapeTimes {
    fn calculate_escape_times(&self, rendering_state: &RenderingState) -> Vec<u32>;
}

#[wasm_bindgen]
pub fn change_zoom(zoom_in: bool) {
    let mut state = RENDERING_STATE.lock().unwrap();
    let factor = float_to_bigint(
        if zoom_in {
            1.0 / ZOOM_FACTOR
        } else {
            ZOOM_FACTOR
        },
        state.digits,
    );
    state.zoom = mul_bigint(&state.zoom, &factor, state.digits);
}

#[wasm_bindgen]
pub fn change_view(delta_x: f64, delta_y: f64) {
    let mut state = RENDERING_STATE.lock().unwrap();
    let delta_x = float_to_bigint(delta_x, state.digits);
    let delta_y = float_to_bigint(delta_y, state.digits);
    let delta_view_x = mul_bigint(&delta_x, &state.zoom, state.digits);
    let delta_view_y = mul_bigint(&delta_y, &state.zoom, state.digits);
    state.view_x += delta_view_x;
    state.view_y += delta_view_y;
}

#[wasm_bindgen]
pub fn set_max_iterations(new_value: u32) {
    let mut state = RENDERING_STATE.lock().unwrap();
    state.max_iterations = new_value;
}

fn render_loop() {
    std::thread::spawn(move || loop {
        let state_lock = RENDERING_STATE.lock().unwrap();
        let state = state_lock.clone();
        drop(state_lock);
        if state.requesting_render {
            render(&state);
            web_sys::window()
                .unwrap()
                .post_message(&JsValue::from_str("render_ready"), "*")
                .unwrap();
        }
    });
}

fn render(state: &RenderingState) {
    let mut new_buffer = CanvasBuffer {
        width: state.screen_width as usize,
        height: state.screen_height as usize,
        data: vec![0; (state.screen_width * state.screen_height * 4) as usize],
    };
    let escape_times = CPURenderer::new().calculate_escape_times(&state);
    calculate_colors(&mut new_buffer, &state, escape_times);
    let mut buffer = CANVAS_BUFFER.lock().unwrap();
    *buffer = new_buffer;
}

fn calculate_colors(buffer: &mut CanvasBuffer, state: &RenderingState, escape_times: Vec<u32>) {
    /*let mut total = 0.0;
      let mut histogram = vec![0_u32; state.max_iterations as usize + 1];
    for escape_time in escape_times.iter() {
        if *escape_time < state.max_iterations {
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
            if escape_time < state.max_iterations {
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

fn mul_bigint(x: &BigInt, y: &BigInt, digits: u32) -> BigInt {
    (x * y) >> ((digits - 1) * BITS_PER_DIGIT) as usize
}

fn float_to_bigint(x: f64, digits: u32) -> BigInt {
    BigInt::from((x * f64::powi(2.0, BITS_PER_DIGIT as i32)) as i128)
        << ((digits - 2) * BITS_PER_DIGIT) as usize
}
