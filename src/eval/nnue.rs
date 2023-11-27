use std::arch::x86_64::{
    __m512i, _mm512_add_epi16, _mm512_add_epi32, _mm512_cmpgt_epi16_mask, _mm512_cmplt_epi16_mask,
    _mm512_dpwssds_epi32, _mm512_loadu_epi16, _mm512_mask_mov_epi16, _mm512_reduce_add_epi32, _mm512_set1_epi16,
    _mm512_setzero_si512, _mm512_storeu_epi16, _mm512_sub_epi16,
};

use crate::{
    board::board::Board,
    types::{
        pieces::{Color, PieceName, NUM_PIECES},
        square::{Square, NUM_SQUARES},
    },
};

pub const INPUT_SIZE: usize = 768;
const HIDDEN_SIZE: usize = 768;
const Q: i32 = 255 * 64;
const SCALE: i32 = 400;
static NET: Network = unsafe { std::mem::transmute(*include_bytes!("../../net.nnue")) };

type Block = [i16; HIDDEN_SIZE];

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C, align(64))]
pub struct Accumulator([Block; 2]);

impl Default for Accumulator {
    fn default() -> Self {
        Self([NET.feature_bias; 2])
    }
}

impl Accumulator {
    pub fn add_feature(&mut self, piece: PieceName, color: Color, sq: Square) {
        let white_idx = feature_idx(color, piece, sq);
        let black_idx = feature_idx(!color, piece, sq.flip_vertical());
        unsafe {
            self.simd_activate(&NET.feature_weights[white_idx], Color::White);
            self.simd_activate(&NET.feature_weights[black_idx], Color::Black);
        }
        // if is_x86_feature_detected!("avx512f") {
        // } else {
        //     // self.activate(&NET.feature_weights[white_idx], Color::White);
        //     // self.activate(&NET.feature_weights[black_idx], Color::Black);
        // }
    }

    pub fn remove_feature(&mut self, piece: PieceName, color: Color, sq: Square) {
        let white_idx = feature_idx(color, piece, sq);
        let black_idx = feature_idx(!color, piece, sq.flip_vertical());
        unsafe {
            self.simd_deactivate(&NET.feature_weights[white_idx], Color::White);
            self.simd_deactivate(&NET.feature_weights[black_idx], Color::Black);
        }
        // if is_x86_feature_detected!("avx512f") {
        // } else {
        //     self.deactivate(&NET.feature_weights[white_idx], Color::White);
        //     self.deactivate(&NET.feature_weights[black_idx], Color::Black);
        // }
    }

    unsafe fn simd_activate(&mut self, weights: &Block, color: Color) {
        const REQUIRED_ITERS: usize = HIDDEN_SIZE / 32;
        const CHUNK_SIZE: usize = 32;
        if is_x86_feature_detected!("avx512f") {
            for i in 0..REQUIRED_ITERS {
                let weights = _mm512_loadu_epi16(&weights[i * CHUNK_SIZE]);
                let acc = _mm512_loadu_epi16(&self.0[color][i * CHUNK_SIZE]);
                let updated_acc = _mm512_add_epi16(acc, weights);
                _mm512_storeu_epi16(&mut self.0[color][i * CHUNK_SIZE], updated_acc);
            }
        }
    }

    unsafe fn simd_deactivate(&mut self, weights: &Block, color: Color) {
        const REQUIRED_ITERS: usize = HIDDEN_SIZE / 32;
        const CHUNK_SIZE: usize = 32;
        if is_x86_feature_detected!("avx512f") {
            for i in 0..REQUIRED_ITERS {
                let weights = _mm512_loadu_epi16(&weights[i * CHUNK_SIZE]);
                let acc = _mm512_loadu_epi16(&self.0[color][i * CHUNK_SIZE]);
                let updated_acc = _mm512_sub_epi16(acc, weights);
                _mm512_storeu_epi16(&mut self.0[color][i * CHUNK_SIZE], updated_acc);
            }
        }
    }

    fn deactivate(&mut self, weights: &Block, color: Color) {
        self.0[color].iter_mut().zip(weights).for_each(|(i, &d)| {
            *i -= d;
        });
    }

    fn activate(&mut self, weights: &Block, color: Color) {
        self.0[color].iter_mut().zip(weights).for_each(|(i, &d)| {
            *i += d;
        });
    }
}

#[derive(Clone, Debug)]
#[repr(C, align(64))]
struct Network {
    feature_weights: [Block; INPUT_SIZE],
    feature_bias: Block,
    output_weights: [Block; 2],
    output_bias: i16,
}

unsafe fn flatten(required_iters: usize, chunk_size: usize, acc: &Block, weights: &Block) -> __m512i {
    let mut sum = _mm512_setzero_si512();
    for i in 0..required_iters {
        let us_vector = _mm512_loadu_epi16(&acc[i * chunk_size]);
        let crelu_result = clipped_relu(us_vector);
        let weights = _mm512_loadu_epi16(&weights[i * chunk_size]);
        sum = _mm512_dpwssds_epi32(sum, crelu_result, weights);
    }
    sum
}

impl Board {
    #[allow(clippy::assertions_on_constants)]
    pub fn evaluate(&self) -> i32 {
        let (us, them) = (&self.accumulator.0[self.to_move], &self.accumulator.0[!self.to_move]);
        let weights = &NET.output_weights;

        let mut output = i32::from(NET.output_bias);

        if is_x86_feature_detected!("avx512f") {
            assert!(HIDDEN_SIZE % 16 == 0);
            const REQUIRED_ITERS: usize = HIDDEN_SIZE / 32;
            const CHUNK_SIZE: usize = 32;
            unsafe {
                // 512 bits / 32 bit integers = 16 ints per chunk
                assert_eq!(REQUIRED_ITERS * CHUNK_SIZE, HIDDEN_SIZE);

                let acc_us = flatten(REQUIRED_ITERS, CHUNK_SIZE, us, &weights[0]);
                let acc_them = flatten(REQUIRED_ITERS, CHUNK_SIZE, them, &weights[1]);

                let result_vector = _mm512_add_epi32(acc_us, acc_them);
                output += _mm512_reduce_add_epi32(result_vector);
            }
        } else {
            for (&i, &w) in us.iter().zip(&weights[0]) {
                output += crelu(i) * i32::from(w);
            }

            for (&i, &w) in them.iter().zip(&weights[1]) {
                output += crelu(i) * i32::from(w);
            }
        }
        let a = (output) * SCALE / Q;
        assert!(i16::MIN as i32 <= a && a <= i16::MAX as i32);
        a
    }
}

const RELU_MIN: i16 = 0;
const RELU_MAX: i16 = 255;

fn crelu(i: i16) -> i32 {
    i32::from(i.clamp(RELU_MIN, RELU_MAX))
}

unsafe fn clipped_relu(i: __m512i) -> __m512i {
    let min = _mm512_set1_epi16(RELU_MIN);
    let max = _mm512_set1_epi16(RELU_MAX);
    let cmp_lt = _mm512_cmplt_epi16_mask(i, min);
    let cmp_gt = _mm512_cmpgt_epi16_mask(i, max);

    let result_lt = _mm512_mask_mov_epi16(i, cmp_lt, min);

    _mm512_mask_mov_epi16(result_lt, cmp_gt, max)
}

const COLOR_OFFSET: usize = NUM_SQUARES * NUM_PIECES;
const PIECE_OFFSET: usize = NUM_SQUARES;

fn feature_idx(color: Color, piece: PieceName, sq: Square) -> usize {
    color.idx() * COLOR_OFFSET + piece.idx() * PIECE_OFFSET + sq.idx()
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
