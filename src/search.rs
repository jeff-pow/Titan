use crate::{fen, moves::{generate_all_moves, check_check, Move}, board::Board};

pub fn search_moves(board: &Board, depth: i32) -> Move {
    let mut best_score = 0.;
    let mut best_move = Move::new();
    let mut new_board = board.clone();
    let mut moves = generate_all_moves(&new_board);
    check_check(&mut new_board, &mut moves);
    for m in &moves {
        let mut new_b = board.clone();
        new_b.make_move(m);
        let pts = search_helper(board, depth - 1);
        if best_score < m.pts + pts {
            best_score = m.pts + pts;
            best_move = m.clone();
        }
    }
    best_move
}

fn search_helper(board: &Board, depth: i32) -> f32 {
    if depth == 0 {
        return 0.;
    }
    let mut best_score = 0.;
    let mut new_board = board.clone();
    let mut moves = generate_all_moves(&new_board);
    check_check(&mut new_board, &mut moves);
    for m in &moves {
        let mut new_b = board.clone();
        new_b.make_move(m);
        let pts = search_helper(board, depth - 1);
        if best_score < m.pts + pts {
            best_score = m.pts + pts;
        }
    }
    best_score
}

pub fn count_moves(depth: i32, board: &Board) -> usize {
    let mut board = board.clone();
    if depth == 0 {
        return 1;
    }
    let mut count = 0;
    let mut moves = generate_all_moves(&board);
    check_check(&mut board, &mut moves);
    for m in &moves {
        let mut new_b = board.clone();
        new_b.make_move(m);
        count += count_moves(depth - 1, &new_b);
    }
    count
}
