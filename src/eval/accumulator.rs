use crate::{
    board::Board,
    chess_move::{Direction, Move},
    search::search::{MAX_SEARCH_DEPTH, NEAR_CHECKMATE},
    types::{
        pieces::{Color, Piece, PieceName, NUM_PIECES},
        square::{Square, NUM_SQUARES},
    },
};

use super::{
    network::{flatten, NORMALIZATION_FACTOR, QAB, SCALE},
    Align64, Block, NET,
};
use std::ops::{Index, IndexMut};

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
    pub fn raw_evaluate(&self, stm: Color) -> i32 {
        let (us, them) = (&self[stm], &self[!stm]);
        let weights = &NET.output_weights;
        let output = flatten(us, &weights[0]) + flatten(them, &weights[1]);
        ((i32::from(NET.output_bias) + output / NORMALIZATION_FACTOR) * SCALE / QAB)
            .clamp(-NEAR_CHECKMATE + 1, NEAR_CHECKMATE - 1)
    }

    /// Credit to viridithas for these values and concepts
    pub fn scaled_evaluate(&self, board: &Board) -> i32 {
        let raw = self.raw_evaluate(board.stm);
        let eval = raw * board.mat_scale() / 1024;
        let eval = eval * (200 - board.half_moves as i32) / 200;
        (eval).clamp(-NEAR_CHECKMATE, NEAR_CHECKMATE)
    }

    fn add_sub(&mut self, old: &Accumulator, a1: usize, s1: usize, side: Color) {
        #[cfg(feature = "avx512")]
        unsafe {
            self.avx512_add_sub(old, a1, s1, side);
        }
        #[cfg(not(feature = "avx512"))]
        {
            println!("not avx512 addsub");
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
            self.avx512_add_sub_sub(old, a1, s1, s2, side);
        }
        #[cfg(not(feature = "avx512"))]
        {
            let weights = &NET.feature_weights;
            println!("not avx512 addsubsub");
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
            self.avx512_add_add_sub_sub(old, a1, a2, s1, s2, side);
        }
        #[cfg(not(feature = "avx512"))]
        {
            println!("not avx512 addaddsubsub");
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

    pub(crate) fn lazy_update(&mut self, old: &Accumulator, side: Color) {
        let m = self.m;
        let piece_moving = m.promotion().unwrap_or(m.piece_moving());
        let a1 = feature_idx_lazy(piece_moving, m.to(), side);
        let s1 = feature_idx_lazy(m.piece_moving(), m.from(), side);
        if m.is_castle() {
            let rook = Piece::new(PieceName::Rook, m.piece_moving().color());
            let a2 = feature_idx_lazy(rook, m.castle_type().rook_to(), side);
            let s2 = feature_idx_lazy(rook, m.castle_type().rook_from(), side);

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
        } else {
            self.add_sub(old, a1, s1, side);
        }
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

    fn refresh(&mut self, board: &Board, view: Color) {
        self.vals[view] = NET.feature_bias;
        for sq in board.occupancies() {
            let p = board.piece_at(sq);
            let idx = feature_idx_lazy(p, sq, view);
            self.activate(&NET.feature_weights[idx], view);
        }
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
    pub fn new_accumulator(&self) -> Accumulator {
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
    pub fn update_stack(&mut self, m: Move, capture: Piece) {
        self.top += 1;
        self.stack[self.top].m = m;
        self.stack[self.top].capture = capture;
        self.stack[self.top].correct = [false; 2];
        // let (bottom, top) = self.stack.split_at_mut(self.top);
        // top[0].lazy_update(bottom.last().unwrap(), Color::White);
        // top[0].lazy_update(bottom.last().unwrap(), Color::Black);
    }

    fn all_lazy_updates(&mut self, side: Color) {
        let mut curr = self.top;
        while !self.stack[curr].correct[side] {
            curr -= 1;
        }

        while curr < self.top {
            let (bottom, top) = self.stack.split_at_mut(curr + 1);
            top[0].lazy_update(bottom.last().unwrap(), side);
            top[0].correct[side] = true;
            curr += 1;
        }
    }

    fn force_updates(&mut self, board: &Board) {
        for color in Color::iter() {
            if !self.stack[self.top].correct[color] {
                if self.can_efficiently_update(color) {
                    self.all_lazy_updates(color)
                } else {
                    self.stack[self.top].refresh(board, color);
                    self.stack[self.top].correct[color] = true;
                }
            }
        }
    }

    fn can_efficiently_update(&mut self, side: Color) -> bool {
        let mut curr = self.top;
        loop {
            curr -= 1;

            if self.stack[curr].correct[side.idx()] {
                return true;
            }
        }
    }

    pub fn evaluate(&mut self, board: &Board) -> i32 {
        self.force_updates(board);
        assert_eq!(self.stack[self.top].correct, [true; 2]);
        self.top().scaled_evaluate(board)
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
