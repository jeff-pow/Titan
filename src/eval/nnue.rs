use super::{Block, INPUT_SIZE, NET};

use crate::board::board::Board;
/**
* When changing activation functions, both the normalization factor and QA may need to change
* alongside changing the crelu calls to screlu in simd and serial code.
*/
const QA: i32 = 181; // CHANGES WITH NET QUANZIZATION
const QB: i32 = 64;
const QAB: i32 = QA * QB;
const NORMALIZATION_FACTOR: i32 = QA; // CHANGES WITH SCRELU/CRELU ACTIVATION
pub(super) const RELU_MIN: i16 = 0;
pub(super) const RELU_MAX: i16 = QA as i16;

const SCALE: i32 = 400;

#[derive(Clone, Debug)]
#[repr(C, align(64))]
pub(super) struct Network {
    pub feature_weights: [Block; INPUT_SIZE],
    pub feature_bias: Block,
    pub output_weights: [Block; 2],
    pub output_bias: i16,
}

impl Board {
    #[allow(clippy::assertions_on_constants)]
    pub fn evaluate(&self) -> i32 {
        let (us, them) = (&self.accumulator.0[self.to_move], &self.accumulator.0[!self.to_move]);
        let weights = &NET.output_weights;

        let mut output = 0;
        unsafe {
            let us = flatten(us, &weights[0]);
            let them = flatten(them, &weights[1]);

            output += us + them;
            output /= NORMALIZATION_FACTOR;
        }
        let a = (i32::from(NET.output_bias) + output) * SCALE / QAB;
        assert!(i16::MIN as i32 <= a && a <= i16::MAX as i32);
        a
    }
}

#[cfg(all(not(feature = "avx2"), not(feature = "avx512")))]
fn screlu(i: i16) -> i32 {
    crelu(i) * crelu(i)
}

#[cfg(all(not(feature = "avx2"), not(feature = "avx512")))]
fn crelu(i: i16) -> i32 {
    i32::from(i.clamp(RELU_MIN, RELU_MAX))
}

unsafe fn flatten(acc: &Block, weights: &Block) -> i32 {
    #[cfg(feature = "avx512")]
    {
        use super::simd::avx512;
        avx512::flatten(acc, weights)
    }
    #[cfg(all(not(feature = "avx512"), feature = "avx2"))]
    {
        use super::simd::avx2;
        avx2::flatten(acc, weights)
    }
    #[cfg(all(not(feature = "avx2"), not(feature = "avx512")))]
    {
        let mut sum = 0;
        for (&i, &w) in acc.iter().zip(weights) {
            sum += screlu(i) * i32::from(w);
        }
        sum
    }
}

#[cfg(test)]
mod nnue_tests {
    use std::{hint::black_box, time::Instant};

    use crate::board::fen::{build_board, STARTING_FEN};

    #[test]
    fn inference_benchmark() {
        let board = build_board(STARTING_FEN);
        let start = Instant::now();
        let iters = 10_000_000_u128;
        for _ in 0..iters {
            black_box(board.evaluate());
        }
        let duration = start.elapsed().as_nanos();
        println!("{} ns per iter", duration / iters);
        dbg!(duration / iters);
    }
}
