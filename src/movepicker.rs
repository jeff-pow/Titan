use super::{
    chess_move::Move,
    movelist::{MoveList, MoveListEntry},
};
use crate::{board::Board, movegen::MGT, thread::ThreadData};

#[derive(PartialEq, PartialOrd, Eq)]
pub enum Phase {
    TTMove,

    CapturesInit,
    GoodCaptures,

    Killer,

    QuietsInit,
    Quiets,

    BadCaptures,
}

pub struct MovePicker {
    pub phase: Phase,
    return_quiets: bool,
    margin: i32,

    moves: MoveList,
    index: usize,
    bad_captures: MoveList,

    tt_move: Option<Move>,
    killer_move: Option<Move>,
}

impl MovePicker {
    pub fn new(tt_move: Option<Move>, killer: Option<Move>, margin: i32, return_quiets: bool) -> Self {
        Self {
            moves: MoveList::default(),
            bad_captures: MoveList::default(),
            index: 0,
            phase: Phase::TTMove,
            margin,
            tt_move,
            killer_move: killer,
            return_quiets,
        }
    }

    pub const fn skip_quiets(&mut self) {
        self.return_quiets = false;
    }

    /// Select the next move to try. Returns None if there are no more moves to try.
    pub fn next(&mut self, board: &Board, td: &ThreadData) -> Option<Move> {
        if self.phase == Phase::TTMove {
            self.phase = Phase::CapturesInit;

            if let Some(tt_move) = self.tt_move {
                if board.is_pseudo_legal(tt_move) {
                    return Some(tt_move);
                }
            }
        }

        if self.phase == Phase::CapturesInit {
            self.phase = Phase::GoodCaptures;
            board.generate_moves(MGT::CapturesOnly, &mut self.moves);
            score_captures(td, board, &mut self.moves.arr);
        }

        if self.phase == Phase::GoodCaptures {
            while self.index < self.moves.len() {
                let picked = self.moves.pick_move(self.index);
                self.index += 1;

                if Some(picked.m) == self.tt_move {
                    continue;
                }

                if !board.see(picked.m, self.margin) {
                    self.bad_captures.push(picked.m);
                    continue;
                }

                return Some(picked.m);
            }

            self.phase = Phase::Killer;
        }

        if self.phase == Phase::Killer {
            if self.return_quiets {
                self.phase = Phase::QuietsInit;
                if let Some(killer) = self.killer_move {
                    if board.is_pseudo_legal(killer) && self.killer_move != self.tt_move {
                        return Some(killer);
                    }
                }
            } else {
                self.phase = Phase::BadCaptures;
            }
        }

        if self.phase == Phase::QuietsInit {
            if self.return_quiets {
                self.phase = Phase::Quiets;
                let len = self.moves.len();
                board.generate_moves(MGT::QuietsOnly, &mut self.moves);
                score_quiets(board, td, &mut self.moves.arr[len..]);
            } else {
                self.phase = Phase::BadCaptures;
            }
        }

        if self.phase == Phase::Quiets {
            if self.return_quiets {
                while self.index < self.moves.len() {
                    let picked = self.moves.pick_move(self.index);
                    self.index += 1;

                    if self.is_cached(picked.m) {
                        continue;
                    }

                    return Some(picked.m);
                }
            }

            self.phase = Phase::BadCaptures;
        }

        if self.phase == Phase::BadCaptures && !self.bad_captures.is_empty() {
            return Some(self.bad_captures.arr.pop().unwrap().m);
        }

        None
    }

    /// Determines if a move is stored as a special move by the move picker
    fn is_cached(&self, m: Move) -> bool {
        Some(m) == self.tt_move || Some(m) == self.killer_move
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

fn score_captures(td: &ThreadData, board: &Board, moves: &mut [MoveListEntry]) {
    for MoveListEntry { m, score } in moves {
        *score = td.capt_hist.get(*m, board.piece_at(m.from()), board);
    }
}
