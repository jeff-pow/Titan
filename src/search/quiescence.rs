use crate::board::lib::Board;
use crate::board::zobrist::check_for_3x_repetition;
use crate::moves::{lib::Move, movegenerator::generate_psuedolegal_captures};
use crate::search::alpha_beta::STALEMATE;

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

    let mut moves = generate_psuedolegal_captures(board);
    moves.sort_unstable_by_key(|m| score_move(board, m));
    moves.reverse();

    for m in moves.iter() {
        let mut best_node_moves = Vec::new();
        let mut new_b = board.to_owned();
        new_b.make_move(m);
        // Just generate psuedolegal moves to save computation time on checks for moves that will be
        // pruned
        if new_b.side_in_check(board.to_move) {
            continue;
        }
        new_b.add_to_history();

        // Draw if a position has occurred three times
        if check_for_3x_repetition(&new_b) {
            return STALEMATE;
        }

        let eval = -quiescence(
            ply + 1,
            -beta,
            -alpha,
            &mut best_node_moves,
            search_info,
            &new_b,
        );

        new_b.remove_from_history();

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
