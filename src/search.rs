use std::time::Instant;

use crate::board::Board;
use crate::eval::eval;
use crate::moves::{generate_all_moves, in_check, Castle, EnPassant, Move, Promotion};
use crate::pieces::PieceName;
use std::cmp::{max, min};

pub const IN_CHECK_MATE: i32 = 100000;
pub const INFINITY: i32 = 9999999;

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
    let mut total = 0;
    let moves = generate_all_moves(board);
    for m in &moves {
        let mut new_b = *board;
        new_b.make_move(m);
        let count = count_moves(depth - 1, &new_b);
        total += count;
        println!("{}: {}", m.to_lan(), count);
    }
    println!("\nNodes searched: {}", total);
    total
}

fn count_moves_with_undo(depth: i32, board: &mut Board) -> usize {
    if depth == 0 {
        return 1;
    }
    let mut count = 0;
    let moves = generate_all_moves(board);
    for m in &moves {
        board.make_move(m);
        count += count_moves(depth - 1, board);
        board.unmake_move(m);
    }
    count
}

fn count_moves(depth: i32, board: &Board) -> usize {
    if depth == 0 {
        return 1;
    }
    let mut count = 0;
    let moves = generate_all_moves(board);
    for m in &moves {
        let mut new_b = *board;
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
    en_passant: EnPassant::None,
};

pub fn old_search(board: &Board, depth: i32) -> Move {
    for i in 1..=depth {
        let start = Instant::now();
        old_search_helper(board, i, 0, -INFINITY, INFINITY);
        let elapsed_time = start.elapsed().as_millis();
        println!("info time {} depth {}", elapsed_time, i);
    }
    unsafe { BEST_MOVE }
}

fn old_search_helper(
    board: &Board,
    depth: i32,
    dist_from_root: i32,
    mut alpha: i32,
    beta: i32,
) -> i32 {
    if depth == 0 {
        //return board.position_eval();
        return eval(board);
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
                unsafe {
                    BEST_MOVE = best_move_for_pos;
                }
            }
        }
    }
    alpha
}

pub fn search(board: &Board, depth: i32) -> Move {
    let mut best_move = Move::invalid();

    for i in 1..=depth {
        let start = Instant::now();
        let mut alpha = -INFINITY;
        let beta = INFINITY;
        let moves = generate_all_moves(board);
        for m in &moves {
            let mut new_b = *board;
            new_b.make_move(m);
            let eval = -search_helper(board, i - 1, 1, -beta, -alpha);
            if eval > alpha {
                alpha = eval;
                best_move = *m;
            }
        }
        let elapsed_time = start.elapsed().as_millis();
        println!("info time {} depth {}", elapsed_time, i);
    }
    best_move
}

fn basic_search_helper(
    board: &Board,
    depth: i32,
    dist_from_root: i32,
    mut alpha: i32,
    mut beta: i32,
) -> i32 {
    if depth == 0 {
        return eval(board);
    }
    let moves = generate_all_moves(board);
    if moves.is_empty() {
        // Checkmate
        if in_check(board, board.to_move) {
            // Distance from root is returned in order for other recursive calls to determine
            // shortest viable checkmate path
            return -IN_CHECK_MATE;
        }
        // Stalemate
        return 0;
    }
    let mut max_value = -INFINITY;
    for m in &moves {
        let mut new_b = *board;
        new_b.make_move(m);
        let eval = -search_helper(&new_b, depth - 1, dist_from_root + 1, -beta, -alpha);
        max_value = max(max_value, eval);
        alpha = max(alpha, eval);
        if alpha >= beta {
            break;
        }
    }
    max_value
}

fn search_helper(
    board: &Board,
    depth: i32,
    dist_from_root: i32,
    mut alpha: i32,
    mut beta: i32,
) -> i32 {
    if depth == 0 {
        return eval(board);
    }
    // Skip move if a path to checkmate has already been found in this path
    alpha = max(alpha, -IN_CHECK_MATE + dist_from_root);
    beta = min(beta, IN_CHECK_MATE - dist_from_root);
    if alpha >= beta {
        return alpha;
    }
    let moves = generate_all_moves(board);
    if moves.is_empty() {
        // Checkmate
        if in_check(board, board.to_move) {
            // Distance from root is returned in order for other recursive calls to determine
            // shortest viable checkmate path
            return -IN_CHECK_MATE + dist_from_root;
        }
        // Stalemate
        return 0;
    }
    for m in &moves {
        let mut new_b = *board;
        new_b.make_move(m);
        let eval = -search_helper(&new_b, depth - 1, dist_from_root + 1, -beta, -alpha);
        if eval > alpha {
            alpha = eval;
        }
    }
    alpha
}
