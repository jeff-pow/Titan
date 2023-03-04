use crate::{fen, moves::{generate_all_moves, check_check, Move}, board::Board};

pub fn search_moves(board: &Board, depth: i32) -> Move {
    let mut new_board = board.clone();
    search_helper(&mut new_board, depth).0
}

fn search_helper(board: &mut Board, depth: i32) -> (Move, f32) {
    let mut best_score = 0.;
    let mut best_move = None;
    let mut moves = generate_all_moves(&board);
    //check_check(&mut board, &mut moves);
    for m in &moves {
        let mut new_b = board.clone();
        new_b.make_move(m);
        let (_, pts) = search_helper(board, depth - 1);
        if best_score < m.pts + pts {
            best_score = m.pts + pts;
            best_move = Some(m);
        }
    }
    (best_move.expect("Best Move was None").to_owned(), best_score)
}

pub fn count_moves(a: i32, board: &Board) -> usize {
    let mut board = board.clone();
    if depth == 0 {
        return 1;
    }
    let mut moves = generate_all_moves(&board);
    check_check(&mut board, &mut moves);
    for m in &moves {
        let mut new_b = board.clone();
        new_b.make_move(m);
        count += recur_depth_moves(a - 1, &new_b);
    }
    count += moves.len() as u128;
    count
}
