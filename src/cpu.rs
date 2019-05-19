use crate::{CalculateEscapeTimes, RenderingState, CanvasBuffer};
use num::Complex;

pub struct CPURenderer {

}

impl CalculateEscapeTimes for CPURenderer {
    fn calculate_escape_times(&self, state: &RenderingState, buffer: &CanvasBuffer) -> Vec<u32> {
        let mut width = 2.0 / state.zoom;
        let mut height = 2.0 / state.zoom;

        if buffer.width > buffer.height {
            width *= buffer.width as f64 / buffer.height as f64;
        } else {
            height *= buffer.height as f64 / buffer.width as f64
        }
        
        let pos_x = state.view_x - width / 2.0;
        let pos_y = state.view_y - height / 2.0;

        let mut escape_times = vec![0_u32; buffer.width*buffer.height];

        for y in 0..buffer.height {
            for x in 0..buffer.width {
                let real = pos_x + x as f64 * (width / buffer.width as f64);
                let img = pos_y + y as f64 * (height / buffer.height as f64);

                let c = Complex::new(real, img);
                let mut z = Complex::new(0.0, 0.0);

                escape_times[y * buffer.width + x] = state.max_iterations;

                for i in 0..state.max_iterations {
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
}