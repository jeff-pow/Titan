mod attack_boards;
mod bit_hacks;
mod eval;
mod magics;
mod moves;
mod pieces;
mod search;
mod uci;
mod zobrist;

#[allow(unused_imports)]
// use crate::{moves::generate_all_moves, search::time_move_generation};
use board::Board;

#[allow(unused_imports)]
use search::*;
use std::process::exit;

use crate::moves::generate_legal_moves;

mod board;
mod fen;

fn main() {
    let board = fen::build_board("8/6P1/3r4/3P4/2P5/6b1/1P4P1/8 w - - 0 1");
    dbg!(board.occupancy());
    let board = fen::build_board(fen::STARTING_FEN);
    dbg!(board.occupancy());
    let _b = attack_boards::AttackBoards::new();
    print_moves(&board);
    uci::main_loop();
}

#[allow(dead_code)]
fn print_moves(board: &Board) {
    println!("{}", board);
    let bb = attack_boards::AttackBoards::new();
    let moves = generate_legal_moves(board, &bb);
    let i = moves.len();
    for m in moves.iter() {
        println!("{}", m);
        let mut cloned_board = *board;
        cloned_board.make_move(m, &bb);
        println!("{}", cloned_board);
        println!("---------------------------------------------------------");
    }
    println!("{} moves possible pre check", i);
    println!("{} moves possible post check", moves.len());
    exit(0);
}

#[cfg(test)]
mod move_number_tests {
    use crate::{
        attack_boards::AttackBoards,
        fen::{self, build_board},
        search::perft,
    };

    // Positions and expected values from https://www.chessprogramming.org/Perft_Results

    #[test]
    fn test_starting_pos() {
        let board = build_board(fen::STARTING_FEN);
        let bb = AttackBoards::new();
        assert_eq!(119060324, perft(&board, &bb, 6));
    }

    #[test]
    fn test_position_2() {
        let board = build_board("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -");
        let bb = AttackBoards::new();
        assert_eq!(193690690, perft(&board, &bb, 5));
    }

    #[test]
    fn test_position_3() {
        let board = build_board("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - -");
        let bb = AttackBoards::new();
        assert_eq!(11030083, perft(&board, &bb, 6));
    }

    #[test]
    fn test_position_4() {
        let board = build_board("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1");
        let bb = AttackBoards::new();
        assert_eq!(15833292, perft(&board, &bb, 5));
    }

    #[test]
    fn test_position_5() {
        let board = build_board("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8");
        let bb = AttackBoards::new();
        assert_eq!(89941194, perft(&board, &bb, 5));
    }

    #[test]
    fn test_position_6() {
        let board =
            build_board("r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10");
        let bb = AttackBoards::new();
        assert_eq!(164075551, perft(&board, &bb, 5));
    }
}
