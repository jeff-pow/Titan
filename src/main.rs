mod eval;
mod moves;
mod pieces;
mod search;
mod uci;
mod zobrist;

#[allow(unused_imports)]
use crate::{moves::generate_all_moves, search::time_move_search};
use board::Board;
use pieces::Piece;
#[allow(unused_imports)]
use search::*;
use std::process::exit;
use uci::main_loop;

mod board;
mod fen;

fn main() {
    main_loop();
    // position fen r3k2r/pp3p2/2p2pp1/4p3/2P5/3R1P2/PP3P1P/2K3NR b kq - 0 15 moves h8h4 g1e2 h4c4 c1d2 c4a4 d3a3 a4a3 b2a3 e8c8 d2c2 c8c7 h1d1
    // r1bqkb1r/pppppppp/8/4P2Q/1nPN1P2/2N5/P1P3PP/R1B1K2R b KQk - 0 13
    // rnb1k1nr/2pp1ppp/1p6/2bP4/p2NqP2/8/PPP3PP/RNBQ1BKR b kq - 5 10
    // K1k3n1/8/7p/1q5p/8/8/6p1/8 b - - 1 51
}

#[allow(dead_code)]
fn print_moves(board: &Board) {
    println!("{}", board);
    let moves = generate_all_moves(board);
    let i = moves.len();
    for m in moves.iter() {
        println!("{}", m);
        let mut cloned_board = *board;
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
    use crate::{
        fen::{self, build_board},
        search::perft,
    };

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
        let board =
            build_board("r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10");
        assert_eq!(164075551, perft(&board, 5));
    }
}
