use crate::{
    board::Board,
    chess_move::{Direction, Move},
    search::search::{MAX_SEARCH_DEPTH, NEAR_CHECKMATE},
    types::{
        pieces::{Color, Piece, PieceName},
        square::Square,
    },
};

use super::{
    network::{flatten, Network, BUCKETS, NORMALIZATION_FACTOR, QAB, SCALE},
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

    pub(crate) fn lazy_update(&mut self, old: &Accumulator, side: Color, board: &Board) {
        let m = self.m;
        let from = if side == Color::Black { m.from().flip_vertical() } else { m.from() };
        let to = if side == Color::Black { m.to().flip_vertical() } else { m.to() };
        assert!(
            m.piece_moving().name() != PieceName::King
                || m.piece_moving().color() != side
                || BUCKETS[from] == BUCKETS[to]
        );
        let piece_moving = m.promotion().unwrap_or(m.piece_moving());
        let king = board.king_square(side);
        let a1 = Network::feature_idx(piece_moving, m.to(), king, side);
        let s1 = Network::feature_idx(m.piece_moving(), m.from(), king, side);
        if m.is_castle() {
            let rook = Piece::new(PieceName::Rook, m.piece_moving().color());
            let a2 = Network::feature_idx(rook, m.castle_type().rook_to(), king, side);
            let s2 = Network::feature_idx(rook, m.castle_type().rook_from(), king, side);

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
            let s2 = Network::feature_idx(capture, cap_square, king, side);
            self.add_sub_sub(old, a1, s1, s2, side);
        } else {
            self.add_sub(old, a1, s1, side);
        }
    }

    pub fn add_feature(&mut self, piece: Piece, sq: Square, white_king: Square, black_king: Square) {
        let white_idx = Network::feature_idx(piece, sq, white_king, Color::White);
        let black_idx = Network::feature_idx(piece, sq, black_king, Color::Black);
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
            let idx = Network::feature_idx(p, sq, board.king_square(view), view);
            self.activate(&NET.feature_weights[idx], view);
        }
    }
}

impl Board {
    pub fn new_accumulator(&self) -> Accumulator {
        let mut acc = Accumulator::default();
        for sq in self.occupancies() {
            let p = self.piece_at(sq);
            acc.add_feature(p, sq, self.king_square(Color::White), self.king_square(Color::Black));
        }
        acc
    }
}

#[derive(Clone, Debug)]
pub struct AccumulatorStack {
    pub(crate) stack: Vec<Accumulator>,
    /// Top points to the active accumulator, not the space above it
    pub top: usize,
}

impl AccumulatorStack {
    pub fn update_stack(&mut self, m: Move, capture: Piece) {
        self.top += 1;
        self.stack[self.top].m = m;
        self.stack[self.top].capture = capture;
        self.stack[self.top].correct = [false; 2];
    }

    fn all_lazy_updates(&mut self, board: &Board, side: Color) {
        let mut curr = self.top;
        while !self.stack[curr].correct[side] {
            curr -= 1;
        }

        while curr < self.top {
            let (bottom, top) = self.stack.split_at_mut(curr + 1);
            top[0].lazy_update(bottom.last().unwrap(), side, board);
            top[0].correct[side] = true;
            curr += 1;
        }
    }

    fn force_updates(&mut self, board: &Board) {
        for color in Color::iter() {
            if !self.stack[self.top].correct[color] {
                if self.can_efficiently_update(color) {
                    self.all_lazy_updates(board, color)
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
            let m = self.stack[curr].m;
            let from = if side == Color::Black { m.from().flip_vertical() } else { m.from() };
            let to = if side == Color::Black { m.to().flip_vertical() } else { m.to() };

            if m.piece_moving().color() == side
                && m.piece_moving().name() == PieceName::King
                && BUCKETS[from] != BUCKETS[to]
            {
                return false;
            }

            if self.stack[curr].correct[side.idx()] {
                return true;
            }

            curr -= 1;
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

#[cfg(test)]
mod acc_test {
    use super::AccumulatorStack;
    use crate::{board::Board, chess_move::Move};

    macro_rules! make_move_nnue {
        ($board:ident, $stack:ident, $mv_str:literal) => {{
            let m = Move::from_san($mv_str, &$board);
            $stack.update_stack(m, $board.capture(m));
            assert!($board.make_move(m));
        }};
    }

    macro_rules! assert_correct {
        ($board:ident, $stack:ident) => {
            $stack.evaluate(&$board);
            assert_eq!($stack.top().vals, $board.new_accumulator().vals);
        };
    }

    #[test]
    fn lazy_updates() {
        let mut board = Board::from_fen("r3k2r/2pb1ppp/2pp1q2/p7/1nP1B3/1P2P3/P2N1PPP/R2QK2R w KQkq a6 0 14");
        let mut stack = AccumulatorStack::new(&board.new_accumulator());
        make_move_nnue!(board, stack, "e1g1");

        make_move_nnue!(board, stack, "e8d8");
        assert_correct!(board, stack);
    }

    #[test]
    fn deeper_error() {
        let mut board = Board::from_fen("8/8/1p2k1p1/3p3p/1p1P1P1P/1P2PK2/8/8 w - - 3 54");
        let mut stack = AccumulatorStack::new(&board.new_accumulator());

        make_move_nnue!(board, stack, "e3e4");
        make_move_nnue!(board, stack, "e6e7");
        make_move_nnue!(board, stack, "e4e5");
        make_move_nnue!(board, stack, "e7e6");
        make_move_nnue!(board, stack, "f3g3");
        make_move_nnue!(board, stack, "e6f5");
        make_move_nnue!(board, stack, "g3h3");
        make_move_nnue!(board, stack, "f5f4");
        make_move_nnue!(board, stack, "e5e6");
        make_move_nnue!(board, stack, "f4e3");
        assert_correct!(board, stack);
    }
}
