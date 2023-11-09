use std::sync::RwLock;

use rayon::prelude::{IntoParallelIterator, ParallelIterator};

use crate::{board::board::Board, moves::movegenerator::generate_legal_moves};

pub fn multi_threaded_perft(board: Board, depth: i32) -> usize {
    let total = RwLock::new(0);
    let moves = generate_legal_moves(&board);

    (0..moves.len()).into_par_iter().for_each(|idx| {
        let m = moves[idx];
        let mut new_b = board.to_owned();
        assert!(new_b.make_move(m));
        let count = count_moves(depth - 1, &new_b);
        *total.write().unwrap() += count;
        println!("{}: {}", m.to_san(), count);
    });
    println!("\nNodes searched: {}", total.read().unwrap());

    let x = *total.read().unwrap();
    x
}

pub fn perft(board: Board, depth: i32) -> usize {
    let mut total = 0;
    let moves = generate_legal_moves(&board);
    // for MoveListEntry { m, .. } in moves {
    for i in 0..moves.len() {
        let m = moves[i];
        let mut new_b = board.to_owned();
        assert!(new_b.make_move(m));
        let count = count_moves(depth - 1, &new_b);
        total += count;
        println!("{}: {}", m.to_san(), count);
    }
    println!("\nNodes searched: {}", total);
    total
}

/// Recursively counts the number of moves down to a certain depth
pub fn count_moves(depth: i32, board: &Board) -> usize {
    let mut count = 0;
    let moves = generate_legal_moves(board);
    assert!(depth >= 0);

    if depth == 1 {
        return moves.len();
    }
    if depth == 0 {
        return 1;
    }

    for i in 0..moves.len() {
        let m = moves[i];
        let mut new_b = board.to_owned();
        assert!(new_b.make_move(m));
        count += count_moves(depth - 1, &new_b);
    }
    count
}

#[cfg(test)]
mod movegen_tests {
    use std::{fs::File, io::BufRead, io::BufReader};

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

    #[test]
    pub fn epd_perft() {
        let file = BufReader::new(File::open("./src/engine/ethereal_perft.epd").expect("File not found"));
        for (test_num, line) in file.lines().enumerate() {
            let l = line.unwrap().clone();
            let vec = l.split(" ;").collect::<Vec<&str>>();
            let mut iter = vec.iter();
            let board = build_board(iter.next().unwrap());
            for entry in iter {
                let (depth, nodes) = entry.split_once(' ').unwrap();
                let depth = depth[1..].parse::<i32>().unwrap();
                let nodes = nodes.parse::<usize>().unwrap();
                assert_eq!(nodes, multi_threaded_perft(board, depth));
            }
            println!("{test_num} passed");
        }
    }
}
