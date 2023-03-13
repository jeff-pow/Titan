use std::sync::Mutex;
use std::time::Instant;

use crate::board::Board;
use crate::moves::{generate_all_moves, Move, in_check, Castle, Promotion};
use crate::pieces::{INFINITY, PieceName};

pub const IN_CHECK_MATE: i32 = 10000000;

pub fn time_move_search(board: &Board, depth: i32) {
    for i in 1..=depth {
        let start = Instant::now();
        print!("{}", count_moves(i, board));
        let elapsed = start.elapsed();
        print!(" moves generated in {:?} ", elapsed);
        println!("at a depth of {i}");
    }
}

pub fn perft(board: &Board, depth: i32) -> usize {
    let mut board = board.clone();
    let mut total = 0;
    let moves = generate_all_moves(&mut board);
    for m in &moves {
        let mut new_b = board.clone();
        new_b.make_move(m);
        let count = count_moves(depth - 1, &new_b);
        total += count;
        println!("{} {}", m.to_lan(), count);
    }
    println!("\n{}", total);
    total
}

fn count_moves(depth: i32, board: &Board) -> usize {
    let mut board = board.clone();
    if depth == 0 {
        return 1;
    }
    let mut count = 0;
    let moves = generate_all_moves(&mut board);
    for m in &moves {
        let mut new_b = board.clone();
        new_b.make_move(m);
        count += count_moves(depth - 1, &new_b);
    }
    count
}

static mut BEST_MOVE: Move = Move {
    starting_idx: -1,
    end_idx: -1,
    castle: Castle::None,
    promotion: Promotion::None,
    piece_moving: PieceName::Pawn,
    capture: None,
};

pub fn search(board: &Board, depth: i32) -> Move {
    for i in 1..=depth {
        let start = Instant::now();
        search_helper(board, i, 0, -INFINITY, INFINITY);
        let elapsed_time = start.elapsed().as_millis();
        println!("info time {} depth {}", elapsed_time, i);
    }
    unsafe { BEST_MOVE }
}

fn search_helper(board: &Board, depth: i32, dist_from_root: i32, mut alpha: i32, beta: i32) -> i32 {
    if depth == 0 {
        return board.evaluation();
    }
    let mut board = board.clone();
    let moves = generate_all_moves(&mut board);
    if moves.is_empty() {
        // Determine if empty moves means stalemate or checkmate
        if in_check(&board, board.to_move) {
            return -IN_CHECK_MATE;
        }
        return 0;
    }
    let mut best_move_for_pos = Move::invalid();
    for m in &moves {
        let mut new_b = board.clone();
        new_b.make_move(m);
        let eval = -search_helper(&new_b, depth - 1, dist_from_root + 1, -beta, -alpha);
        if eval >= beta {
            return beta;
        }
        if eval > alpha {
            best_move_for_pos = *m;
            alpha = eval;
            if dist_from_root == 0 {
                unsafe {BEST_MOVE = best_move_for_pos; }
            }
        }
    }
    alpha
}
