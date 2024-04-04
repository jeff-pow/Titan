use super::{Align64, Block, INPUT_SIZE};

use crate::types::{pieces::Color, square::Square};
/**
* When changing activation functions, both the normalization factor and QA may need to change
* alongside changing the crelu calls to screlu in simd and serial code.
*/
const QA: i32 = 255; // CHANGES WITH NET QUANZIZATION
const QB: i32 = 64;
pub(super) const QAB: i32 = QA * QB;
pub(super) const NORMALIZATION_FACTOR: i32 = QA; // CHANGES WITH SCRELU/CRELU ACTIVATION
pub(super) const RELU_MIN: i16 = 0;
pub(super) const RELU_MAX: i16 = QA as i16;

pub(super) const SCALE: i32 = 400;

const NUM_BUCKETS: usize = 4;

#[rustfmt::skip]
pub static BUCKETS: [usize; 64] = [
    0, 0, 1, 1, 5, 5, 4, 4,
    2, 2, 2, 2, 6, 6, 6, 6,
    3, 3, 3, 3, 7, 7, 7, 7,
    3, 3, 3, 3, 7, 7, 7, 7,
    3, 3, 3, 3, 7, 7, 7, 7,
    3, 3, 3, 3, 7, 7, 7, 7,
    3, 3, 3, 3, 7, 7, 7, 7,
    3, 3, 3, 3, 7, 7, 7, 7,
];

#[derive(Debug)]
#[repr(C, align(64))]
pub(super) struct Network {
    pub feature_weights: [Align64<Block>; INPUT_SIZE * NUM_BUCKETS],
    pub feature_bias: Align64<Block>,
    pub output_weights: [Align64<Block>; 2],
    pub output_bias: i16,
}

impl Network {
    pub fn bucket(&self, mut king: Square, view: Color) -> usize {
        if view == Color::Black {
            king = king.flip_vertical();
        }
        BUCKETS[king]
    }
}

#[cfg(all(not(target_feature = "avx2"), not(feature = "avx512")))]
fn screlu(i: i16) -> i32 {
    crelu(i) * crelu(i)
}

#[cfg(all(not(target_feature = "avx2"), not(feature = "avx512")))]
fn crelu(i: i16) -> i32 {
    i32::from(i.clamp(RELU_MIN, RELU_MAX))
}

pub(super) fn flatten(acc: &Block, weights: &Block) -> i32 {
    #[cfg(feature = "avx512")]
    {
        use super::simd::avx512;
        unsafe { avx512::flatten(acc, weights) }
    }
    #[cfg(all(not(feature = "avx512"), target_feature = "avx2"))]
    {
        use super::simd::avx2;
        unsafe { avx2::flatten(acc, weights) }
    }
    #[cfg(all(not(target_feature = "avx2"), not(feature = "avx512")))]
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

    use crate::{board::Board, fen::STARTING_FEN};

    #[test]
    fn inference_benchmark() {
        let board = Board::from_fen(STARTING_FEN);
        let acc = board.new_accumulator();
        let start = Instant::now();
        let iters = 10_000_000_u128;
        for _ in 0..iters {
            black_box(acc.scaled_evaluate(&board));
        }
        let duration = start.elapsed().as_nanos();
        println!("{} ns per iter", duration / iters);
        dbg!(duration / iters);
    }
}
