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
        let (tree, coord) =
            APTNode::create_random_tree(rng.gen_range(min..max), video, rng, pic_names);
        Pic::Mono(MonoData { c: tree, coord })
    }
    fn get_rgba8<S: Simd>(
        &self,
        threaded: bool,
        pics: Arc<HashMap<String, ActualPicture>>,
        w: u32,
        h: u32,
        t: f32,
    ) -> Vec<u8> {
        unsafe {
            let ts = S::set1_ps(t);
            let wf = S::set1_ps(w as f32);
            let hf = S::set1_ps(h as f32);
            let vec_len = (w * h * 4) as usize;
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
                let chunk_len = chunk.len();
                for i in (0..w * 4).step_by(S::VF32_WIDTH * 4) {
                    let v = if self.coord == CoordinateSystem::Cartesian {
                        sm.execute(&mut stack, pics.clone(), x, y, ts, wf, hf)
                    } else {
                        let (r, theta) = cartesian_to_polar::<S>(x, y);
                        sm.execute(&mut stack, pics.clone(), r, theta, ts, wf, hf)
                    };

                    for j in 0..S::VF32_WIDTH {
                        let j4: usize = j * 4;
                        let ij4 = i as usize + j4;
                        if ij4 >= chunk_len {
                            break;
                        }
                        let c = if v[j] >= 0.0 { 255 } else { 0 };
                        chunk[ij4] = c;
                        chunk[ij4 + 1] = c;
                        chunk[ij4 + 2] = c;
                        chunk[ij4 + 3] = 255 as u8;
                    }
                    x = x + x_step;
                }
            };

            if threaded {
                result
                    .par_chunks_mut(4 * w as usize)
                    .enumerate()
                    .for_each(process);
            } else {
                result
                    .chunks_exact_mut(4 * w as usize)
                    .enumerate()
                    .for_each(process);
            }
            // println!("min:{} max:{} range:{}",min,max,max-min);
            result
        }
    }
    fn simplify<S: Simd>(
        &mut self,
        pics: Arc<HashMap<String, ActualPicture>>,
        w: u32,
        h: u32,
        t: f32,
    ) {
        self.c =
            self.c
                .constant_fold::<S>(&self.coord, pics, None, None, Some(w), Some(h), Some(t));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pic_new_mono() {
        let mut rng = StdRng::from_rng(rand::thread_rng()).unwrap();
        let pic = MonoData::new(0, 60, false, &mut rng, &vec![&"eye.jpg".to_string()]);
        match &pic {
            Pic::Mono(MonoData { c, coord: _coord }) => {
                let len = c.get_children().unwrap().len();
                assert!(len > 0 && len < 60);
            }
            _ => {
                panic!("wrong type");
            }
        };
    }
}
