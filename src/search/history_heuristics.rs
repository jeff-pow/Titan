use crate::{
    board::board::Board,
    moves::moves::Move,
    types::pieces::{Color, PieceName},
};

use super::SearchStack;

pub const MAX_HIST_VAL: i32 = i16::MAX as i32;

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

fn capthist_capture(board: &Board, m: Move) -> PieceName {
    if m.is_en_passant() || m.promotion().is_some() {
        // Use Pawn for promotions here because pawns can't be in the back ranks anyways, so these
        // spaces can't be occupied anyway
        // Credit to viridithas
        PieceName::Pawn
    } else {
        board.piece_at(m.dest_square()).unwrap()
    }
}

impl MoveHistory {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn update_histories(
        &mut self,
        best_move: Move,
        quiets_tried: &[Move],
        tacticals_tried: &[Move],
        prev: Move,
        board: &Board,
        depth: i32,
        stack: &SearchStack,
        ply: i32,
    ) {
        if best_move.is_tactical(board) {
            let cap = capthist_capture(board, best_move);
            self.update_capt_hist(best_move, board.to_move, cap, depth, true);
        } else {
            self.set_counter(board.to_move, prev, best_move);
            self.update_quiet_history(best_move, true, board.to_move, depth);
            self.update_cont_hist(best_move, stack, ply, true, board.to_move, depth);
            // Only penalize quiets if best_move was quiet
            for m in quiets_tried {
                self.update_quiet_history(*m, false, board.to_move, depth);
                self.update_cont_hist(*m, stack, ply, false, board.to_move, depth)
            }
        }

        // Always penalize tacticals since they should always be good no matter what the position
        for m in tacticals_tried {
            let cap = capthist_capture(board, *m);
            self.update_capt_hist(*m, board.to_move, cap, depth, false);
        }
    }

    fn update_quiet_history(&mut self, m: Move, is_good: bool, side: Color, depth: i32) {
        let i = &mut self.search_history[side][m.piece_moving()][m.dest_square()].score;
        update_history(i, depth, is_good);
    }

    pub(crate) fn quiet_history(&self, m: Move, side: Color, stack: &SearchStack, ply: i32) -> i32 {
        self.search_history[side][m.piece_moving()][m.dest_square()].score + self.cont_hist(m, stack, ply, side)
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

    fn update_capt_hist(&mut self, m: Move, side: Color, capture: PieceName, depth: i32, is_good: bool) {
        let i = &mut self.search_history[side][m.piece_moving()][m.dest_square()].capt_hist[capture];
        update_history(i, depth, is_good);
    }

    pub fn capt_hist(&self, m: Move, side: Color, board: &Board) -> i32 {
        let cap = capthist_capture(board, m);
        self.search_history[side][m.piece_moving()][m.dest_square()].capt_hist[cap]
    }

    fn update_cont_hist(&mut self, m: Move, stack: &SearchStack, ply: i32, is_good: bool, side: Color, depth: i32) {
        let ply = ply as usize;
        if ply > 0 {
            let prev = stack[ply - 1].played_move;
            if prev != Move::NULL {
                let i = &mut self.search_history[side][prev.piece_moving()][prev.dest_square()].cont_hist
                    [m.piece_moving()][m.dest_square()];
                update_history(i, depth, is_good);
            }

            if ply > 1 {
                let prev = stack[ply - 2].played_move;
                if prev != Move::NULL {
                    let i = &mut self.search_history[side][prev.piece_moving()][prev.dest_square()].cont_hist
                        [m.piece_moving()][m.dest_square()];
                    update_history(i, depth, is_good);
                }

                if ply > 3 {
                    let prev = stack[ply - 4].played_move;
                    if prev != Move::NULL {
                        let i = &mut self.search_history[side][prev.piece_moving()][prev.dest_square()].cont_hist
                            [m.piece_moving()][m.dest_square()];
                        update_history(i, depth, is_good);
                    }
                }
            }
        }
    }

    pub(crate) fn cont_hist(&self, m: Move, stack: &SearchStack, ply: i32, side: Color) -> i32 {
        let ply = ply as usize;
        let mut score = 0;
        // let entry = &self.search_history[side][m.piece_moving()][m.dest_square()];
        if ply > 0 {
            let prev = stack[ply - 1].played_move;
            // score += entry.cont_hist[prev.piece_moving()][prev.dest_square()];
            score += self.search_history[side][prev.piece_moving()][prev.dest_square()].cont_hist[m.piece_moving()]
                [m.dest_square()];

            if ply > 1 {
                let prev = stack[ply - 2].played_move;
                // score += entry.cont_hist[prev.piece_moving()][prev.dest_square()];
                score += self.search_history[side][prev.piece_moving()][prev.dest_square()].cont_hist[m.piece_moving()]
                    [m.dest_square()];

                if ply > 3 {
                    let prev = stack[ply - 4].played_move;
                    // score += entry.cont_hist[prev.piece_moving()][prev.dest_square()];
                    score += self.search_history[side][prev.piece_moving()][prev.dest_square()].cont_hist
                        [m.piece_moving()][m.dest_square()];
                }
            }
        }
        score
    }
}

impl Default for MoveHistory {
    fn default() -> Self {
        Self {
            search_history: Box::new([[[HistoryEntry::default(); 64]; 6]; 2]),
        }
    }
}
