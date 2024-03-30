use crate::{
    board::Board,
    chess_move::{Direction, Move},
    search::search::MAX_SEARCH_DEPTH,
    types::{
        pieces::{Color, Piece, PieceName, NUM_PIECES},
        square::{Square, NUM_SQUARES},
    },
};

use super::{Align64, Block, NET};
use std::ops::{Index, IndexMut};

#[derive(Copy, Clone, Default, Debug, PartialEq, Eq)]
pub struct Delta {
    pub add: [(u16, u16); 2],
    pub num_add: usize,
    pub sub: [(u16, u16); 2],
    pub num_sub: usize,
}

impl Delta {
    fn clear(&mut self) {
        *self = Self::default();
    }

    fn add_contains(&self, x: usize) -> bool {
        for idx in 0..self.num_add {
            if self.add[idx].0 as usize == x || self.add[idx].1 as usize == x {
                return true;
            }
        }
        false
    }

    fn sub_contains(&self, x: usize) -> bool {
        for idx in 0..self.num_sub {
            if self.sub[idx].0 as usize == x || self.sub[idx].1 as usize == x {
                return true;
            }
        }
        false
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
pub struct Accumulator {
    pub vals: [Align64<Block>; 2],
    pub correct: [bool; 2],
    pub m: Move,
    pub capture: Piece,
}

impl Default for Accumulator {
    fn default() -> Self {
        Self { vals: [NET.feature_bias; 2], correct: [true; 2], m: Move::NULL, capture: Piece::None }
    }
}

impl Index<Color> for Accumulator {
    type Output = Block;

    fn index(&self, index: Color) -> &Self::Output {
        &self.vals[index.idx()]
    }
}

impl IndexMut<Color> for Accumulator {
    fn index_mut(&mut self, index: Color) -> &mut Self::Output {
        &mut self.vals[index.idx()]
    }
}

impl Accumulator {
    fn add_sub(&mut self, old: &Accumulator, a1: usize, s1: usize, side: Color) {
        #[cfg(feature = "avx512")]
        unsafe {
            self.avx512_add_sub(old, a1, black_add, s1, black_sub);
        }
        #[cfg(not(feature = "avx512"))]
        {
            let weights = &NET.feature_weights;
            self[side].iter_mut().zip(&weights[a1].0).zip(&weights[s1].0).zip(old[side].iter()).for_each(
                |(((i, &a), &s), &o)| {
                    *i = o + a - s;
                },
            );
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn add_sub_sub(&mut self, old: &Accumulator, a1: usize, s1: usize, s2: usize, side: Color) {
        #[cfg(feature = "avx512")]
        unsafe {
            self.avx512_add_sub_sub(old, a1, black_add, s1, black_sub_1, s2, black_sub_2);
        }
        #[cfg(not(feature = "avx512"))]
        {
            let weights = &NET.feature_weights;
            self[side]
                .iter_mut()
                .zip(&weights[a1].0)
                .zip(&weights[s1].0)
                .zip(&weights[s2].0)
                .zip(old[side].iter())
                .for_each(|((((i, &a), &s1), &s2), &o)| {
                    *i = o + a - s1 - s2;
                });
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn add_add_sub_sub(&mut self, old: &Accumulator, a1: usize, a2: usize, s1: usize, s2: usize, side: Color) {
        #[cfg(feature = "avx512")]
        unsafe {
            self.avx512_add_add_sub_sub(old, a1, black_add_1, a2, black_add_2, s1, black_sub_1, s2, black_sub_2);
        }
        #[cfg(not(feature = "avx512"))]
        {
            let weights = &NET.feature_weights;
            self[side]
                .iter_mut()
                .zip(&weights[a1].0)
                .zip(&weights[a2].0)
                .zip(&weights[s1].0)
                .zip(&weights[s2].0)
                .zip(old[side].iter())
                .for_each(|(((((i, &a1), &a2), &s1), &s2), &o)| {
                    *i = o + a1 + a2 - s1 - s2;
                });
        }
    }

    pub(crate) fn lazy_update(&mut self, delta: &mut Delta, old: &Accumulator, side: Color, board: &Board) {
        let m = self.m;
        let piece_moving = m.promotion().unwrap_or(m.piece_moving());
        let a1 = feature_idx_lazy(piece_moving, m.to(), side);
        assert!(delta.add_contains(a1), "{:?} {:?}", a1, delta);
        let s1 = feature_idx_lazy(m.piece_moving(), m.from(), side);
        assert!(delta.sub_contains(s1), "{:?} {:?}", s1, delta);
        if m.is_castle() {
            let rook = Piece::new(PieceName::Rook, m.piece_moving().color());
            let a2 = feature_idx_lazy(rook, m.castle_type().rook_to(), side);
            let s2 = feature_idx_lazy(rook, m.castle_type().rook_from(), side);
            assert!(delta.add_contains(a2));
            assert!(delta.sub_contains(s2));
            assert_eq!(delta.num_add, 2);
            assert_eq!(delta.num_sub, 2);
            self.add_add_sub_sub(old, a1, a2, s1, s2, side);
        } else if self.capture != Piece::None || m.is_en_passant() {
            let cap_square = if m.is_en_passant() {
                match m.piece_moving().color() {
                    Color::White => m.to().shift(Direction::South),
                    Color::Black => m.to().shift(Direction::North),
                }
            } else {
                m.to()
            };
            let capture =
                if m.is_en_passant() { Piece::new(PieceName::Pawn, !m.piece_moving().color()) } else { self.capture };
            let s2 = feature_idx_lazy(capture, cap_square, side);
            self.add_sub_sub(old, a1, s1, s2, side);
            assert!(delta.sub_contains(s2));

            assert_eq!(delta.num_sub, 2);
            assert_eq!(delta.num_add, 1);
        } else {
            self.add_sub(old, a1, s1, side);
            assert_eq!(delta.num_sub, 1);
            assert_eq!(delta.num_add, 1);
        }
        self.correct[side] = true;
        // Castling
        let (w_add1, b_add1) = delta.add[0];
        let (w_add2, b_add2) = delta.add[1];
        let (w_sub1, b_sub1) = delta.sub[0];
        let (w_sub2, b_sub2) = delta.sub[1];

        // if side == Color::White {
        //     self.add_add_sub_sub(
        //         old,
        //         usize::from(w_add1),
        //         usize::from(w_add2),
        //         usize::from(w_sub1),
        //         usize::from(w_sub2),
        //         Color::White,
        //     );
        // } else {
        //     self.add_add_sub_sub(
        //         old,
        //         usize::from(b_add1),
        //         usize::from(b_add2),
        //         usize::from(b_sub1),
        //         usize::from(b_sub2),
        //         Color::Black,
        //     );
        // }
    }

    pub fn add_feature(&mut self, piece: PieceName, color: Color, sq: Square) {
        let white_idx = feature_idx(color, piece, sq);
        let black_idx = feature_idx(!color, piece, sq.flip_vertical());
        self.activate(&NET.feature_weights[white_idx], Color::White);
        self.activate(&NET.feature_weights[black_idx], Color::Black);
    }

    fn activate(&mut self, weights: &Block, color: Color) {
        #[cfg(feature = "avx512")]
        unsafe {
            self.avx512_activate(weights, color);
        }

        #[cfg(not(feature = "avx512"))]
        self[color].iter_mut().zip(weights).for_each(|(i, &d)| {
            *i += d;
        });
    }
}

const COLOR_OFFSET: usize = NUM_SQUARES * NUM_PIECES;
const PIECE_OFFSET: usize = NUM_SQUARES;

const fn feature_idx(color: Color, piece: PieceName, sq: Square) -> usize {
    color.idx() * COLOR_OFFSET + piece.idx() * PIECE_OFFSET + sq.idx()
}

fn feature_idx_lazy(piece: Piece, sq: Square, view: Color) -> usize {
    match view {
        Color::White => piece.color().idx() * COLOR_OFFSET + piece.name().idx() * PIECE_OFFSET + sq.idx(),
        Color::Black => {
            (!piece.color()).idx() * COLOR_OFFSET + piece.name().idx() * PIECE_OFFSET + sq.flip_vertical().idx()
        }
    }
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

#[derive(Clone)]
pub struct AccumulatorStack {
    pub(crate) stack: Vec<Accumulator>,
    pub top: usize,
}

impl AccumulatorStack {
    pub fn apply_update(&mut self, delta: &mut Delta, board: &Board, m: Move, capture: Piece) {
        let (bottom, top) = self.stack.split_at_mut(self.top + 1);
        top[0].correct = [false; 2];
        top[0].m = m;
        top[0].capture = capture;
        top[0].lazy_update(delta, bottom.last().unwrap(), Color::White, board);
        top[0].lazy_update(delta, bottom.last().unwrap(), Color::Black, board);
        delta.clear();
        self.top += 1;
    }

    pub fn top(&mut self) -> &mut Accumulator {
        &mut self.stack[self.top]
    }

    pub fn pop(&mut self) -> Accumulator {
        self.top -= 1;
        self.stack[self.top + 1]
    }

    pub fn push(&mut self, acc: Accumulator) {
        self.top += 1;
        self.stack[self.top] = acc;
    }

    pub fn clear(&mut self, base_accumulator: &Accumulator) {
        self.stack[0] = *base_accumulator;
        self.top = 0;
    }

    pub fn new(base_accumulator: &Accumulator) -> Self {
        let mut vec = vec![Accumulator::default(); MAX_SEARCH_DEPTH as usize + 50];
        vec[0] = *base_accumulator;
        Self { stack: vec, top: 0 }
    }
}
