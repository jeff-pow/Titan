use crate::moves::movegenerator::MGT;
use crate::search::killers::{KillerMoves, NUM_KILLER_MOVES};
use crate::{board::board::Board, moves::movegenerator::generate_psuedolegal_moves};

use super::{movelist::MoveList, moves::Move};

#[derive(Default, PartialEq)]
enum MovePickerPhase {
    #[default]
    TTMove,
    CapturesInit,
    Captures,
    KillerMovesInit,
    KillerMoves,
    QuietsInit,
    Quiets,
}

pub struct MovePicker<'a> {
    phase: MovePickerPhase,
    moves: MoveList,
    processed_idx: usize,
    tt_move: Move,
    board: &'a Board,
    killers: [Move; NUM_KILLER_MOVES],
    gen_quiets: bool,
}

impl<'a> Iterator for MovePicker<'a> {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        if self.phase == MovePickerPhase::TTMove {
            self.phase = MovePickerPhase::CapturesInit;
            if self.tt_move.is_valid(self.board) {
                return Some(self.tt_move);
            }
        }

        if self.phase == MovePickerPhase::CapturesInit {
            self.phase = MovePickerPhase::Captures;
            self.processed_idx = 0;
            self.moves = generate_psuedolegal_moves(self.board, MGT::CapturesOnly);
            self.moves.score_move_list(self.board, self.tt_move, &self.killers);
        }

        'captures: {
            if self.phase == MovePickerPhase::Captures {
                if self.processed_idx == self.moves.len {
                    self.phase = MovePickerPhase::KillerMovesInit;
                    break 'captures;
                }
                let m = self.moves.next().unwrap();
                assert_ne!(m, Move::NULL);
                assert!(m.is_valid(self.board));
                self.processed_idx += 1;
                return Some(m);
            }
        }

        if !self.gen_quiets {
            return None;
        }

        if self.phase == MovePickerPhase::KillerMovesInit {
            self.phase = MovePickerPhase::KillerMoves;
            self.moves = MoveList::default();
            self.processed_idx = 0;
            for m in self.killers {
                if m.is_valid(self.board) {
                    self.moves.push(m);
                }
            }
            self.moves.score_move_list(self.board, self.tt_move, &self.killers);
        }

        'killers: {
            if self.phase == MovePickerPhase::KillerMoves {
                if self.processed_idx == self.moves.len {
                    self.phase = MovePickerPhase::QuietsInit;
                    break 'killers;
                }
                let m = self.moves.next().unwrap();
                assert_ne!(m, Move::NULL);
                assert!(m.is_valid(self.board));
                self.processed_idx += 1;
                return Some(m);
            }
        }

        if self.phase == MovePickerPhase::QuietsInit {
            self.phase = MovePickerPhase::Quiets;
            self.processed_idx = 0;
            self.moves = generate_psuedolegal_moves(self.board, MGT::QuietsOnly);
            self.moves.score_move_list(self.board, self.tt_move, &self.killers);
        }

        if self.phase == MovePickerPhase::Quiets {
            if self.processed_idx == self.moves.len {
                return None;
            }
            assert_ne!(0, self.moves.len);
            let m = self.moves.next().unwrap();
            assert_ne!(m, Move::NULL);
            assert!(m.is_valid(self.board));
            self.processed_idx += 1;
            return Some(m);
        }

        unreachable!()
    }
}

impl<'a> MovePicker<'a> {
    #[allow(dead_code)]
    fn dummy(board: &'a Board) -> Self {
        MovePicker {
            tt_move: Move::NULL,
            board,
            killers: [Move::NULL; NUM_KILLER_MOVES],
            phase: MovePickerPhase::TTMove,
            moves: MoveList::default(),
            processed_idx: 0,
            gen_quiets: true,
        }
    }

    pub fn qsearch(board: &'a Board, tt_move: Move, gen_quiets: bool) -> Self {
        MovePicker {
            tt_move,
            board,
            killers: [Move::NULL; NUM_KILLER_MOVES],
            phase: MovePickerPhase::TTMove,
            moves: MoveList::default(),
            processed_idx: 0,
            gen_quiets,
        }
    }

    pub fn new(board: &'a Board, ply: i32, tt_move: Move, killers: &KillerMoves) -> Self {
        MovePicker {
            tt_move,
            board,
            killers: killers[ply as usize],
            phase: MovePickerPhase::TTMove,
            moves: MoveList::default(),
            processed_idx: 0,
            gen_quiets: true,
        }
    }
}
