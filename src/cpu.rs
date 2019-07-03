use crate::{CalculateEscapeTimes, RenderingState, mul_bigint, float_to_bigint};
use num::BigInt;
pub struct CPURenderer {}

impl CPURenderer {
    pub fn new() -> CPURenderer {
        CPURenderer{}
    }
}

impl CalculateEscapeTimes for CPURenderer {
    fn calculate_escape_times(&self, state: &RenderingState) -> Vec<u32> {
        let mut width = state.zoom.clone() << 1;
        let mut height = state.zoom.clone() << 1;

        if state.screen_width > state.screen_height {
            width = mul_bigint(&width, &float_to_bigint(state.screen_width as f64 / state.screen_height as f64, state.digits), state.digits)
        } else {
            height = mul_bigint(&height, &float_to_bigint(state.screen_height as f64 / state.screen_width as f64, state.digits), state.digits);
        }
        
        let pos_x = state.view_x.clone() - (width.clone() >> 1);
        let pos_y = state.view_y.clone() - (height.clone() >> 1);

        let mut escape_times = vec![0_u32; state.screen_width as usize * state.screen_height as usize];

        for y in 0..state.screen_height as usize {
            for x in 0..state.screen_width as usize {
                let c_real = pos_x.clone() + x * (width.clone() / state.screen_width);
                let c_img = pos_y.clone() + y * (height.clone() / state.screen_height);

                let mut z_real = BigInt::from(0);
                let mut z_img = BigInt::from(0);

                escape_times[y * state.screen_width as usize + x] = state.max_iterations;

                for i in 0..state.max_iterations {
                    let new_z_real = mul_bigint(&z_real, &z_real, state.digits) - mul_bigint(&z_img, &z_img, state.digits) + c_real.clone();
                    z_img = (mul_bigint(&z_real, &z_img, state.digits) << 1) + c_img.clone();
                    z_real = new_z_real;


                    if mul_bigint(&z_real, &z_real, state.digits) + mul_bigint(&z_img, &z_img, state.digits) > float_to_bigint(4.0, state.digits) {
                        escape_times[y * state.screen_width as usize + x] = i;
                        break;
                    }
                }
            }
        }

        escape_times
    }
}

mod tests {
    use super::*;

    #[test]
    fn test_escape_times() {
        let renderer = CPURenderer {};
        let digits = 2;
        let state = RenderingState {
                zoom: float_to_bigint(1.0, digits),
                view_x: float_to_bigint(-0.3, digits),
                view_y: float_to_bigint(0.0, digits),
                max_iterations: 30,
                screen_width: 10,
                screen_height: 10,
                digits
            };
        println!("{:?}", renderer.calculate_escape_times(&state));
    }
}