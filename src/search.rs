use std::time::Instant;

use crate::board::Board;
use crate::moves::{generate_all_moves, check_check, Move};
use crate::pieces::{get_piece_value, Color};

pub fn time_move_search(board: &Board, depth: i32) {
    for i in 1..depth {
        let start = Instant::now();
        print!("{}", count_moves(i, board));
        let elapsed = start.elapsed();
        print!(" moves generated in {:?} ", elapsed);
        println!("at a depth of {i}");
    }
}

pub fn perft(board: &Board, depth: i32) -> usize {
    let mut board = *board;
    let mut total = 0;
    let mut moves = generate_all_moves(&board);
    check_check(&mut board, &mut moves);
    for m in &moves {
        let mut new_b = board;
        new_b.make_move(m);
        let count = count_moves(depth - 1, &new_b);
        total += count;
        println!("{} {}", m.to_lan(), count);
    }
    println!("\n{}", total);
    total
}

fn count_moves(depth: i32, board: &Board) -> usize {
    let mut board = *board;
    if depth == 0 {
        return 1;
    }
    let mut count = 0;
    let mut moves = generate_all_moves(&board);
    check_check(&mut board, &mut moves);
    for m in &moves {
        let mut new_b = board;
        new_b.make_move(m);
        count += count_moves(depth - 1, &new_b);
    }
    count
}

pub fn search_moves(board: &Board, depth: i32) -> Move {
    let mut best_score = i32::MIN;
    let mut new_board = *board;
    let mut moves = generate_all_moves(&new_board);
    check_check(&mut new_board, &mut moves);
    let mut best_move = moves[0];
    for m in &moves {
        let mut new_b = *board;
        new_b.make_move(m);
        let pts = -search_helper(board, depth);
        if pts > best_score {
            best_score = pts;
            best_move = *m;
        }
    }
    best_move
}

fn search_helper(board: &Board, depth: i32) -> i32 {
    let mut best_score = i32::MIN;
    if depth == 0 {
        return board.evaluation();
    }
    let mut new_board = *board;
    let mut moves = generate_all_moves(&new_board);
    check_check(&mut new_board, &mut moves);
    if moves.is_empty() {
        return 0;
    }
    for m in &moves {
        let mut new_b = *board;
        new_b.make_move(m);
        best_score = i32::max(-search_helper(board, depth - 1), best_score);
    }
    best_score
}

