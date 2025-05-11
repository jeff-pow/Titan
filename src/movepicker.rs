use super::{
    chess_move::Move,
    movelist::{MoveList, MoveListEntry},
};
use crate::{board::Board, movegen::MGT, thread::ThreadData};

const TT_MOVE_SCORE: i32 = i32::MAX - 1000;
const GOOD_CAPTURE: i32 = 10_000_000;
const KILLER_SCORE: i32 = 1_000_000;
const COUNTER_MOVE_SCORE: i32 = 800_000;
const BAD_CAPTURE: i32 = -10000;

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
    return_quiets: bool,
    margin: i32,

    moves: MoveList,
    index: usize,

    tt_move: Option<Move>,
    killer_move: Option<Move>,
    counter_move: Option<Move>,
}

impl MovePicker {
    pub fn new(tt_move: Option<Move>, td: &ThreadData, margin: i32, return_quiets: bool) -> Self {
        Self {
            moves: MoveList::default(),
            index: 0,
            phase: MovePickerPhase::TTMove,
            margin,
            tt_move,
            killer_move: td.stack[td.ply].killer_move,
            counter_move: None,
            return_quiets,
        }
    }

    pub fn skip_quiets(&mut self) {
        self.return_quiets = false;
    }

    /// Select the next move to try. Returns None if there are no more moves to try.
    pub fn next(&mut self, board: &Board, td: &ThreadData) -> Option<MoveListEntry> {
        loop {
            match self.phase {
                MovePickerPhase::TTMove => {
                    self.phase = MovePickerPhase::CapturesInit;
                    if let Some(tt_move) = self.tt_move.filter(|&m| board.is_pseudo_legal(m)) {
                        return Some(MoveListEntry { m: tt_move, score: TT_MOVE_SCORE });
                    }
                }
                MovePickerPhase::CapturesInit => {
                    self.phase = MovePickerPhase::GoodCaptures;
                    board.generate_moves(MGT::CapturesOnly, &mut self.moves);
                    score_captures(td, self.margin, board, &mut self.moves.arr);
                }
                MovePickerPhase::GoodCaptures => {
                    while self.index < self.moves.len() {
                        let picked = self.moves.pick_move(self.index);

                        if self.tt_move == Some(picked.m) {
                            self.index += 1;
                            continue;
                        }

                        if picked.score >= GOOD_CAPTURE {
                            self.index += 1;
                            return Some(picked);
                        } else {
                            break;
                        }
                    }

                    self.phase = if self.return_quiets { MovePickerPhase::Killer } else { MovePickerPhase::Remainders };
                }
                MovePickerPhase::Killer => {
                    self.phase =
                        if self.return_quiets { MovePickerPhase::Counter } else { MovePickerPhase::Remainders };
                    if !self.return_quiets {
                        continue;
                    }

                    if let Some(killer) = self.killer_move {
                        if Some(killer) != self.tt_move && board.is_pseudo_legal(killer) {
                            return Some(MoveListEntry { m: killer, score: KILLER_SCORE });
                        }
                    }
                }
                MovePickerPhase::Counter => {
                    self.phase =
                        if self.return_quiets { MovePickerPhase::QuietsInit } else { MovePickerPhase::Remainders };
                    if !self.return_quiets {
                        continue;
                    }

                    if let Some(counter) = self.counter_move {
                        if Some(counter) != self.tt_move
                            && Some(counter) != self.killer_move
                            && board.is_pseudo_legal(counter)
                        {
                            return Some(MoveListEntry { m: counter, score: COUNTER_MOVE_SCORE });
                        }
                    }
                }
                MovePickerPhase::QuietsInit => {
                    self.phase = MovePickerPhase::Remainders;
                    if !self.return_quiets {
                        continue;
                    }

                    let start_quiets = self.moves.len();
                    board.generate_moves(MGT::QuietsOnly, &mut self.moves);
                    score_quiets(board, td, &mut self.moves.arr[start_quiets..]);
                }
                MovePickerPhase::Remainders => {
                    if let Some(picked) = self.select_next(board) {
                        return Some(picked);
                    }

                    self.phase = MovePickerPhase::Finished;
                }
                MovePickerPhase::Finished => return None,
            }
        }
    }

    /// Chooses the next valid move with the next highest score
    fn select_next(&mut self, board: &Board) -> Option<MoveListEntry> {
        loop {
            if self.index >= self.moves.len() {
                return None;
            }

            let picked = self.moves.pick_move(self.index);

            self.index += 1;
            if (!self.return_quiets && picked.m.is_quiet(board)) || self.is_cached(picked.m) {
                continue;
            }
            return Some(picked);
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
