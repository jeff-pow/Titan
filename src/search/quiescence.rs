use crate::board::lib::Board;
use crate::moves::{lib::Move, movegenerator::generate_psuedolegal_captures};
use crate::search::alpha_beta::STALEMATE;

use super::alpha_beta::{score_move_list, sort_next_move};
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
    // Draw if a position has occurred three times
    if board.is_draw() {
        return STALEMATE;
    }

    search_info.search_stats.nodes_searched += 1;
    search_info.sel_depth = search_info.sel_depth.max(ply);
    let eval = eval(board);
    if ply >= MAX_SEARCH_DEPTH {
        return eval;
    }
    // Give the engine the chance to stop capturing here if it results in a better end result than continuing the chain of capturing
    if eval >= beta {
        return beta;
    }
    if eval > alpha {
        alpha = eval;
    }

    let mut moves = generate_psuedolegal_captures(board);
    score_move_list(board, &mut moves, Move::NULL);

    for i in 0..moves.len {
        let mut best_node_moves = Vec::new();
        let mut new_b = board.to_owned();
        sort_next_move(&mut moves, i);
        let m = moves.get_move(i);
        new_b.make_move(m);
        // Just generate psuedolegal moves to save computation time on checks for moves that will be
        // pruned
        if new_b.side_in_check(board.to_move) {
            continue;
        }

        let eval = -quiescence(
            ply + 1,
            -beta,
            -alpha,
            &mut best_node_moves,
            search_info,
            &new_b,
        );

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
