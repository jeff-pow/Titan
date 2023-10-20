use std::{sync::RwLock, time::Instant};

use rayon::prelude::{IntoParallelIterator, ParallelIterator};

use crate::{
    board::board::Board,
    moves::{movegenerator::generate_moves, movelist::MoveListEntry},
};

#[allow(dead_code)]
/// Counts and times the action of generating moves to a certain depth. Prints this information
pub fn time_move_generation(board: &Board, depth: i32) {
    for i in 1..=depth {
        let start = Instant::now();
        print!("{}", count_moves(i, board));
        let elapsed = start.elapsed();
        print!(" moves generated in {:?} ", elapsed);
        println!("at a depth of {i}");
    }
}

pub fn multi_threaded_perft(board: Board, depth: i32) -> usize {
    let total = RwLock::new(0);
    let moves = generate_moves(&board);

    moves.into_vec().into_par_iter().for_each(|m| {
        let mut new_b = board.to_owned();
        new_b.make_move(m);
        let count = count_moves(depth - 1, &new_b);
        *total.write().unwrap() += count;
        println!("{}: {}", m.to_lan(), count);
    });
    println!("\nNodes searched: {}", total.read().unwrap());

    let x = *total.read().unwrap();
    x
}

pub fn perft(board: Board, depth: i32) -> usize {
    let mut total = 0;
    let moves = generate_moves(&board);
    for MoveListEntry { m, .. } in moves {
        let mut new_b = board.to_owned();
        new_b.make_move(m);
        let count = count_moves(depth - 1, &new_b);
        total += count;
        println!("{}: {}", m.to_lan(), count);
    }
    println!("\nNodes searched: {}", total);
    total
}

/// Recursively counts the number of moves down to a certain depth
pub fn count_moves(depth: i32, board: &Board) -> usize {
    let mut count = 0;
    let moves = generate_moves(board);

    if depth == 1 {
        return moves.len();
    }
    for MoveListEntry { m, .. } in moves {
        let mut new_b = board.to_owned();
        new_b.make_move(m);
        count += count_moves(depth - 1, &new_b);
    }
    count
}

#[cfg(test)]
mod movegen_tests {
    // Positions and expected values from https://www.chessprogramming.org/Perft_Results
    use crate::{
        board::fen::{self, build_board},
        engine::perft::{multi_threaded_perft, perft},
    };

    #[test]
    fn test_starting_pos() {
        let board = build_board(fen::STARTING_FEN);
        assert_eq!(119_060_324, multi_threaded_perft(board, 6));
    }

    #[test]
    fn test_position_2() {
        let board = build_board("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -");
        assert_eq!(193_690_690, multi_threaded_perft(board, 5));
    }

    #[test]
    fn test_position_3() {
        let board = build_board("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - -");
        assert_eq!(11_030_083, multi_threaded_perft(board, 6));
    }

    #[test]
    fn test_position_4() {
        let board = build_board("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1");
        assert_eq!(706_045_033, multi_threaded_perft(board, 6));
    }

    #[test]
    fn test_position_5() {
        let board = build_board("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8");
        assert_eq!(89_941_194, multi_threaded_perft(board, 5));
    }

    #[test]
    fn test_position_6() {
        let board = build_board("r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10");
        assert_eq!(164_075_551, perft(board, 5));
    }

    #[test]
    fn test_multithread() {
        let board = build_board("r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10");
        assert_eq!(164_075_551, multi_threaded_perft(board, 5));
    }

    // http://www.rocechess.ch/perft.html
    #[test]
    fn test_position_7() {
        let board = build_board("n1n5/PPPk4/8/8/8/8/4Kppp/5N1N b - - 0 1");
        assert_eq!(71_179_139, multi_threaded_perft(board, 6));
    }
}
