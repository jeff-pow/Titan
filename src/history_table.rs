use crate::{board::Board, chess_move::Move, types::pieces::PieceName};

use crate::correction::CorrectionHistory;
use crate::search::SearchStack;
use crate::types::pieces::Piece;

pub const MAX_HIST_VAL: i32 = 16384;

fn update_history(score: &mut i32, bonus: i32) {
    *score += bonus - *score * bonus.abs() / MAX_HIST_VAL;
}

pub fn capthist_capture(board: &Board, m: Move) -> PieceName {
    if m.is_en_passant() || m.promotion().is_some() {
        // Use Pawn for promotions here because pawns can't be in the back ranks anyways, so these
        // spaces can't be occupied anyway
        // Credit to viridithas
        PieceName::Pawn
    } else {
        board.piece_at(m.to()).name()
    }
}

#[derive(Clone)]
pub struct QuietHistory([[i32; 64]; 12]);

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
        Self([[0; 64]; 12])
    }
}

#[derive(Clone, Copy)]
pub struct HistoryEntry {
    score: i32,
    counter: Option<Move>,
    // King can't be captured, so it doesn't need an entry
    capt_hist: [i32; 5],
    cont_hist: [[i32; 64]; 6],
}

impl Default for HistoryEntry {
    fn default() -> Self {
        Self { score: Default::default(), counter: None, capt_hist: Default::default(), cont_hist: [[0; 64]; 6] }
    }
}

#[derive(Clone)]
pub struct HistoryTable {
    search_history: Box<[[HistoryEntry; 64]; 12]>,
    pub corr_hist: CorrectionHistory,
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
        ply: usize,
    ) {
        let bonus = (238 * depth).min(2095);
        let best_piece = board.piece_at(best_move.from());

        if best_move.is_tactical(board) {
            let cap = capthist_capture(board, best_move);
            self.update_capt_hist(best_move, best_piece, cap, bonus);
        } else {
            if let Some((m, p)) = stack.prev(ply - 1) {
                self.set_counter(m, p, best_move);
            }
            if depth > 3 || quiets_tried.len() > 1 {
                self.update_quiet_history(best_move, best_piece, bonus);
                self.update_cont_hist(best_move, best_piece, stack, ply, bonus);
            }
            // Only penalize quiets if best_move was quiet
            for m in quiets_tried {
                if *m == best_move {
                    continue;
                }
                let p = board.piece_at(m.from());
                self.update_quiet_history(*m, p, -bonus);
                self.update_cont_hist(*m, p, stack, ply, -bonus);
            }
        }

        // Always penalize tacticals since they should always be good no matter what the position
        for m in tacticals_tried {
            if *m == best_move {
                continue;
            }
            let p = board.piece_at(m.from());
            let cap = capthist_capture(board, *m);
            self.update_capt_hist(*m, p, cap, -bonus);
        }
    }

    fn update_quiet_history(&mut self, m: Move, piece: Piece, bonus: i32) {
        let i = &mut self.search_history[piece][m.to()].score;
        update_history(i, bonus);
    }

    pub(crate) fn quiet_history(&self, m: Move, piece: Piece, stack: &SearchStack, ply: usize) -> i32 {
        self.search_history[piece][m.to()].score + self.cont_hist(m, piece, stack, ply)
    }

    fn set_counter(&mut self, prev: Move, prev_piece: Piece, m: Move) {
        self.search_history[prev_piece][prev.to()].counter = Some(m);
    }

    pub fn get_counter(&self, m: Option<Move>, piece: Piece) -> Option<Move> {
        m.and_then(|m| self.search_history[piece][m.to()].counter)
    }

    fn update_capt_hist(&mut self, m: Move, piece: Piece, capture: PieceName, bonus: i32) {
        let i = &mut self.search_history[piece][m.to()].capt_hist[capture];
        update_history(i, bonus);
    }

    pub fn capt_hist(&self, m: Move, piece: Piece, board: &Board) -> i32 {
        let cap = capthist_capture(board, m);
        self.search_history[piece][m.to()].capt_hist[cap]
    }

    fn update_cont_hist(&mut self, m: Move, piece: Piece, stack: &SearchStack, ply: usize, bonus: i32) {
        let prevs = [stack.prev(ply - 1), stack.prev(ply - 2), stack.prev(ply - 4)];
        let entry = &mut self.search_history[piece][m.to()].cont_hist;
        for (prev_m, prev_piece) in prevs.into_iter().flatten() {
            let i = &mut entry[prev_piece.name()][prev_m.to()];
            update_history(i, bonus);
        }
    }

    pub(crate) fn cont_hist(&self, m: Move, piece: Piece, stack: &SearchStack, ply: usize) -> i32 {
        let mut score = 0;
        let prevs = [stack.prev(ply - 1), stack.prev(ply - 2), stack.prev(ply - 4)];
        let entry = &self.search_history[piece][m.to()];
        for (prev_m, prev_piece) in prevs.into_iter().flatten() {
            score += entry.cont_hist[prev_piece.name()][prev_m.to()];
        }
        score
    }
}

impl Default for HistoryTable {
    fn default() -> Self {
        Self { search_history: Box::new([[HistoryEntry::default(); 64]; 12]), corr_hist: CorrectionHistory::default() }
    }
}
