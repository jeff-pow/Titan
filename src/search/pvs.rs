use std::cmp::{max, min};
use std::time::{Duration, Instant};

use crate::board::board::Board;
use crate::engine::transposition::{EntryFlag, TableEntry};
use crate::moves::movegenerator::generate_psuedolegal_moves;
use crate::moves::moves::Move;
use crate::search::{alpha_beta, eval};
use crate::types::pieces::{PieceName, QUEEN_PTS, ROOK_PTS};

use super::eval::eval;
use super::killers::store_killer_move;
use super::quiescence::quiescence;
use super::{SearchInfo, SearchType};

pub const CHECKMATE: i32 = 25000;
pub const STALEMATE: i32 = 0;
pub const NEAR_CHECKMATE: i32 = CHECKMATE - 1000;
pub const INFINITY: i32 = 32000;
pub const MAX_SEARCH_DEPTH: i8 = 100;

pub fn pvs_search(search_info: &mut SearchInfo) -> Move {
    let mut best_move = Move::NULL;
    let mut pv_moves = Vec::new();

    let mut recommended_time = Duration::ZERO;
    match search_info.search_type {
        SearchType::Time => {
            recommended_time = search_info
                .game_time
                .recommended_time(search_info.board.to_move);
        }
        SearchType::Depth => (),
        SearchType::Infinite => {
            search_info.iter_max_depth = MAX_SEARCH_DEPTH;
            search_info.max_depth = MAX_SEARCH_DEPTH;
        }
    }

    search_info.search_stats.start = Instant::now();
    search_info.iter_max_depth = 1;

    while search_info.iter_max_depth <= search_info.max_depth {
        search_info.sel_depth = search_info.iter_max_depth;

        let board = &search_info.board.to_owned();
        let eval = pvs(
            search_info.iter_max_depth,
            -INFINITY,
            INFINITY,
            &mut pv_moves,
            search_info,
            board,
        );

        if !pv_moves.is_empty() {
            best_move = pv_moves[0];
        }
        print_search_stats(search_info, eval, &pv_moves, search_info.iter_max_depth);

        if search_info.search_type == SearchType::Time
            && search_info
                .game_time
                .reached_termination(search_info.search_stats.start, recommended_time)
        {
            break;
        }
        search_info.iter_max_depth += 1;
    }

    assert_ne!(best_move, Move::NULL);

    best_move
}

pub fn print_search_stats(search_info: &SearchInfo, eval: i32, pv: &[Move], iter_depth: i8) {
    print!(
        "info time {} seldepth {} depth {} nodes {} nps {} score cp {} pv ",
        search_info.search_stats.start.elapsed().as_millis(),
        search_info.sel_depth,
        iter_depth,
        search_info.search_stats.nodes_searched,
        (search_info.search_stats.nodes_searched as f64
            / search_info.search_stats.start.elapsed().as_secs_f64()) as i64,
        eval
    );
    for m in pv.iter() {
        print!("{} ", m.to_lan());
    }
    println!();
}

pub fn asp_pvs(search_info: &mut SearchInfo, mut max_depth: i8) -> Move {
    let mut best_move = Move::NULL;
    let mut pv_moves = Vec::new();

    let mut recommended_time = Duration::ZERO;
    match search_info.search_type {
        SearchType::Time => {
            recommended_time = search_info
                .game_time
                .recommended_time(search_info.board.to_move);
        }
        SearchType::Depth => (),
        SearchType::Infinite => {
            max_depth = MAX_SEARCH_DEPTH;
            search_info.max_depth = max_depth;
        }
    }
    search_info.search_stats.start = Instant::now();

    let mut alpha;
    let mut beta;
    // The previous eval from this side (two moves ago) is a good place to estimate the next
    // aspiration window around. First depth will not have an estimate, and we will do a full
    // window search
    let mut score_history = vec![eval(&search_info.board)];
    search_info.iter_max_depth = 1;

    while search_info.iter_max_depth <= max_depth {
        search_info.sel_depth = 0;
        let board = &search_info.board.to_owned();

        let prev_avg = if search_info.iter_max_depth >= 2 {
            *score_history
                .get(search_info.iter_max_depth as usize - 2)
                .unwrap() as f64
        } else {
            -INFINITY as f64
        };
        let mut delta = 10 + (prev_avg * prev_avg * 6.25e-4) as i32;
        alpha = max(prev_avg as i32 - delta, -INFINITY);
        beta = min(prev_avg as i32 + delta, INFINITY);

        let mut score;
        loop {
            score = pvs(
                search_info.iter_max_depth,
                alpha,
                beta,
                &mut pv_moves,
                search_info,
                board,
            );
            if score <= alpha {
                beta = (alpha + beta) / 2;
                alpha = max(score - delta, -INFINITY);
            } else if score >= beta {
                beta = min(score + delta, INFINITY);
            } else {
                break;
            }
            delta += delta / 3;
            debug_assert!(alpha >= -INFINITY && beta <= INFINITY);
        }

        if !pv_moves.is_empty() {
            best_move = pv_moves[0];
        }
        score_history.push(score);

        print_search_stats(search_info, score, &pv_moves, search_info.iter_max_depth);

        if search_info.search_type == SearchType::Time
            && search_info
                .game_time
                .reached_termination(search_info.search_stats.start, recommended_time)
        {
            break;
        }
        search_info.iter_max_depth += 1;
    }

    assert_ne!(best_move, Move::NULL);

    best_move
}

