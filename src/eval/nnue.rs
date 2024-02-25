use super::{accumulator::Accumulator, Align64, Block, INPUT_SIZE, NET};

use crate::{board::board::Board, search::search::NEAR_CHECKMATE, types::pieces::PieceName};
/**
* When changing activation functions, both the normalization factor and QA may need to change
* alongside changing the crelu calls to screlu in simd and serial code.
*/
const QA: i32 = 255; // CHANGES WITH NET QUANZIZATION
const QB: i32 = 64;
const QAB: i32 = QA * QB;
const NORMALIZATION_FACTOR: i32 = QA; // CHANGES WITH SCRELU/CRELU ACTIVATION
pub(super) const RELU_MIN: i16 = 0;
pub(super) const RELU_MAX: i16 = QA as i16;

const SCALE: i32 = 400;

#[derive(Clone, Debug)]
#[repr(C, align(64))]
pub(super) struct Network {
    pub feature_weights: [Align64<Block>; INPUT_SIZE],
    pub feature_bias: Align64<Block>,
    pub output_weights: [Align64<Block>; 2],
    pub output_bias: i16,
}

impl Board {
    pub fn mat_scale(&self) -> i32 {
        700 + (PieceName::Knight.value() * self.piece(PieceName::Knight).count_bits()
            + PieceName::Bishop.value() * self.piece(PieceName::Bishop).count_bits()
            + PieceName::Rook.value() * self.piece(PieceName::Rook).count_bits()
            + PieceName::Queen.value() * self.piece(PieceName::Queen).count_bits())
            / 32
    }

    pub fn evaluate(&self, acc: &Accumulator) -> i32 {
        let raw = self.raw_evaluate(acc);
        let mat_bal = self.mat_scale();
        let draw_scale = (200 - self.half_moves as i32) / 200;
        (raw * mat_bal * draw_scale).clamp(-NEAR_CHECKMATE, NEAR_CHECKMATE)
    }

    #[allow(clippy::assertions_on_constants)]
    pub fn raw_evaluate(&self, acc: &Accumulator) -> i32 {
        let (us, them) = (&acc.0[self.to_move], &acc.0[!self.to_move]);
        let weights = &NET.output_weights;
        let output = flatten(us, &weights[0]) + flatten(them, &weights[1]);
        let a = (i32::from(NET.output_bias) + output / NORMALIZATION_FACTOR) * SCALE / QAB;
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

fn flatten(acc: &Block, weights: &Block) -> i32 {
    #[cfg(feature = "avx512")]
    {
        use super::simd::avx512;
        unsafe { avx512::flatten(acc, weights) }
    }
    #[cfg(all(not(feature = "avx512"), feature = "avx2"))]
    {
        use super::simd::avx2;
        unsafe { avx2::flatten(acc, weights) }
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
        let mut board = build_board(STARTING_FEN);
        let acc = board.new_accumulator();
        let start = Instant::now();
        let iters = 10_000_000_u128;
        for _ in 0..iters {
            black_box(board.evaluate(&acc));
        }
        let duration = start.elapsed().as_nanos();
        println!("{} ns per iter", duration / iters);
        dbg!(duration / iters);
    }
}
