use crate::{moves::moves::Move, types::pieces::Color};

pub const MAX_HIST_VAL: i32 = i16::MAX as i32;

#[derive(Clone, Copy)]
struct HistoryEntry {
    score: i32,
    counter: Move,
    continuation: [[i32; 64]; 6],
}

impl Default for HistoryEntry {
    fn default() -> Self {
        Self {
            score: 0,
            counter: Move::NULL,
            continuation: [[0; 64]; 6],
        }
    }
}

#[derive(Clone)]
pub struct MoveHistory {
    search_history: Box<[[[HistoryEntry; 64]; 6]; 2]>,
}

impl MoveHistory {
    fn update_search_history(&mut self, m: Move, bonus: i32, side: Color) {
        let i = &mut self.search_history[side.idx()][m.piece_moving().idx()][m.dest_square().idx()].score;
        *i += bonus - *i * bonus.abs() / MAX_HIST_VAL;
    }

    fn update_conthist_score(&mut self, m: Move, bonus: i32, side: Color, prev_moves: &[Move]) {
        let entry = &mut self.search_history[side.idx()][m.piece_moving().idx()][m.dest_square().idx()];
        for x in prev_moves {
            if *x != Move::NULL {
                let e = &mut entry.continuation[x.piece_moving().idx()][x.dest_square().idx()];
                *e += bonus - *e * bonus.abs() / MAX_HIST_VAL;
            }
        }
    }

    pub fn update_history(&mut self, m: Move, depth: i32, side: Color, move_history: &[Move], good: bool) {
        let bonus = (155 * depth).min(2000) * if good { 1 } else { -1 };

        let entry = &mut self.search_history[side.idx()][m.piece_moving().idx()][m.dest_square().idx()];
        entry.score += bonus - entry.score * bonus.abs() / MAX_HIST_VAL;

        let len = move_history.len();
        let max_elements = 4.min(len);
        let prev_moves = &move_history[max_elements..];

        for x in prev_moves {
            if *x != Move::NULL {
                let cont = &mut entry.continuation[x.piece_moving().idx()][x.dest_square().idx()];
                *cont += bonus - *cont * bonus.abs() / MAX_HIST_VAL;
            }
        }
    }

    pub fn set_counter(&mut self, m: Move, side: Color, prev: Move) {
        self.search_history[side.idx()][prev.piece_moving().idx()][prev.dest_square().idx()].counter = m;
    }

    pub fn get_counter(&self, m: Move, side: Color) -> Move {
        self.search_history[side.idx()][m.piece_moving().idx()][m.dest_square().idx()].counter
    }

    pub fn get_history(&self, m: Move, side: Color, prev_moves: &[Move]) -> i32 {
        self.get_search_history(m, side) + self.get_conthist_score(m, prev_moves, side)
    }

    fn get_search_history(&self, m: Move, side: Color) -> i32 {
        self.search_history[side.idx()][m.piece_moving().idx()][m.dest_square().idx()].score
    }

    fn get_conthist_score(&self, m: Move, prev_moves: &[Move], side: Color) -> i32 {
        let entry = &self.search_history[side.idx()][m.piece_moving().idx()][m.dest_square().idx()];
        let mut score = 0;
        for x in prev_moves {
            if *x != Move::NULL {
                score += entry.continuation[x.piece_moving().idx()][x.dest_square().idx()];
            }
        }
        score
    }

    pub fn clear(&mut self) {
        *self = Self::default();
    }
}

impl Default for MoveHistory {
    fn default() -> Self {
        Self {
            search_history: Box::new([[[HistoryEntry::default(); 64]; 6]; 2]),
        }
    }
}
