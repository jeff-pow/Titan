use super::{Block, INPUT_SIZE, NET};
#[cfg(feature = "simd")]
use super::{CHUNK_SIZE, REQUIRED_ITERS};
#[cfg(feature = "simd")]
use std::arch::x86_64::{
    __m256i, _mm256_dpwssd_epi32, _mm256_loadu_epi16, _mm256_set1_epi16, _mm256_setzero_si256,
};
use std::arch::x86_64::{
    _mm256_castsi256_si128, _mm256_extracti128_si256, _mm_add_epi32, _mm_cvtsi128_si32,
    _mm_shuffle_epi32, _mm_unpackhi_epi64,
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
unsafe fn clipped_relu(i: __m256i) -> __m256i {
    use std::arch::x86_64::{_mm256_max_epi16, _mm256_min_epi16};
    let min = _mm256_set1_epi16(RELU_MIN);
    let max = _mm256_set1_epi16(RELU_MAX);

    _mm256_min_epi16(_mm256_max_epi16(i, min), max)
}

#[cfg(feature = "simd")]
unsafe fn squared_crelu(i: __m256i) -> __m256i {
    use std::arch::x86_64::_mm256_mullo_epi16;

    let clamp = clipped_relu(i);
    _mm256_mullo_epi16(clamp, clamp)
}

#[inline(never)]
unsafe fn flatten(acc: &Block, weights: &Block) -> i32 {
    #[cfg(feature = "simd")]
    {
        let mut sum = _mm256_setzero_si256();
        for i in 0..REQUIRED_ITERS {
            let us_vector = _mm256_loadu_epi16(&acc[i * CHUNK_SIZE]);
            let crelu_result = squared_crelu(us_vector);
            let weights = _mm256_loadu_epi16(&weights[i * CHUNK_SIZE]);
            sum = _mm256_dpwssd_epi32(sum, crelu_result, weights);
        }
        hadd(sum)
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

#[cfg(feature = "simd")]
unsafe fn hadd(sum: __m256i) -> i32 {
    let mut xmm0;
    let mut xmm1;

    // Get the lower and upper half of the register:
    xmm0 = _mm256_castsi256_si128(sum);
    xmm1 = _mm256_extracti128_si256(sum, 1);

    // Add the lower and upper half vertically:
    xmm0 = _mm_add_epi32(xmm0, xmm1);

    // Get the upper half of the result:
    xmm1 = _mm_unpackhi_epi64(xmm0, xmm0);

    // Add the lower and upper half vertically:
    xmm0 = _mm_add_epi32(xmm0, xmm1);

    // Shuffle the result so that the lower 32-bits are directly above the second-lower 32-bits:
    xmm1 = _mm_shuffle_epi32::<0b00000001>(xmm0); // 2, 3, 0, 1

    // Add the lower 32-bits to the second-lower 32-bits vertically:
    xmm0 = _mm_add_epi32(xmm0, xmm1);

    // Cast the result to the 32-bit integer type and return it:
    _mm_cvtsi128_si32(xmm0)
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
