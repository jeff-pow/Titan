mod moves;
mod pieces;
mod uci;
mod search;

use std::process::exit;
use board::Board;
use fen::build_board;
use pieces::Piece;
#[allow(unused_imports)]
use search::*;
use uci::main_loop;
use crate::{moves::generate_all_moves, search::time_move_search};


mod board;
mod fen;

fn main() {
    let board = build_board(fen::STARTING_FEN);
    // r1bqkb1r/pppppppp/8/4P2Q/1nPN1P2/2N5/P1P3PP/R1B1K2R b KQk - 0 13
    main_loop();
}

#[allow(dead_code)]
fn print_moves(board: &Board) {
    println!("{}", board);
    let mut board = board.clone();
    let moves = generate_all_moves(&mut board);
    let i = moves.len();
    for m in moves.iter() {
        println!("{}", m);
        let mut cloned_board = board.clone();
        cloned_board.make_move(m);
        println!("{}", cloned_board);
        println!("---------------------------------------------------------");
    }
    println!("{} moves possible pre check", i);
    println!("{} moves possible post check", moves.len());
    exit(0);
}

#[cfg(test)]
mod move_number_tests {
    use crate::{search::perft, fen::{build_board, self}};

    // Positions and expected values from https://www.chessprogramming.org/Perft_Results

    #[test]
    fn test_starting_pos() {
        let board = build_board(fen::STARTING_FEN);
        assert_eq!(119060324, perft(&board, 6));
    }

    #[test]
    fn test_position_2() {
        let board = build_board("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -");
        assert_eq!(193690690, perft(&board, 5));
    }

    #[test]
    fn test_position_3() {
        let board = build_board("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - -");
        assert_eq!(11030083, perft(&board, 6));
    }

    #[test]
    fn test_position_4() {
        let board = build_board("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1");
        assert_eq!(15833292, perft(&board, 5));
    }

    #[test]
    fn test_position_5() {
        let board = build_board("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8");
        assert_eq!(89941194, perft(&board, 5));
    }

    #[test]
    fn test_position_6() {
        let board = build_board("r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10");
        assert_eq!(164075551, perft(&board, 5));
    }
}
