use crate::board::board::Board;
use crate::moves::movegenerator::generate_psuedolegal_moves;
use crate::moves::moves::Move;
use crate::search::eval::evaluate;
use crate::search::pvs::{MAX_SEARCH_DEPTH, STALEMATE};
use crate::search::quiescence::quiescence;
use crate::search::SearchInfo;

use super::pvs::{CHECKMATE, INFINITY};

#[allow(dead_code)]
pub fn alpha_beta(
    depth: i8,
    mut alpha: i32,
    beta: i32,
    pv: &mut Vec<Move>,
    search_info: &mut SearchInfo,
    board: &Board,
) -> i32 {
    let ply = search_info.iter_max_depth - depth;
    search_info.sel_depth = search_info.sel_depth.max(ply);
    if ply >= MAX_SEARCH_DEPTH {
        return evaluate(board);
    }

    if ply > 0 {
        if board.is_draw() {
            return STALEMATE;
        }
        // Determines if there is a faster path to checkmate than evaluating the current node, and
        // if there is, it returns early
        let alpha = alpha.max(-CHECKMATE + ply as i32);
        let beta = beta.min(CHECKMATE - ply as i32 - 1);
        if alpha >= beta {
            return alpha;
        }
    }

    if depth <= 0 {
        return quiescence(ply, alpha, beta, pv, search_info, board);
    }

    search_info.search_stats.nodes_searched += 1;

    let mut moves = generate_psuedolegal_moves(board);
    let mut legal_moves = 0;
    moves.score_move_list(ply, board, Move::NULL, search_info);

    let mut score = -INFINITY;

    for i in 0..moves.len {
        let mut new_b = board.to_owned();
        moves.sort_next_move(i);
        let m = moves.get_move(i);
        new_b.make_move(m);
        // Just generate psuedolegal moves to save computation time on legality for moves that will be
        // pruned
        if new_b.side_in_check(board.to_move) {
            continue;
        }
        legal_moves += 1;

        let mut node_pvs = Vec::new();

        let eval = -alpha_beta(depth - 1, -beta, -alpha, &mut node_pvs, search_info, &new_b);

        if eval > score {
            score = eval;
        }

        if eval >= beta {
            return eval;
        }

        if eval > alpha {
            alpha = eval;
            // A principal variation has been found, so we can do pvs on the remaining nodes of this level
            pv.clear();
            pv.push(*m);
            pv.append(&mut node_pvs);
        }
    }

    if legal_moves == 0 {
        // Checkmate
        if board.side_in_check(board.to_move) {
            // Distance from root is returned in order for other recursive calls to determine
            // shortest viable checkmate path
            return -CHECKMATE + ply as i32;
        }
        return STALEMATE;
    }

    score
}
