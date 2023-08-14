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
) -> i32 {
    search_info.search_stats.nodes_searched += 1;
    let eval = eval(&search_info.board);

    if ply >= MAX_SEARCH_DEPTH {
        return eval;
    }

    if eval >= beta {
        return beta;
    }

    if eval > alpha {
        alpha = eval;
    }

    let mut moves = generate_capture_moves(&search_info.board);
    moves.sort_unstable_by_key(|m| score_move(&search_info.board, m));
    moves.reverse();

    for m in moves.iter() {
        let mut best_node_moves = Vec::new();
        let eval = -quiescence(ply + 1, -beta, -alpha, &mut best_node_moves, search_info);

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
