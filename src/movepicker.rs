use super::{
    chess_move::Move,
    movelist::{MoveList, MoveListEntry},
};
use crate::{board::Board, movegen::MGT, thread::ThreadData};

pub const TT_MOVE_SCORE: i32 = i32::MAX - 1000;
pub const GOOD_CAPTURE: i32 = 10_000_000;
pub const FIRST_KILLER_SCORE: i32 = 1_000_000;
pub const COUNTER_MOVE_SCORE: i32 = 800_000;
pub const BAD_CAPTURE: i32 = -10000;

#[derive(PartialEq, PartialOrd, Eq)]
pub enum MovePickerPhase {
    TTMove,

    CapturesInit,
    GoodCaptures,

    Killer,
    Counter,

    QuietsInit,
    Remainders,

    Finished,
}

pub struct MovePicker {
    pub phase: MovePickerPhase,
    skip_quiets: bool,
    margin: i32,

    moves: MoveList,
    index: usize,

    tt_move: Option<Move>,
    killer_move: Option<Move>,
    counter_move: Option<Move>,
}

impl MovePicker {
    pub fn new(tt_move: Option<Move>, td: &ThreadData, margin: i32, skip_quiets: bool) -> Self {
        Self {
            moves: MoveList::default(),
            index: 0,
            phase: MovePickerPhase::TTMove,
            margin,
            tt_move,
            killer_move: td.stack[td.ply].killer_move,
            counter_move: None,
            skip_quiets,
        }
    }

    pub fn skip_quiets(&mut self) {
        self.skip_quiets = true;
    }

    /// Select the next move to try. Returns None if there are no more moves to try.
    pub fn next(&mut self, board: &Board, td: &ThreadData) -> Option<MoveListEntry> {
        if self.phase == MovePickerPhase::TTMove {
            self.phase = MovePickerPhase::CapturesInit;
            if let Some(tt_move) = self.tt_move {
                if board.occupancies().empty(tt_move.to()) && self.skip_quiets {
                    return self.next(board, td);
                }
                if board.is_pseudo_legal(self.tt_move) {
                    return Some(MoveListEntry { m: tt_move, score: TT_MOVE_SCORE });
                }
            }
        }

        if self.phase == MovePickerPhase::CapturesInit {
            self.phase = MovePickerPhase::GoodCaptures;
            board.generate_moves(MGT::CapturesOnly, &mut self.moves);
            score_captures(td, self.margin, board, &mut self.moves.arr);
        }

        if self.phase == MovePickerPhase::GoodCaptures {
            if let Some(m) = self.select_next(board) {
                if m.score >= GOOD_CAPTURE {
                    return Some(m);
                }
                // Move did not win, so we move on to quiet moves, and decrement index to play the
                // move again later
                self.index -= 1;
            }

            self.phase = if self.skip_quiets { MovePickerPhase::Finished } else { MovePickerPhase::Killer };
        }

        if self.phase == MovePickerPhase::Killer {
            self.phase = MovePickerPhase::Counter;
            if let Some(killer) = self.killer_move {
                if !self.skip_quiets && self.killer_move != self.tt_move && board.is_pseudo_legal(self.killer_move) {
                    return Some(MoveListEntry { m: killer, score: FIRST_KILLER_SCORE });
                }
            }
        }

        if self.phase == MovePickerPhase::Counter {
            self.phase = MovePickerPhase::QuietsInit;
            if let Some(counter_move) = self.counter_move {
                if !self.skip_quiets
                    && self.counter_move != self.tt_move
                    && self.counter_move != self.killer_move
                    && board.is_pseudo_legal(self.counter_move)
                {
                    return Some(MoveListEntry { m: counter_move, score: COUNTER_MOVE_SCORE });
                }
            }
        }

        if self.phase == MovePickerPhase::QuietsInit {
            self.phase = MovePickerPhase::Remainders;
            if !self.skip_quiets {
                let start = self.moves.len();
                board.generate_moves(MGT::QuietsOnly, &mut self.moves);
                let len = self.moves.len();
                let quiets = &mut self.moves.arr[start..len];
                score_quiets(board, td, quiets);
            }
        }

        if self.phase == MovePickerPhase::Remainders {
            if let Some(m) = self.select_next(board) {
                return Some(m);
            }
            self.phase = MovePickerPhase::Finished;
        }

        None
    }

    /// Chooses the next valid move with the next highest score
    fn select_next(&mut self, board: &Board) -> Option<MoveListEntry> {
        if self.index >= self.moves.len() {
            return None;
        }

        let entry = self.moves.pick_move(self.index);

        self.index += 1;

        if self.skip_quiets && entry.m.is_quiet(board) || self.is_cached(entry.m) {
            self.select_next(board)
        } else {
            Some(entry)
        }
    }

    /// Determines if a move is stored as a special move by the move picker
    fn is_cached(&self, m: Move) -> bool {
        let m = Some(m);
        m == self.tt_move || m == self.killer_move || m == self.counter_move
    }
}

fn score_quiets(board: &Board, td: &ThreadData, moves: &mut [MoveListEntry]) {
    for MoveListEntry { m, score } in moves {
        let p = board.piece_at(m.from());
        *score = td.quiet_hist.get(*m, p)
            + td.cont_hist.get(*m, p, &td.stack, td.ply - 1)
            + td.cont_hist.get(*m, p, &td.stack, td.ply - 2);
    }
}

fn score_captures(td: &ThreadData, margin: i32, board: &Board, moves: &mut [MoveListEntry]) {
    for MoveListEntry { m, score } in moves {
        *score = (if board.see(*m, margin) { GOOD_CAPTURE } else { BAD_CAPTURE })
            + td.capt_hist.get(*m, board.piece_at(m.from()), board);
    }
}
