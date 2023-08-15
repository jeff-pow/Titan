use crate::board::board::Board;
use crate::engine::transposition::{add_to_history, remove_from_history};
use crate::moves::{movegenerator::generate_capture_moves, moves::Move};

use super::{
    alpha_beta::{score_move, MAX_SEARCH_DEPTH},
    eval::eval,
    SearchInfo,
};

pub fn quiescence(
    ply: i8,
    mut alpha: i32,
    beta: i32,
    best_moves: &mut Vec<Move>,
    search_info: &mut SearchInfo,
    board: &Board,
) -> i32 {
    search_info.search_stats.nodes_searched += 1;
    let eval = eval(board);

    if ply >= MAX_SEARCH_DEPTH {
        return eval;
    }

    if eval >= beta {
        return beta;
    }

    if eval > alpha {
        alpha = eval;
    }

    let mut moves = generate_capture_moves(board);
    moves.sort_unstable_by_key(|m| score_move(board, m));
    moves.reverse();

    for m in moves.iter() {
        let mut best_node_moves = Vec::new();
        let mut new_b = board.to_owned();
        new_b.make_move(m);
        add_to_history(&mut new_b);

        let eval = -quiescence(
            ply + 1,
            -beta,
            -alpha,
            &mut best_node_moves,
            search_info,
            &new_b,
        );

        remove_from_history(&mut new_b);

        if eval >= beta {
            return beta;
        }

        if eval > alpha {
            alpha = eval;

            best_moves.clear();
            best_moves.push(*m);
            best_moves.append(&mut best_node_moves);
        }
    }

    alpha
}
