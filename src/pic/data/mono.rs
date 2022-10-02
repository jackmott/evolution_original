use rand::prelude::*;
use rand::rngs::StdRng;
use std::collections::HashMap;
use std::sync::Arc;

use crate::parser::aptnode::APTNode;
use crate::pic::actual_picture::ActualPicture;
use crate::pic::coordinatesystem::{cartesian_to_polar, CoordinateSystem};
use crate::pic::data::PicData;
use crate::pic::pic::Pic;
use crate::vm::stackmachine::StackMachine;

use rayon::prelude::*;
use simdeez::Simd;

#[derive(Clone, Debug, PartialEq)]
pub struct MonoData {
    pub c: APTNode,
    pub coord: CoordinateSystem,
}

impl PicData for MonoData {
    fn new(min: usize, max: usize, video: bool, rng: &mut StdRng, pic_names: &Vec<&String>) -> Pic {
        let (tree, coord) = APTNode::generate_tree(rng.gen_range(min..max), video, rng, pic_names);
        Pic::Mono(MonoData { c: tree, coord })
    }
    fn get_rgba8<S: Simd>(
        &self,
        threaded: bool,
        pics: Arc<HashMap<String, ActualPicture>>,
        w: usize,
        h: usize,
        t: f32,
    ) -> Vec<u8> {
        unsafe {
            let ts = S::set1_ps(t);
            let wf = S::set1_ps(w as f32);
            let hf = S::set1_ps(h as f32);
            let vec_len = w * h * 4;
            let mut result = Vec::<u8>::with_capacity(vec_len);
            result.set_len(vec_len);
            let sm = StackMachine::<S>::build(&self.c);
            /*
            let mut min = 999999.0;
            let mut max = -99999.0;
            */

            let process = |(y_pixel, chunk): (usize, &mut [u8])| {
                let mut stack = Vec::with_capacity(sm.instructions.len());
                stack.set_len(sm.instructions.len());

                let y = S::set1_ps((y_pixel as f32 / h as f32) * 2.0 - 1.0);
                let x_step = 2.0 / (w - 1) as f32;
                let mut x = S::setzero_ps();
                for i in (0..S::VF32_WIDTH).rev() {
                    x[i] = -1.0 + (x_step * i as f32);
                }
                let x_step = S::set1_ps(x_step * S::VF32_WIDTH as f32);

                for i in (0..w * 4).step_by(S::VF32_WIDTH * 4) {
                    let v = if self.coord == CoordinateSystem::Cartesian {
                        sm.execute(&mut stack, pics.clone(), x, y, ts, wf, hf)
                    } else {
                        let (r, theta) = cartesian_to_polar::<S>(x, y);
                        sm.execute(&mut stack, pics.clone(), r, theta, ts, wf, hf)
                    };

                    for j in 0..S::VF32_WIDTH {
                        let c = if v[j] >= 0.0 { 255 } else { 0 };
                        let j4 = j * 4;
                        chunk[i + j4] = c;
                        chunk[i + 1 + j4] = c;
                        chunk[i + 2 + j4] = c;
                        chunk[i + 3 + j4] = 255 as u8;
                    }
                    x = x + x_step;
                }
            };

            if threaded {
                result.par_chunks_mut(4 * w).enumerate().for_each(process);
            } else {
                result.chunks_exact_mut(4 * w).enumerate().for_each(process);
            }
            // println!("min:{} max:{} range:{}",min,max,max-min);
            result
        }
    }
}
