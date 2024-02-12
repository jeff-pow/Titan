use crate::{
    board::board::Board,
    moves::moves::Move,
    spsa::{HIST_DEPTH_MOD, HIST_MIN},
    types::pieces::PieceName,
};

use super::SearchStack;

pub const MAX_HIST_VAL: i32 = 16384;

#[derive(Clone, Copy)]
pub struct HistoryEntry {
    score: i32,
    counter: Move,
    // King can't be captured, so it doesn't need an entry
    capt_hist: [i32; 5],
    cont_hist: [[i32; 64]; 6],
}

impl Default for HistoryEntry {
    fn default() -> Self {
        Self {
            score: Default::default(),
            counter: Default::default(),
            capt_hist: Default::default(),
            cont_hist: [[0; 64]; 6],
        }
    }
}

#[derive(Clone)]
pub struct HistoryTable {
    search_history: Box<[[HistoryEntry; 64]; 12]>,
}

fn calc_bonus(depth: i32) -> i32 {
    (HIST_DEPTH_MOD.val() * depth).min(HIST_MIN.val())
}

fn update_history(score: &mut i32, depth: i32, is_good: bool) {
    let bonus = if is_good { 1 } else { -1 } * calc_bonus(depth);
    *score += bonus - *score * bonus.abs() / MAX_HIST_VAL;
}

pub(crate) fn capthist_capture(board: &Board, m: Move) -> PieceName {
    if m.is_en_passant() || m.promotion().is_some() {
        // Use Pawn for promotions here because pawns can't be in the back ranks anyways, so these
        // spaces can't be occupied anyway
        // Credit to viridithas
        PieceName::Pawn
    } else {
        board.piece_at(m.dest_square()).name()
    }
}

impl HistoryTable {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn update_histories(
        &mut self,
        best_move: Move,
        quiets_tried: &[Move],
        tacticals_tried: &[Move],
        board: &Board,
        depth: i32,
        stack: &SearchStack,
        ply: i32,
    ) {
        assert_ne!(best_move, Move::NULL);
        if best_move.is_tactical(board) {
            let cap = capthist_capture(board, best_move);
            self.update_capt_hist(best_move, cap, depth, true);
        } else {
            if stack.prev_move(ply - 1) != Move::NULL {
                self.set_counter(stack.prev_move(ply - 1), best_move);
            }
            if depth > 3 || quiets_tried.len() > 1 {
                self.update_quiet_history(best_move, true, depth);
                self.update_cont_hist(best_move, stack, ply, true, depth);
            }
            // Only penalize quiets if best_move was quiet
            for m in quiets_tried {
                if *m == best_move {
                    continue;
                }
                self.update_quiet_history(*m, false, depth);
                self.update_cont_hist(*m, stack, ply, false, depth)
            }
        }

        // Always penalize tacticals since they should always be good no matter what the position
        for m in tacticals_tried {
            if *m == best_move {
                continue;
            }
            assert_ne!(m, &Move::NULL);
            let cap = capthist_capture(board, *m);
            self.update_capt_hist(*m, cap, depth, false);
        }
    }

    fn update_quiet_history(&mut self, m: Move, is_good: bool, depth: i32) {
        let i = &mut self.search_history[m.piece_moving()][m.dest_square()].score;
        update_history(i, depth, is_good);
    }

    pub(crate) fn quiet_history(&self, m: Move, stack: &SearchStack, ply: i32) -> i32 {
        self.search_history[m.piece_moving()][m.dest_square()].score + self.cont_hist(m, stack, ply)
    }

    fn set_counter(&mut self, prev: Move, m: Move) {
        self.search_history[prev.piece_moving()][prev.dest_square()].counter = m;
    }

    pub fn get_counter(&self, m: Move) -> Move {
        if m == Move::NULL {
            Move::NULL
        } else {
            self.search_history[m.piece_moving()][m.dest_square()].counter
        }
    }

    fn update_capt_hist(&mut self, m: Move, capture: PieceName, depth: i32, is_good: bool) {
        let i = &mut self.search_history[m.piece_moving()][m.dest_square()].capt_hist[capture];
        update_history(i, depth, is_good);
    }

    pub fn capt_hist(&self, m: Move, board: &Board) -> i32 {
        let cap = capthist_capture(board, m);
        self.search_history[m.piece_moving()][m.dest_square()].capt_hist[cap]
    }

    fn update_cont_hist(
        &mut self,
        m: Move,
        stack: &SearchStack,
        ply: i32,
        is_good: bool,
        depth: i32,
    ) {
        let prevs = [stack.prev_move(ply - 1), stack.prev_move(ply - 2), stack.prev_move(ply - 4)];
        let entry = &mut self.search_history[m.piece_moving()][m.dest_square()].cont_hist;
        for prev in prevs {
            if prev != Move::NULL {
                let i = &mut entry[prev.piece_moving().name()][prev.dest_square()];
                update_history(i, depth, is_good);
            }
        }
    }

    pub(crate) fn cont_hist(&self, m: Move, stack: &SearchStack, ply: i32) -> i32 {
        let mut score = 0;
        let prevs = [stack.prev_move(ply - 1), stack.prev_move(ply - 2), stack.prev_move(ply - 4)];
        let entry = &self.search_history[m.piece_moving()][m.dest_square()];
        for prev in prevs {
            if prev != Move::NULL {
                score += entry.cont_hist[prev.piece_moving().name()][prev.dest_square()];
            }
        }
        score
    }
}

impl Default for HistoryTable {
    fn default() -> Self {
        Self { search_history: Box::new([[HistoryEntry::default(); 64]; 12]) }
    }
}
