use crate::board::board::Board;
use crate::engine::transposition::{EntryFlag, TableEntry};
use crate::moves::movegenerator::generate_psuedolegal_moves;
use crate::moves::moves::Move;
use crate::search::eval::eval;
use crate::search::killers::store_killer_move;
use crate::search::pvs::{
    CHECKMATE, EXT_FUTIL_DEPTH, EXT_FUTIL_MARGIN, FUTIL_DEPTH, FUTIL_MARGIN, INFINITY,
    MAX_SEARCH_DEPTH, RAZORING_DEPTH, RAZOR_MARGIN, STALEMATE,
};
use crate::search::quiescence::quiescence;
use crate::search::SearchInfo;

pub fn alpha_beta(
    mut depth: i8,
    mut alpha: i32,
    beta: i32,
    pv: &mut Vec<Move>,
    search_info: &mut SearchInfo,
    board: &Board,
) -> i32 {
    let ply = search_info.iter_max_depth - depth;
    let is_root = ply == 0;
    search_info.sel_depth = search_info.sel_depth.max(ply);
    // Needed since the function can calculate extensions in cases where it finds itself in check
    if ply >= MAX_SEARCH_DEPTH {
        if board.side_in_check(board.to_move) {
            return quiescence(ply, alpha, beta, pv, search_info, board);
        }
        return eval(board);
    }

    if board.is_draw() {
        return STALEMATE;
    }
    if ply > 0 {
        // Determines if there is a faster path to checkmate than evaluating the current node, and
        // if there is, it returns early
        let alpha = alpha.max(-CHECKMATE + ply as i32);
        let beta = beta.min(CHECKMATE - ply as i32);
        if alpha >= beta {
            return alpha;
        }
    }

    let (table_value, table_move) = {
        let hash = board.zobrist_hash;
        let entry = search_info.transpos_table.get(&hash);
        if let Some(entry) = entry {
            entry.get(depth, ply, alpha, beta)
        } else {
            (None, Move::NULL)
        }
    };
    if let Some(eval) = table_value {
        if !is_root {
            return eval;
        }
    }

    let in_check = board.side_in_check(board.to_move);

    if in_check {
        depth += 1;
    }

    if depth <= 0 {
        return quiescence(ply, alpha, beta, pv, search_info, board);
    }

    search_info.search_stats.nodes_searched += 1;

    let mut moves = generate_psuedolegal_moves(board);
    let mut legal_moves = 0;
    moves.score_move_list(ply, board, table_move, search_info);

    let mut score = -INFINITY;
    let mut entry_flag = EntryFlag::AlphaCutOff;
    let mut best_move = Move::NULL;

    if !in_check && depth == FUTIL_DEPTH && search_info.iter_max_depth > FUTIL_DEPTH {
        let eval = eval(board);
        if eval + FUTIL_MARGIN < alpha {
            return quiescence(ply, alpha, beta, pv, search_info, board);
        }
    }

    if !in_check && depth == EXT_FUTIL_DEPTH && search_info.iter_max_depth > EXT_FUTIL_DEPTH {
        let eval = eval(board);
        if eval + EXT_FUTIL_MARGIN < alpha {
            return quiescence(ply, alpha, beta, pv, search_info, board);
        }
    }

    if !in_check && depth == RAZORING_DEPTH && search_info.iter_max_depth > RAZORING_DEPTH {
        let eval = eval(board);
        if eval + RAZOR_MARGIN < alpha {
            return quiescence(ply, alpha, beta, pv, search_info, board);
        }
    }

    for i in 0..moves.len {
        let mut new_b = board.to_owned();
        moves.sort_next_move(i);
        let m = moves.get_move(i);
        new_b.make_move(m);
        let _s = m.to_lan();
        let _c = moves.get_score(i);
        // Just generate psuedolegal moves to save computation time on legality for moves that will be
        // pruned
        if new_b.side_in_check(board.to_move) {
            continue;
        }
        legal_moves += 1;

        let mut node_pvs = Vec::new();
        let mut eval;

        // TODO: Test whether or not aspiration windows are worth doing with pvs search
        // do_pvs = false;
        eval = -alpha_beta(depth - 1, -beta, -alpha, &mut node_pvs, search_info, &new_b);

        if eval > score {
            score = eval;
            best_move = *m;
        }

        if eval >= beta {
            search_info.transpos_table.insert(
                board.zobrist_hash,
                TableEntry::new(depth, ply, EntryFlag::BetaCutOff, eval, best_move),
            );
            let capture = board.piece_at(m.dest_square());

            // Store a killer move if it is not a capture, but good enough to cause a beta cutoff
            // anyway
            if capture.is_none() {
                store_killer_move(ply, m, search_info);
            }
            return eval;
        }

        if eval > alpha {
            alpha = eval;
            entry_flag = EntryFlag::Exact;
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

    search_info.transpos_table.insert(
        board.zobrist_hash,
        TableEntry::new(depth, ply, entry_flag, score, best_move),
    );

    score
}
