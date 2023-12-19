use crate::types::{
    pieces::{Color, PieceName, NUM_PIECES},
    square::{Square, NUM_SQUARES},
};

use super::{Block, NET};

#[cfg(feature = "simd")]
use super::{CHUNK_SIZE, REQUIRED_ITERS};
#[cfg(feature = "simd")]
use std::arch::x86_64::{
    _mm256_add_epi16, _mm256_loadu_epi16, _mm256_storeu_epi16, _mm256_sub_epi16,
};

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C, align(64))]
pub struct Accumulator(pub(super) [Block; 2]);

impl Default for Accumulator {
    fn default() -> Self {
        Self([NET.feature_bias; 2])
    }
}

impl Accumulator {
    pub fn add_feature(&mut self, piece: PieceName, color: Color, sq: Square) {
        let white_idx = feature_idx(color, piece, sq);
        let black_idx = feature_idx(!color, piece, sq.flip_vertical());
        self.activate(&NET.feature_weights[white_idx], Color::White);
        self.activate(&NET.feature_weights[black_idx], Color::Black);
    }

    pub fn remove_feature(&mut self, piece: PieceName, color: Color, sq: Square) {
        let white_idx = feature_idx(color, piece, sq);
        let black_idx = feature_idx(!color, piece, sq.flip_vertical());
        self.deactivate(&NET.feature_weights[white_idx], Color::White);
        self.deactivate(&NET.feature_weights[black_idx], Color::Black);
    }

    fn deactivate(&mut self, weights: &Block, color: Color) {
        #[cfg(feature = "simd")]
        unsafe {
            for i in 0..REQUIRED_ITERS {
                let weights = _mm256_loadu_epi16(&weights[i * CHUNK_SIZE]);
                let acc = _mm256_loadu_epi16(&self.0[color][i * CHUNK_SIZE]);
                let updated_acc = _mm256_sub_epi16(acc, weights);
                _mm256_storeu_epi16(&mut self.0[color][i * CHUNK_SIZE], updated_acc);
            }
        }
        #[cfg(not(feature = "simd"))]
        self.0[color].iter_mut().zip(weights).for_each(|(i, &d)| {
            *i -= d;
        });
    }

    fn activate(&mut self, weights: &Block, color: Color) {
        #[cfg(feature = "simd")]
        unsafe {
            for i in 0..REQUIRED_ITERS {
                let weights = _mm256_loadu_epi16(&weights[i * CHUNK_SIZE]);
                let acc = _mm256_loadu_epi16(&self.0[color][i * CHUNK_SIZE]);
                let updated_acc = _mm256_add_epi16(acc, weights);
                _mm256_storeu_epi16(&mut self.0[color][i * CHUNK_SIZE], updated_acc);
            }
        }
        #[cfg(not(feature = "simd"))]
        self.0[color].iter_mut().zip(weights).for_each(|(i, &d)| {
            *i += d;
        });
    }
}

const COLOR_OFFSET: usize = NUM_SQUARES * NUM_PIECES;
const PIECE_OFFSET: usize = NUM_SQUARES;

fn feature_idx(color: Color, piece: PieceName, sq: Square) -> usize {
    color.idx() * COLOR_OFFSET + piece.idx() * PIECE_OFFSET + sq.idx()
}
