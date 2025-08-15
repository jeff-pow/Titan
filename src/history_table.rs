use crate::{board::Board, chess_move::Move, types::pieces::PieceName};

use crate::search::SearchStack;
use crate::types::pieces::{Color, Piece};
use crate::utils::zeroed_box;

pub const MAX_HIST_VAL: i32 = 16384;

const fn update_history(score: &mut i32, bonus: i32) {
    *score += bonus - *score * bonus.abs() / MAX_HIST_VAL;
}

#[derive(Clone)]
pub struct QuietHistory(Box<[[i32; 64]; 12]>);

impl QuietHistory {
    pub fn update(&mut self, m: Move, piece: Piece, bonus: i32) {
        update_history(&mut self.0[piece][m.to()], bonus);
    }

    pub fn get(&self, m: Move, piece: Piece) -> i32 {
        self.0[piece][m.to()]
    }
}

impl Default for QuietHistory {
    fn default() -> Self {
        Self(Box::new([[0; 64]; 12]))
    }
}

#[derive(Clone)]
pub struct CaptureHistory(Box<[[[i32; 5]; 64]; 12]>);

impl CaptureHistory {
    pub fn update(&mut self, m: Move, piece: Piece, board: &Board, bonus: i32) {
        let capture = Self::map_capture(board, m);
        update_history(&mut self.0[piece][m.to()][capture], bonus);
    }

    pub fn get(&self, m: Move, piece: Piece, board: &Board) -> i32 {
        let capture = Self::map_capture(board, m);
        self.0[piece][m.to()][capture]
    }

    fn map_capture(board: &Board, m: Move) -> PieceName {
        if m.is_en_passant() || m.promotion().is_some() {
            // Use Pawn for promotions here because pawns can't be in the back ranks anyways, so these
            // spaces can't be occupied anyway
            // Credit to viridithas
            PieceName::Pawn
        } else {
            board.piece_at(m.to()).name()
        }
    }
}

impl Default for CaptureHistory {
    fn default() -> Self {
        Self(Box::new([[[0; 5]; 64]; 12]))
    }
}

#[derive(Clone)]
pub struct ContinuationHistory(Box<[[[[i32; 64]; 12]; 64]; 12]>);

impl ContinuationHistory {
    pub fn update(&mut self, m: Move, piece: Piece, stack: &SearchStack, ply: usize, bonus: i32) {
        let prev = stack.prev(ply);
        if let Some((prev_m, prev_piece)) = prev {
            update_history(&mut self.0[piece][m.to()][prev_piece][prev_m.to()], bonus);
        }
    }

    pub fn get(&self, m: Move, piece: Piece, stack: &SearchStack, ply: usize) -> i32 {
        stack.prev(ply).map_or(0, |(prev_m, prev_piece)| self.0[piece][m.to()][prev_piece][prev_m.to()])
    }
}

impl Default for ContinuationHistory {
    fn default() -> Self {
        Self(zeroed_box())
    }
}

#[derive(Clone)]
pub struct CorrectionHistory(Box<[[i32; Self::SIZE]; 2]>);

impl CorrectionHistory {
    const SIZE: usize = 16384;
    const LIMIT: i32 = 16384;

    pub fn update(&mut self, stm: Color, key: u64, diff: i32, depth: i32) {
        let bonus = (diff * depth).clamp(-Self::LIMIT / 4, Self::LIMIT / 4);
        update_history(&mut self.0[stm][key as usize % Self::SIZE], bonus);
    }

    pub fn get(&self, stm: Color, key: u64) -> i32 {
        self.0[stm][key as usize % Self::SIZE] / 100
    }
}

impl Default for CorrectionHistory {
    fn default() -> Self {
        Self(zeroed_box())
    }
}

// TODO: Make sure to reset history tables in threadpool's reset function when adding more histories
