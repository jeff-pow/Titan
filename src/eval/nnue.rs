use super::{Block, INPUT_SIZE, NET};
#[cfg(feature = "simd")]
use super::{CHUNK_SIZE, REQUIRED_ITERS};
#[cfg(feature = "simd")]
use std::arch::x86_64::{
    __m512i, _mm512_dpwssd_epi32, _mm512_loadu_epi16, _mm512_reduce_add_epi32, _mm512_set1_epi16,
    _mm512_setzero_si512,
};

use crate::board::board::Board;
/**
* When changing activation functions, both the normalization factor and QA may need to change
* alongside changing the crelu calls to screlu in simd and serial code.
*/
const QA: i32 = 181; // CHANGES WITH NET QUANZIZATION
const QB: i32 = 64;
const QAB: i32 = QA * QB;
const NORMALIZATION_FACTOR: i32 = QA; // CHANGES WITH SCRELU/CRELU ACTIVATION
const RELU_MIN: i16 = 0;
const RELU_MAX: i16 = QA as i16;

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

#[cfg(not(feature = "simd"))]
fn screlu(i: i16) -> i32 {
    crelu(i) * crelu(i)
}

#[cfg(not(feature = "simd"))]
fn crelu(i: i16) -> i32 {
    i32::from(i.clamp(RELU_MIN, RELU_MAX))
}

#[cfg(feature = "simd")]
unsafe fn clipped_relu(i: __m512i) -> __m512i {
    use std::arch::x86_64::{_mm512_max_epi16, _mm512_min_epi16};
    let min = _mm512_set1_epi16(RELU_MIN);
    let max = _mm512_set1_epi16(RELU_MAX);

    _mm512_min_epi16(_mm512_max_epi16(i, min), max)
}

#[cfg(feature = "simd")]
unsafe fn squared_crelu(i: __m512i) -> __m512i {
    use std::arch::x86_64::_mm512_mullo_epi16;

    let clamp = clipped_relu(i);
    _mm512_mullo_epi16(clamp, clamp)
}

#[inline(never)]
unsafe fn flatten(acc: &Block, weights: &Block) -> i32 {
    #[cfg(feature = "simd")]
    {
        let mut sum = _mm512_setzero_si512();
        for i in 0..REQUIRED_ITERS {
            let us_vector = _mm512_loadu_epi16(&acc[i * CHUNK_SIZE]);
            let crelu_result = squared_crelu(us_vector);
            let weights = _mm512_loadu_epi16(&weights[i * CHUNK_SIZE]);
            sum = _mm512_dpwssd_epi32(sum, crelu_result, weights);
        }
        _mm512_reduce_add_epi32(sum)
    }

    #[cfg(not(feature = "simd"))]
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
