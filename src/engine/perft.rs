use std::sync::RwLock;

use rayon::prelude::{IntoParallelIterator, ParallelIterator};

use crate::{
    board::board::Board,
    moves::movegenerator::{generate_legal_moves, generate_moves, MGT},
};

pub fn multi_threaded_perft(board: Board, depth: i32) -> usize {
    let total = RwLock::new(0);
    let moves = generate_legal_moves(&board);

    (0..moves.len()).into_par_iter().for_each(|idx| {
        let m = moves[idx];
        let mut new_b = board.to_owned();
        assert!(new_b.make_move(m));
        let count = count_moves::<true>(depth - 1, &new_b);
        *total.write().unwrap() += count;
        println!("{}: {}", m.to_san(), count);
    });
    println!("\nNodes searched: {}", total.read().unwrap());

    let x = *total.read().unwrap();
    x
}

pub fn non_bulk_perft(board: Board, depth: i32) -> usize {
    if depth == 0 {
        return 1;
    }
    let mut total = 0;
    let moves = generate_moves(&board, MGT::All);
    for i in 0..moves.len() {
        let m = moves[i];
        let mut new_b = board.to_owned();
        if !new_b.make_move(m) {
            continue;
        }
        let count = non_bulk_perft(new_b, depth - 1);
        total += count;
    }
    total
}

pub fn perft<const BULK: bool>(board: Board, depth: i32) -> usize {
    let mut total = 0;
    let moves = generate_legal_moves(&board);
    // for MoveListEntry { m, .. } in moves {
    for i in 0..moves.len() {
        let m = moves[i];
        let mut new_b = board.to_owned();
        assert!(new_b.make_move(m));
        let count = count_moves::<BULK>(depth - 1, &new_b);
        total += count;
        println!("{}: {}", m.to_san(), count);
    }
    println!("\nNodes searched: {}", total);
    total
}

/// Recursively counts the number of moves down to a certain depth
pub fn count_moves<const BULK: bool>(depth: i32, board: &Board) -> usize {
    let mut count = 0;
    let moves = generate_legal_moves(board);
    assert!(depth >= 0);

    if depth == 1 && BULK {
        return moves.len();
    }
    if depth == 0 {
        return 1;
    }

    for i in 0..moves.len() {
        let m = moves[i];
        let mut new_b = board.to_owned();
        assert!(new_b.make_move(m));
        count += count_moves::<BULK>(depth - 1, &new_b);
    }
    count
}

#[cfg(test)]
mod movegen_tests {
    use std::{fs::File, io::BufRead, io::BufReader};

    use rayon::iter::IndexedParallelIterator;
    use rayon::iter::ParallelIterator;
    use rayon::prelude::IntoParallelRefIterator;

    use crate::engine::perft::non_bulk_perft;
    use crate::{board::fen::build_board, engine::perft::perft};

    #[test]
    pub fn epd_perft() {
        let file = BufReader::new(File::open("./src/engine/ethereal_perft.epd").expect("File not found"));
        let vec = file.lines().collect::<Vec<_>>();
        vec.par_iter().enumerate().for_each(|(test_num, line)| {
            let l = line.as_ref().unwrap().clone();
            let vec = l.split(" ;").collect::<Vec<&str>>();
            let mut iter = vec.iter();
            let board = build_board(iter.next().unwrap());
            for entry in iter {
                let (depth, nodes) = entry.split_once(' ').unwrap();
                let depth = depth[1..].parse::<i32>().unwrap();
                let nodes = nodes.parse::<usize>().unwrap();
                eprintln!("test {test_num}: depth {depth} expected {nodes}");
                assert_eq!(nodes, perft::<true>(board, depth));
            }
            eprintln!("{test_num} passed");
        });
    }
}
