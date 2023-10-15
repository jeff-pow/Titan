use crate::moves::movegenerator::MGT;
use crate::search::killers::{empty_killers, KillerMoves, NUM_KILLER_MOVES};
use crate::{board::board::Board, moves::movegenerator::generate_psuedolegal_moves};

use super::{movelist::MoveList, moves::Move};

#[derive(Default, PartialEq)]
enum MovePickerPhase {
    #[default]
    TTMove,
    CapturesInit,
    Captures,
    Killer1,
    Killer2,
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
        // if self.moves.is_empty() {
        //     self.moves = generate_psuedolegal_moves(self.board, MGT::All);
        //     self.moves.score_move_list(self.board, self.tt_move, &self.killers);
        // }
        // return self.moves.next();
        if self.phase == MovePickerPhase::CapturesInit {
            self.phase = MovePickerPhase::Captures;
            debug_assert_eq!(0, self.moves.len());
            // self.moves = generate_psuedolegal_moves(self.board, MGT::CapturesOnly);
            self.moves
                .append(&generate_psuedolegal_moves(self.board, MGT::CapturesOnly));
            self.moves.score_move_list(self.board, self.tt_move, &self.killers);
        }

        if self.phase == MovePickerPhase::Captures {
            if let Some(e) = self.moves.get_one(self.processed_idx) {
                if e.score >= 1 {
                    self.processed_idx += 1;
                    return Some(e.m);
                }
            }
            self.phase = MovePickerPhase::Killer1;
        }

        if !self.gen_quiets {
            return None;
        }

        if self.phase == MovePickerPhase::Killer1 {
            self.phase = MovePickerPhase::Killer2;
            if self.killers[0].is_valid(self.board) && self.killers[0] != self.tt_move {
                return Some(self.killers[0]);
            }
        }

        if self.phase == MovePickerPhase::Killer2 {
            self.phase = MovePickerPhase::QuietsInit;
            if self.killers[1].is_valid(self.board) && self.killers[1] != self.tt_move {
                return Some(self.killers[1]);
            }
        }

        if self.phase == MovePickerPhase::QuietsInit {
            self.phase = MovePickerPhase::Quiets;
            self.processed_idx = self.moves.len();
            // self.moves = generate_psuedolegal_moves(self.board, MGT::QuietsOnly);
            self.moves
                .append(&generate_psuedolegal_moves(self.board, MGT::QuietsOnly));
            self.moves.score_move_list(self.board, self.tt_move, &self.killers);
        }

        if self.phase == MovePickerPhase::Quiets {
            let m = self.moves.get_one(self.processed_idx).map(|entry| entry.m);
            self.processed_idx += 1;
            return m;
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

#[allow(dead_code)]
pub fn perft(board: Board, depth: i32) -> usize {
    let mut total = 0;
    let moves = MovePicker::new(&board, 0, Move::NULL, &empty_killers());
    for m in moves {
        let mut new_b = board.to_owned();
        new_b.make_move(m);
        if new_b.in_check(board.to_move) {
            continue;
        }
        let count = count_moves(depth - 1, &new_b);
        total += count;
        println!("{}: {}", m.to_lan(), count);
    }
    println!("\nNodes searched: {}", total);
    total
}

/// Recursively counts the number of moves down to a certain depth
fn count_moves(depth: i32, board: &Board) -> usize {
    if depth == 0 {
        return 1;
    }
    let mut count = 0;
    let moves = MovePicker::new(board, 0, Move::NULL, &empty_killers());
    for m in moves {
        let mut new_b = board.to_owned();
        new_b.make_move(m);
        if new_b.in_check(board.to_move) {
            continue;
        }
        count += count_moves(depth - 1, &new_b);
    }
    count
}

#[cfg(test)]
mod move_picker_tests {
    use crate::{
        board::fen::{self, build_board},
        moves::{
            movelist::MoveList,
            movepicker::{perft, MovePicker},
            moves::{from_lan, Move},
        },
        search::killers::NUM_KILLER_MOVES,
    };

    #[test]
    fn test_movegenerator() {
        let board = build_board(fen::STARTING_FEN);
        let gen = MovePicker::dummy(&board);
        let mut count = 0;
        for m in gen {
            assert_ne!(m, Move::NULL);
            assert!(m.is_valid(&board));
            dbg!(m.to_lan());
            count += 1;
        }
        assert_eq!(20, count);
    }

    #[test]
    fn full_generator() {
        let board = build_board("k7/2r3n1/8/3q1b2/4P3/8/3N4/KR6 w - - 0 1");
        let gen = MovePicker {
            phase: super::MovePickerPhase::TTMove,
            moves: MoveList::default(),
            processed_idx: 0,
            tt_move: from_lan("e4d5", &board),
            board: &board,
            killers: [from_lan("b1b5", &board), from_lan("b1c1", &board)],
            gen_quiets: true,
        };
        for m in gen {
            assert!(m.is_valid(&board));
        }
    }

    #[test]
    fn null_moves() {
        let board = build_board("k7/2r3n1/8/3q1b2/8/8/3N4/KR6 w - - 0 1");
        let gen = MovePicker {
            phase: super::MovePickerPhase::TTMove,
            moves: MoveList::default(),
            processed_idx: 0,
            tt_move: Move::NULL,
            board: &board,
            killers: [Move::NULL; NUM_KILLER_MOVES],
            gen_quiets: true,
        };
        for m in gen {
            assert!(m.is_valid(&board));
        }
    }

    #[test]
    fn test_starting_pos() {
        let board = build_board(fen::STARTING_FEN);
        assert_eq!(119_060_324, perft(board, 6));
    }

    #[test]
    fn test_position_2() {
        let board = build_board("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -");
        assert_eq!(193_690_690, perft(board, 5));
    }

    #[test]
    fn test_position_3() {
        let board = build_board("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - -");
        assert_eq!(11_030_083, perft(board, 6));
    }

    #[test]
    fn test_position_4() {
        let board = build_board("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1");
        assert_eq!(706_045_033, perft(board, 6));
    }

    #[test]
    fn test_position_5() {
        let board = build_board("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8");
        assert_eq!(89_941_194, perft(board, 5));
    }

    #[test]
    fn test_position_6() {
        let board = build_board("r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10");
        assert_eq!(164_075_551, perft(board, 5));
    }

    // http://www.rocechess.ch/perft.html
    #[test]
    fn test_position_7() {
        let board = build_board("n1n5/PPPk4/8/8/8/8/4Kppp/5N1N b - - 0 1");
        assert_eq!(71_179_139, perft(board, 6));
    }
}
