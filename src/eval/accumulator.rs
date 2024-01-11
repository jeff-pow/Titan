use arrayvec::ArrayVec;

use crate::types::{
    pieces::{Color, Piece, PieceName, NUM_PIECES},
    square::{Square, NUM_SQUARES},
};

use super::{Block, HIDDEN_SIZE, NET};

#[derive(Clone, Default, Debug)]
pub(crate) struct Delta {
    // Only 32 pieces to add, so that's the cap
    add: ArrayVec<(Piece, Square), 32>,
    sub: ArrayVec<(Piece, Square), 32>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C, align(64))]
pub struct Accumulator(pub(super) [Block; 2]);

impl Default for Accumulator {
    fn default() -> Self {
        Self([NET.feature_bias; 2])
    }
}

impl Accumulator {
    pub(self) fn apply_update(&mut self, delta: &mut Delta) {
        let weights = &NET.feature_weights;
        for i in 0..HIDDEN_SIZE {
            let [white, black] = &mut self.0;
            for (p, sq) in &delta.add {
                let white_idx = feature_idx(p.color(), p.name(), *sq);
                let black_idx = feature_idx(!p.color(), p.name(), sq.flip_vertical());
                white[i] += weights[white_idx][i];
                black[i] += weights[black_idx][i];
            }
            for (p, sq) in &delta.sub {
                let white_idx = feature_idx(p.color(), p.name(), *sq);
                let black_idx = feature_idx(!p.color(), p.name(), sq.flip_vertical());
                white[i] -= weights[white_idx][i];
                black[i] -= weights[black_idx][i];
            }
        }
    }

    fn add_sub(&mut self, white_add: usize, black_add: usize, white_sub: usize, black_sub: usize) {
        let weights = &NET.output_weights;
        self.0[Color::White].iter_mut().zip(&weights[white_add]).zip(&weights[white_sub]).for_each(
            |((i, &a), &s)| {
                *i += a - s;
            },
        );
        self.0[Color::Black].iter_mut().zip(&weights[black_add]).zip(&weights[black_sub]).for_each(
            |((i, &a), &s)| {
                *i += a - s;
            },
        )
    }

    pub(crate) fn lazy_update(&mut self, delta: &Delta) {
        if delta.add.len() == 1 && delta.sub.len() == 1 {
            let [white, black] = &mut self.0;
        }
    }

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
        #[cfg(feature = "avx512")]
        unsafe {
            self.avx512_deactivate(weights, color);
        }

        #[cfg(not(feature = "avx512"))]
        self.0[color].iter_mut().zip(weights).for_each(|(i, &d)| {
            *i -= d;
        });
    }

    fn activate(&mut self, weights: &Block, color: Color) {
        #[cfg(feature = "avx512")]
        unsafe {
            self.avx512_activate(weights, color);
        }

        #[cfg(not(feature = "avx512"))]
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

#[cfg(test)]
mod accumulator_tests {
    use crate::{
        board::fen::{build_board, STARTING_FEN},
        moves::moves::from_san,
    };
    use std::time::Instant;

    use super::*;

    #[test]
    fn delta() {
        let mut d = Delta::default();
        let mut board = build_board(STARTING_FEN);
        let m = from_san("e2e4", &board);
        let mut acc = board.accumulator;
        let mut asdf = acc;
        let start = Instant::now();
        asdf.add_feature(m.piece_moving().name(), m.piece_moving().color(), m.dest_square());
        asdf.remove_feature(m.piece_moving().name(), m.piece_moving().color(), m.origin_square());
        dbg!(start.elapsed());

        let mut make_move_acc = board.refresh_accumulators();
        assert!(board.make_move::<true>(m, &mut make_move_acc));

        d.add.push((m.piece_moving(), m.dest_square()));
        d.sub.push((m.piece_moving(), m.origin_square()));
        let delta = Instant::now();
        acc.apply_update(&mut d);
        dbg!(delta.elapsed());
        assert_eq!(asdf, acc);
        assert_eq!(make_move_acc, acc);
        assert_eq!(make_move_acc, board.refresh_accumulators());
    }
}
