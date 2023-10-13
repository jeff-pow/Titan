use std::{sync::RwLock, time::Instant};

use rayon::prelude::{IntoParallelIterator, ParallelIterator};

use crate::{board::board::Board, moves::movegenerator::generate_moves};

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
    for m in moves {
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
fn count_moves(depth: i32, board: &Board) -> usize {
    let mut count = 0;
    let moves = generate_moves(board);
    if depth == 1 {
        return moves.len;
    }
    for m in moves {
        let mut new_b = board.to_owned();
        new_b.make_move(m);
        count += count_moves(depth - 1, &new_b);
    }
    count
}
