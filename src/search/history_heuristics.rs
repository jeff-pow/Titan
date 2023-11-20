use crate::{
    moves::moves::Move,
    types::pieces::{Color, PieceName},
};

pub const MAX_HIST_VAL: i32 = i16::MAX as i32;

#[derive(Clone, Copy, Default)]
pub struct HistoryEntry {
    score: i32,
    counter: Move,
    // King can't be captured, so it doesn't need an entry
    capt_hist: [i32; 5],
}

fn calc_bonus(depth: i32) -> i32 {
    // if depth > 13 {
    //     32
    // } else {
    //     16 * depth * depth + 128 * (depth - 1).max(0)
    // }
    (155 * depth).min(2000)
}

fn update_history(score: &mut i32, depth: i32, is_good: bool) {
    let bonus = if is_good { calc_bonus(depth) } else { -calc_bonus(depth) };
    *score += bonus - (*score * bonus.abs() / MAX_HIST_VAL);
}

#[derive(Clone)]
pub struct MoveHistory {
    search_history: Box<[[[HistoryEntry; 64]; 6]; 2]>,
}

impl MoveHistory {
    pub fn update_quiet_history(&mut self, m: Move, is_good: bool, side: Color, depth: i32) {
        let i = &mut self.search_history[side][m.piece_moving()][m.dest_square()].score;
        update_history(i, depth, is_good);
    }

    pub fn quiet_history(&self, m: Move, side: Color) -> i32 {
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

    pub fn update_capt_hist(&mut self, m: Move, side: Color, capture: PieceName, depth: i32, is_good: bool) {
        let i = &mut self.search_history[side][m.piece_moving()][m.dest_square()].capt_hist[capture];
        update_history(i, depth, is_good);
    }

    pub fn capt_hist(&self, m: Move, side: Color, capture: PieceName) -> i32 {
        self.search_history[side][m.piece_moving()][m.dest_square()].capt_hist[capture]
    }
}

impl Default for MoveHistory {
    fn default() -> Self {
        Self {
            search_history: Box::new([[[HistoryEntry::default(); 64]; 6]; 2]),
        }
    }
}
