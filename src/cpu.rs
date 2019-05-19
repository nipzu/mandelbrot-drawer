use crate::{CalculateEscapeTimes, RenderingState};
use num::Complex;

pub struct CPURenderer {}

impl CalculateEscapeTimes for CPURenderer {
    fn calculate_escape_times(&self, state: &RenderingState) -> Vec<u32> {
        let mut width = 2.0 / state.zoom;
        let mut height = 2.0 / state.zoom;

        if state.screen_width > state.screen_height {
            width *= state.screen_width as f64 / state.screen_height as f64;
        } else {
            height *= state.screen_height as f64 / state.screen_width as f64
        }
        
        let pos_x = state.view_x - width / 2.0;
        let pos_y = state.view_y - height / 2.0;

        let mut escape_times = vec![0_u32; state.screen_width as usize * state.screen_height as usize];

        for y in 0..state.screen_height as usize {
            for x in 0..state.screen_width as usize {
                let real = pos_x + x as f64 * (width / state.screen_width as f64);
                let img = pos_y + y as f64 * (height / state.screen_height as f64);

                let c = Complex::new(real, img);
                let mut z = Complex::new(0.0, 0.0);

                escape_times[y * state.screen_width as usize + x] = state.max_iterations;

                for i in 0..state.max_iterations {
                    z = z * z + c;
                    if z.norm_sqr() > 4.0 {
                        escape_times[y * state.screen_width as usize + x] = i;
                        break;
                    }
                }
            }
        }

        escape_times
    }
}