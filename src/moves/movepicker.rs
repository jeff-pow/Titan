use crate::board::board::Board;
use crate::moves::movegenerator::MGT;
use crate::search::history_table::capthist_capture;
use crate::search::thread::ThreadData;
use crate::search::NUM_KILLER_MOVES;
use crate::types::pieces::PieceName;

use super::movelist::MoveListEntry;
use super::{movelist::MoveList, moves::Move};

#[derive(Default, PartialEq)]
enum MovePickerPhase {
    #[default]
    TTMove,

    CapturesInit,
    GoodCaptures,

    FirstKiller,
    SecondKiller,
    Counter,

    QuietsInit,
    Remainders,

    Finished,
}

pub struct MovePicker {
    phase: MovePickerPhase,
    gen_quiets: bool,

    moves: MoveList,
    current: usize,

    tt_move: Move,
    killers: [Move; NUM_KILLER_MOVES],
    counter: Move,
}

impl MovePicker {
    pub(crate) fn next(&mut self, td: &ThreadData, board: &Board) -> Option<MoveListEntry> {
        if self.phase == MovePickerPhase::TTMove {
            self.phase = MovePickerPhase::CapturesInit;
            if board.is_pseudo_legal(self.tt_move) {
                return Some(MoveListEntry { m: self.tt_move, score: TTMOVE });
            }
        }

        if self.phase == MovePickerPhase::CapturesInit {
            self.phase = MovePickerPhase::GoodCaptures;
            self.moves = board.generate_moves(MGT::CapturesOnly);
            score_captures(td, board, &mut self.moves.arr[self.current..]);
        }

        if self.phase == MovePickerPhase::GoodCaptures {
            if self.current < self.moves.len() {
                let entry = self.moves.pick_move(self.current);
                self.current += 1;
                if entry.m == self.tt_move {
                    return self.next(td, board);
                }
                return Some(entry);
            }
            self.phase = if self.gen_quiets {
                MovePickerPhase::FirstKiller
            } else {
                MovePickerPhase::Remainders
            };
        }

        if self.phase == MovePickerPhase::FirstKiller {
            self.phase = MovePickerPhase::SecondKiller;
            if self.gen_quiets
                && self.killers[0] != self.tt_move
                && board.is_pseudo_legal(self.killers[0])
            {
                return Some(MoveListEntry { m: self.killers[0], score: KILLER_ONE });
            }
        }

        if self.phase == MovePickerPhase::SecondKiller {
            self.phase = MovePickerPhase::Counter;
            if self.gen_quiets
                && self.killers[1] != self.tt_move
                && self.killers[1] != self.killers[0]
                && board.is_pseudo_legal(self.killers[1])
            {
                return Some(MoveListEntry { m: self.killers[1], score: KILLER_TWO });
            }
        }

        if self.phase == MovePickerPhase::Counter {
            self.phase = MovePickerPhase::QuietsInit;
            if self.gen_quiets
                && self.counter != self.tt_move
                && self.counter != self.killers[0]
                && self.counter != self.killers[1]
                && board.is_pseudo_legal(self.counter)
            {
                return Some(MoveListEntry { m: self.counter, score: COUNTER_MOVE });
            }
        }

        if self.phase == MovePickerPhase::QuietsInit {
            self.phase = MovePickerPhase::Remainders;
            if self.gen_quiets {
                self.current = self.moves.len();
                self.moves.append(board.generate_moves(MGT::QuietsOnly));
            }

            for m in self.moves.arr.iter().take(self.moves.len()) {
                assert!(
                    self.moves.arr.iter().take(self.moves.len()).filter(|&x| x == m).count() == 1,
                    "{}",
                    m.m.to_san()
                );
            }

            score_quiets(td, board, &mut self.moves.arr[self.current..]);
        }

        if self.phase == MovePickerPhase::Remainders {
            if self.current < self.moves.len() {
                let entry = self.moves.pick_move(self.current);
                self.current += 1;
                if self.is_cached(entry.m) {
                    return self.next(td, board);
                }
                return Some(entry);
            } else {
                self.phase = MovePickerPhase::Finished;
                return None;
            }
        }

        if self.phase == MovePickerPhase::Finished {
            return None;
        }
        None
    }

    pub(crate) fn qsearch(tt_move: Move, td: &ThreadData, board: &Board, gen_quiets: bool) -> Self {
        let prev = td.stack.prev_move(td.ply - 1);
        let counter = td.history.get_counter(board.to_move, prev);
        MovePicker {
            tt_move,
            killers: [Move::NULL; NUM_KILLER_MOVES],
            phase: MovePickerPhase::TTMove,
            moves: MoveList::default(),
            current: 0,
            gen_quiets,
            counter,
        }
    }

    pub(crate) fn new(tt_move: Move, td: &ThreadData, board: &Board) -> Self {
        let prev = td.stack.prev_move(td.ply - 1);
        let counter = td.history.get_counter(board.to_move, prev);
        MovePicker {
            tt_move,
            killers: td.stack[td.ply].killers,
            phase: MovePickerPhase::TTMove,
            moves: MoveList::default(),
            current: 0,
            gen_quiets: true,
            counter,
        }
    }

    fn is_cached(&self, m: Move) -> bool {
        m == self.tt_move || self.killers.contains(&m) || m == self.counter
    }
}

fn score_captures(td: &ThreadData, board: &Board, moves: &mut [MoveListEntry]) {
    const MVV: [i32; 6] = [0, 2400, 2400, 4800, 9600, 0];
    for m in &mut *moves {
        assert!(m.score == 0);
        assert!(m.m.is_tactical(board));
    }

    for MoveListEntry { m, score } in moves {
        *score = if let Some(p) = m.promotion() {
            match p {
                PieceName::Queen => QUEEN_PROMOTION + td.history.capt_hist(*m, board),
                _ => BAD_PROMOTION,
            }
        } else {
            (if board.see(*m, -PieceName::Pawn.value()) { GOOD_CAPTURE } else { BAD_CAPTURE })
                + MVV[capthist_capture(board, *m)]
                + td.history.capt_hist(*m, board)
        }
    }
}

fn score_quiets(td: &ThreadData, board: &Board, moves: &mut [MoveListEntry]) {
    for m in &mut *moves {
        assert!(m.score == 0);
        assert!(board.is_quiet(m.m));
    }

    for MoveListEntry { m, score } in moves {
        *score = td.history.quiet_history(*m, board.to_move, &td.stack, td.ply)
    }
}

const TTMOVE: i32 = i32::MAX - 1000;
const QUEEN_PROMOTION: i32 = 20_000_001;
pub const GOOD_CAPTURE: i32 = 10_000_000;
const KILLER_ONE: i32 = 1_000_000;
const KILLER_TWO: i32 = 900_000;
const COUNTER_MOVE: i32 = 800_000;
pub const BAD_CAPTURE: i32 = -10000;
const BAD_PROMOTION: i32 = -QUEEN_PROMOTION;
