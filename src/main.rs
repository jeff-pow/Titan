mod board;
mod engine;
mod init;
mod moves;
mod search;
mod types;

use board::board::Board;

use crate::engine::uci::main_loop;
use crate::init::init;
use crate::moves::moves::generate_moves;

fn main() {
    init();
    let board = board::fen::build_board(board::fen::STARTING_FEN);
    let mut searcher = search::search::Search::new();
    println!("{}", searcher.search(&board, 7));
    main_loop();
}

#[allow(dead_code)]
fn print_moves(board: &Board) {
    println!("{}", board);
    let moves = generate_moves(board);
    for m in moves.iter() {
        println!("{}", m);
        let mut cloned_board = *board;
        cloned_board.make_move(m);
        println!("{}", cloned_board);
        println!("---------------------------------------------------------");
    }
    println!("{} moves found", moves.len());
}

#[cfg(test)]
mod move_number_tests {
    use crate::{
        board::fen::{self, build_board},
        init::init,
        search::search::perft,
    };

    // Positions and expected values from https://www.chessprogramming.org/Perft_Results

    #[test]
    fn test_starting_pos() {
        init();
        let board = build_board(fen::STARTING_FEN);
        assert_eq!(119_060_324, perft(&board, 6));
    }

    #[test]
    fn test_position_2() {
        init();
        let board = build_board("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -");
        assert_eq!(193_690_690, perft(&board, 5));
    }

    #[test]
    fn test_position_3() {
        init();
        let board = build_board("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - -");
        assert_eq!(11_030_083, perft(&board, 6));
    }

    #[test]
    fn test_position_4() {
        init();
        let board = build_board("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1");
        assert_eq!(706_045_033, perft(&board, 6));
    }

    #[test]
    fn test_position_5() {
        init();
        let board = build_board("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8");
        assert_eq!(89_941_194, perft(&board, 5));
    }

    #[test]
    fn test_position_6() {
        init();
        let board =
            build_board("r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10");
        assert_eq!(164_075_551, perft(&board, 5));
    }

    // http://www.rocechess.ch/perft.html
    #[test]
    fn test_position_7() {
        init();
        let board = build_board("n1n5/PPPk4/8/8/8/8/4Kppp/5N1N b - - 0 1");
        assert_eq!(71_179_139, perft(&board, 6));
    }
}
