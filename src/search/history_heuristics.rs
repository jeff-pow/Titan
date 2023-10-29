use crate::{
    moves::moves::Move,
    types::{
        bitboard::Bitboard,
        pieces::{Color, PieceName},
        square::Square,
    },
};

pub const MAX_HIST_VAL: i32 = i16::MAX as i32;

#[derive(Clone, Copy)]
struct HistoryEntry {
    score: [[i32; 2]; 2],
    counter: Move,
    continuation: [[i32; 64]; 6],
}

impl Default for HistoryEntry {
    fn default() -> Self {
        Self {
            score: [[0; 2]; 2],
            counter: Move::NULL,
            continuation: [[0; 64]; 6],
        }
    }
}

#[derive(Clone)]
pub struct MoveHistory {
    // Indexed [side][src sq][dest sq]
    // TODO: Index by piece instead of butterfly boards
    search_history: Box<[[[HistoryEntry; 64]; 8]; 2]>,
}

fn threatened(threats: Bitboard, sq: Square) -> usize {
    usize::from(threats & sq.bitboard() != Bitboard::EMPTY)
}

impl MoveHistory {
    fn get_score(&self, side: Color, m: Move, prevs: [Move; 2], threats: Bitboard, piece: PieceName) -> i32 {
        let entry = &self.search_history[side.idx()][piece.idx()][m.dest_square().idx()];
        let mut score = entry.score[threatened(threats, m.origin_square())][threatened(threats, m.dest_square())];
        for prev in prevs {
            if prev != Move::NULL {
                score += 0;
            }
        }
        0
    }

    pub fn update_history(&mut self, m: Move, bonus: i32, side: Color) {
        todo!()
    }

    pub fn get_history(&self, m: Move, side: Color) -> i32 {
        todo!()
    }
}

impl Default for MoveHistory {
    fn default() -> Self {
        Self {
            search_history: Box::new([[[HistoryEntry::default(); 64]; 8]; 2]),
        }
    }
}