pub const FUTIL_MARGIN: i32 = 200;
pub const FUTIL_DEPTH: i8 = 1;
pub const EXT_FUTIL_MARGIN: i32 = ROOK_PTS;
pub const EXT_FUTIL_DEPTH: i8 = 2;
pub const RAZOR_MARGIN: i32 = QUEEN_PTS;
pub const RAZORING_DEPTH: i8 = 3;

/// Principal variation search - uses reduced alpha beta windows around a likely best move candidate
/// to refute other variations
pub(crate) fn pvs(
    mut depth: i8,
    mut alpha: i32,
    beta: i32,
    pv: &mut Vec<Move>,
    search_info: &mut SearchInfo,
    board: &Board,
) -> i32 {
    let ply = search_info.iter_max_depth - depth;
    let is_root = ply == 0;
    let in_check = board.side_in_check(board.to_move);
    let can_prune = !in_check;
    search_info.sel_depth = search_info.sel_depth.max(ply);
    // Don't do pvs unless you have a pv - otherwise you're wasting time
    let mut do_pvs = false;
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
    // IIR (Internal Iterative Deepening) - Reduce depth if a node doesn't have a TT eval, isn't a
    // PV node, and is a cutNode(?)
    else if depth >= 4 {
        depth -= 2;
    }

    if in_check {
        depth += 1;
    }

    if depth <= 0 {
        return quiescence(ply, alpha, beta, pv, search_info, board);
    }

    let mut best_score = -INFINITY;
    let mut entry_flag = EntryFlag::AlphaCutOff;
    let mut best_move = Move::NULL;

    // Futility pruning
    if can_prune && depth == FUTIL_DEPTH && search_info.iter_max_depth > FUTIL_DEPTH {
        let eval = eval(board);
        if eval + FUTIL_MARGIN < alpha {
            return quiescence(ply, alpha, beta, pv, search_info, board);
        }
    }

    // Extended futility pruning
    if can_prune && depth == EXT_FUTIL_DEPTH && search_info.iter_max_depth > EXT_FUTIL_DEPTH {
        let eval = eval(board);
        if eval + EXT_FUTIL_MARGIN < alpha {
            return quiescence(ply, alpha, beta, pv, search_info, board);
        }
    }

    // Razoring
    if can_prune && depth == RAZORING_DEPTH && search_info.iter_max_depth > RAZORING_DEPTH {
        let eval = eval(board);
        if eval + RAZOR_MARGIN < alpha {
            return quiescence(ply, alpha, beta, pv, search_info, board);
        }
    }

    // Null pruning
    if can_prune && !board.side_in_check(board.to_move) && null_ok(board) {
        let mut node_pvs = Vec::new();
        let mut new_b = board.to_owned();
        new_b.to_move = new_b.to_move.opp();
        let null_eval = -pvs(
            depth - 1,
            -beta,
            -beta + 1,
            &mut node_pvs,
            search_info,
            &new_b,
        );
        if null_eval >= beta {
            return null_eval;
        }
    }

    search_info.search_stats.nodes_searched += 1;
    // Just generate psuedolegal moves to save computation time on legality for moves that will be
    // pruned
    let mut moves = generate_psuedolegal_moves(board);
    let mut legal_moves_searched = 0;
    moves.score_move_list(ply, board, table_move, search_info);

    // Start of search
    for i in 0..moves.len {
        let mut new_b = board.to_owned();
        moves.sort_next_move(i);
        let m = moves.get_move(i);
        new_b.make_move(m);
        if new_b.side_in_check(board.to_move) {
            continue;
        }
        legal_moves_searched += 1;

        let mut node_pvs = Vec::new();
        let mut eval;

        // LMR
        let mut do_full_search = false;
        if depth > 2
            && legal_moves_searched > 2
            && (!(m.is_capture(&new_b) || m.promotion().is_some()))
        {
            let depth = depth - 2;
            let val = -pvs(
                depth,
                -alpha - 1,
                -alpha,
                &mut Vec::new(),
                search_info,
                &new_b,
            );
            if val > alpha {
                do_full_search = true;
            }
        }
        if do_pvs && do_full_search {
            eval = -pvs(
                depth - 1,
                -alpha - 1,
                -alpha,
                &mut node_pvs,
                search_info,
                &new_b,
            );
            if eval > alpha && alpha < beta {
                eval = -pvs(depth - 1, -beta, -alpha, &mut node_pvs, search_info, &new_b);
            }
        } else {
            eval = -pvs(depth - 1, -beta, -alpha, &mut node_pvs, search_info, &new_b);
        }

        if eval > best_score {
            best_score = eval;
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
            best_move = *m;
            entry_flag = EntryFlag::Exact;
            // A principal variation has been found, so we can do pvs on the remaining nodes of this level
            do_pvs = true;
            pv.clear();
            pv.push(*m);
            pv.append(&mut node_pvs);
        }
    }

    if legal_moves_searched == 0 {
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
        TableEntry::new(depth, ply, entry_flag, best_score, best_move),
    );

    best_score
}

/// Arbitrary value determining if a side is in endgame yet
const ENDGAME_THRESHOLD: i32 =
    PieceName::Queen.value() + PieceName::Rook.value() + PieceName::Bishop.value();
fn null_ok(board: &Board) -> bool {
    board.material_val[board.to_move as usize] > ENDGAME_THRESHOLD
        && board.material_val[board.to_move.opp() as usize] > ENDGAME_THRESHOLD
}
