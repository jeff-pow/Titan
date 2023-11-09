use crate::{moves::moves::Move, types::pieces::Color};

pub const MAX_HIST_VAL: i32 = i16::MAX as i32;

#[derive(Default, Clone, Copy)]
pub struct HistoryEntry {
    score: i32,
    counter: Move,
}

#[derive(Clone)]
pub struct MoveHistory {
    search_history: Box<[[[HistoryEntry; 64]; 6]; 2]>,
}

impl MoveHistory {
    fn update_search_history(&mut self, m: Move, bonus: i32, side: Color) {
        let i = &mut self.search_history[side][m.piece_moving()][m.dest_square()].score;
        *i += bonus - *i * bonus.abs() / MAX_HIST_VAL;
    }

    pub fn update_history(&mut self, m: Move, bonus: i32, side: Color) {
        self.update_search_history(m, bonus, side);
    }

    pub fn get_history(&self, m: Move, side: Color) -> i32 {
        self.get_search_history(m, side)
    }

    fn get_search_history(&self, m: Move, side: Color) -> i32 {
        self.search_history[side][m.piece_moving()][m.dest_square()].score
    }

    pub fn set_counter(&mut self, side: Color, prev: Move, m: Move) {
        self.search_history[side][prev.piece_moving()][prev.dest_square()].counter = m;
    }

    pub fn get_counter(&self, side: Color, m: Move) -> Move {
        if m == Move::NULL {
            Move::NULL
        } else {
            self.search_history[side][m.piece_moving()][m.dest_square()].counter
        }
    }
}

impl Default for MoveHistory {
    fn default() -> Self {
        Self {
            search_history: Box::new([[[HistoryEntry::default(); 64]; 6]; 2]),
        }
    }
}
