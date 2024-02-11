use crate::{
    board::board::Board,
    types::{
        pieces::{Color, Piece, PieceName, NUM_PIECES},
        square::{Square, NUM_SQUARES},
    },
};

use super::{Block, NET};

#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub(crate) struct Delta {
    pub add: [(u16, u16); 2],
    pub num_add: usize,
    pub sub: [(u16, u16); 2],
    pub num_sub: usize,
}

impl Delta {
    fn clear(&mut self) {
        *self = Self::default();
    }

    pub(crate) fn add(&mut self, p: Piece, sq: Square) {
        let w_idx = feature_idx(p.color(), p.name(), sq);
        let b_idx = feature_idx(!p.color(), p.name(), sq.flip_vertical());
        self.add[self.num_add] = (w_idx as u16, b_idx as u16);
        self.num_add += 1;
    }

    pub(crate) fn remove(&mut self, p: Piece, sq: Square) {
        let w_idx = feature_idx(p.color(), p.name(), sq);
        let b_idx = feature_idx(!p.color(), p.name(), sq.flip_vertical());
        self.sub[self.num_sub] = (w_idx as u16, b_idx as u16);
        self.num_sub += 1;
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C, align(64))]
pub struct Accumulator(pub(super) [Block; 2]);

impl Default for Accumulator {
    fn default() -> Self {
        Self([*NET.feature_bias; 2])
    }
}

impl Accumulator {
    fn add_sub(&mut self, white_add: usize, black_add: usize, white_sub: usize, black_sub: usize) {
        #[cfg(feature = "avx512")]
        unsafe {
            self.avx512_add_sub(white_add, black_add, white_sub, black_sub);
        }
        #[cfg(not(feature = "avx512"))]
        {
            let weights = &NET.feature_weights;
            self.0[Color::White]
                .iter_mut()
                .zip(&weights[white_add].0)
                .zip(&weights[white_sub].0)
                .for_each(|((i, &a), &s)| {
                    *i += a - s;
                });
            self.0[Color::Black]
                .iter_mut()
                .zip(&weights[black_add].0)
                .zip(&weights[black_sub].0)
                .for_each(|((i, &a), &s)| {
                    *i += a - s;
                })
        }
    }

    fn add_sub_sub(
        &mut self,
        white_add: usize,
        black_add: usize,
        white_sub_1: usize,
        black_sub_1: usize,
        white_sub_2: usize,
        black_sub_2: usize,
    ) {
        #[cfg(feature = "avx512")]
        unsafe {
            self.avx512_add_sub_sub(
                white_add,
                black_add,
                white_sub_1,
                black_sub_1,
                white_sub_2,
                black_sub_2,
            );
        }
        #[cfg(not(feature = "avx512"))]
        {
            let weights = &NET.feature_weights;
            self.0[Color::White]
                .iter_mut()
                .zip(&weights[white_add].0)
                .zip(&weights[white_sub_1].0)
                .zip(&weights[white_sub_2].0)
                .for_each(|(((i, &a), &s1), &s2)| {
                    *i += a - s1 - s2;
                });
            self.0[Color::Black]
                .iter_mut()
                .zip(&weights[black_add].0)
                .zip(&weights[black_sub_1].0)
                .zip(&weights[black_sub_2].0)
                .for_each(|(((i, &a), &s1), &s2)| {
                    *i += a - s1 - s2;
                });
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn add_add_sub_sub(
        &mut self,
        white_add_1: usize,
        black_add_1: usize,
        white_add_2: usize,
        black_add_2: usize,
        white_sub_1: usize,
        black_sub_1: usize,
        white_sub_2: usize,
        black_sub_2: usize,
    ) {
        #[cfg(feature = "avx512")]
        unsafe {
            self.avx512_add_add_sub_sub(
                white_add_1,
                black_add_1,
                white_add_2,
                black_add_2,
                white_sub_1,
                black_sub_1,
                white_sub_2,
                black_sub_2,
            );
        }
        #[cfg(not(feature = "avx512"))]
        {
            let weights = &NET.feature_weights;
            self.0[Color::White]
                .iter_mut()
                .zip(&weights[white_add_1].0)
                .zip(&weights[white_add_2].0)
                .zip(&weights[white_sub_1].0)
                .zip(&weights[white_sub_2].0)
                .for_each(|((((i, &a1), &a2), &s1), &s2)| {
                    *i += a1 + a2 - s1 - s2;
                });
            self.0[Color::Black]
                .iter_mut()
                .zip(&weights[black_add_1].0)
                .zip(&weights[black_add_2].0)
                .zip(&weights[black_sub_1].0)
                .zip(&weights[black_sub_2].0)
                .for_each(|((((i, &a1), &a2), &s1), &s2)| {
                    *i += a1 + a2 - s1 - s2;
                });
        }
    }

    pub(crate) fn lazy_update(&mut self, delta: &mut Delta) {
        if delta.add.len() == 1 && delta.sub.len() == 1 {
            let (w_add, b_add) = delta.add[0];
            let (w_sub, b_sub) = delta.sub[0];
            self.add_sub(
                usize::from(w_add),
                usize::from(b_add),
                usize::from(w_sub),
                usize::from(b_sub),
            );
        } else if delta.add.len() == 1 && delta.sub.len() == 2 {
            let (w_add, b_add) = delta.add[0];
            let (w_sub1, b_sub1) = delta.sub[0];
            let (w_sub2, b_sub2) = delta.sub[1];
            self.add_sub_sub(
                usize::from(w_add),
                usize::from(b_add),
                usize::from(w_sub1),
                usize::from(b_sub1),
                usize::from(w_sub2),
                usize::from(b_sub2),
            );
        } else {
            // Castling
            let (w_add1, b_add1) = delta.add[0];
            let (w_add2, b_add2) = delta.add[1];
            let (w_sub1, b_sub1) = delta.sub[0];
            let (w_sub2, b_sub2) = delta.sub[1];
            self.add_add_sub_sub(
                usize::from(w_add1),
                usize::from(b_add1),
                usize::from(w_add2),
                usize::from(b_add2),
                usize::from(w_sub1),
                usize::from(b_sub1),
                usize::from(w_sub2),
                usize::from(b_sub2),
            );
        }
        delta.clear();
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

impl Board {
    pub fn new_accumulator(&mut self) -> Accumulator {
        let mut acc = Accumulator::default();
        for c in Color::iter() {
            for p in PieceName::iter() {
                for sq in self.bitboard(c, p) {
                    acc.add_feature(p, c, sq);
                }
            }
        }
        acc
    }
}

#[cfg(test)]
mod accumulator_tests {

    #[test]
    fn delta() {}
}
