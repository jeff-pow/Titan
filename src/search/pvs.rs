use std::cmp::{max, min};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::board::board::Board;
use crate::engine::transposition::{EntryFlag, TableEntry};
use crate::eval::eval::evaluate;
use crate::moves::movegenerator::{generate_psuedolegal_moves, MGT};
use crate::moves::movepicker::MovePicker;
use crate::moves::moves::Move;

use super::killers::store_killer_move;
use super::quiescence::quiescence;
use super::see::see;
use super::{reduction, SearchInfo, SearchType};

pub const CHECKMATE: i32 = 30000;
pub const STALEMATE: i32 = 0;
pub const NEAR_CHECKMATE: i32 = CHECKMATE - 1000;
pub const INFINITY: i32 = 50000;
pub const MAX_SEARCH_DEPTH: i32 = 100;
/// Initial aspiration window value
pub const INIT_ASP: i32 = 10;

pub fn print_search_stats(search_info: &SearchInfo, eval: i32, pv: &[Move], iter_depth: i32) {
    print!(
        "info time {} seldepth {} depth {} nodes {} nps {} score cp {} pv ",
        search_info.search_stats.start.elapsed().as_millis(),
        search_info.sel_depth,
        iter_depth,
        search_info.search_stats.nodes_searched,
        (search_info.search_stats.nodes_searched as f64 / search_info.search_stats.start.elapsed().as_secs_f64())
            as i64,
        eval
    );
    for m in pv.iter() {
        print!("{} ", m.to_lan());
    }
    println!();
}

pub fn search(search_info: &mut SearchInfo, mut max_depth: i32, halt: Arc<AtomicBool>) -> Move {
    let mut best_move = Move::NULL;
    let mut pv_moves = Vec::new();

    let mut recommended_time = Duration::ZERO;
    match search_info.search_type {
        SearchType::Time => {
            recommended_time = search_info.game_time.recommended_time(search_info.board.to_move);
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
    let mut score_history = vec![evaluate(&search_info.board)];
    search_info.iter_max_depth = 1;

    while search_info.iter_max_depth <= max_depth {
        search_info.sel_depth = 0;
        let board = &search_info.board.to_owned();

        // We assume the average eval for the board from two iterations ago is a good estimate for
        // the next iteration
        let prev_avg = if search_info.iter_max_depth >= 2 {
            *score_history.get(search_info.iter_max_depth as usize - 2).unwrap() as f64
        } else {
            -INFINITY as f64
        };
        let mut delta = INIT_ASP + (prev_avg * prev_avg * 6.25e-5) as i32;
        alpha = max(prev_avg as i32 - delta, -INFINITY);
        beta = min(prev_avg as i32 + delta, INFINITY);

        let mut score;
        loop {
            score =
                pvs(search_info.iter_max_depth, alpha, beta, &mut pv_moves, search_info, board, false, halt.clone());
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
        if halt.load(Ordering::SeqCst) {
            break;
        }
        search_info.iter_max_depth += 1;
    }

    assert_ne!(best_move, Move::NULL);

    best_move
}

/// Principal variation search - uses reduced alpha beta windows around a likely best move candidate
/// to refute other variations
#[allow(clippy::too_many_arguments)]
fn pvs(
    mut depth: i32,
    mut alpha: i32,
    beta: i32,
    pv: &mut Vec<Move>,
    search_info: &mut SearchInfo,
    board: &Board,
    cut_node: bool,
    halt: Arc<AtomicBool>,
) -> i32 {
    let ply = search_info.iter_max_depth - depth;
    let is_root = ply == 0;
    let in_check = board.in_check(board.to_move);
    // Is a zero width search if alpha and beta are one apart
    let is_pv_node = (beta - alpha).abs() != 1;
    search_info.sel_depth = search_info.sel_depth.max(ply);
    // Don't do pvs unless you have a pv - otherwise you're wasting time
    if halt.load(Ordering::SeqCst) {
        return evaluate(board);
    }

    // if in_check {
    //     depth += 1;
    // }
    // Needed since the function can calculate extensions in cases where it finds itself in check
    if ply >= MAX_SEARCH_DEPTH {
        if board.in_check(board.to_move) {
            return quiescence(ply, alpha, beta, pv, search_info, board);
        }
        return evaluate(board);
    }

    if ply > 0 {
        if board.is_draw() {
            return STALEMATE;
        }
        // Determines if there is a faster path to checkmate than evaluating the current node, and
        // if there is, it returns early
        let alpha = alpha.max(-CHECKMATE + ply);
        let beta = beta.min(CHECKMATE - ply - 1);
        if alpha >= beta {
            return alpha;
        }
    }

    let (table_value, table_move) = {
        if let Some(entry) = search_info.transpos_table.read().unwrap().get(&board.zobrist_hash) {
            entry.get(depth, ply, alpha, beta)
        } else {
            (None, Move::NULL)
        }
    };
    if let Some(eval) = table_value {
        if !is_root {
            // This can cut off evals in certain cases, but it's easy to implement :)
            // pv.push(table_move);
            return eval;
        }
    }
    // IIR (Internal Iterative Deepening) - Reduce depth if a node doesn't have a TT eval, isn't a
    // PV node, and is a cutNode
    else if depth >= 4 && !is_pv_node {
        depth -= 1;
    }

    if depth <= 0 {
        return quiescence(ply, alpha, beta, pv, search_info, board);
    }

    let mut best_score = -INFINITY;
    let mut entry_flag = EntryFlag::AlphaUnchanged;
    let mut best_move = Move::NULL;
    let static_eval = evaluate(board);

    // TODO: Make sure we aren't at root here as well
    if !is_pv_node && !in_check {
        // Reverse futility pruning
        if static_eval - 70 * depth >= beta && depth < 9 && static_eval.abs() < NEAR_CHECKMATE {
            return static_eval;
        }

        // Null move pruning (NMP)
        if board.has_non_pawns(board.to_move) && depth >= 3 && static_eval >= beta && board.prev_move != Move::NULL {
            let mut node_pvs = Vec::new();
            let mut new_b = board.to_owned();
            new_b.to_move = !new_b.to_move;
            new_b.en_passant_square = None;
            new_b.prev_move = Move::NULL;
            let r = 3 + depth / 3 + min((static_eval - beta) / 200, 3);
            let mut null_eval =
                -pvs(depth - r, -beta, -beta + 1, &mut node_pvs, search_info, &new_b, !cut_node, halt.clone());
            if null_eval >= beta {
                if null_eval > NEAR_CHECKMATE {
                    null_eval = beta;
                }
                if search_info.nmp_plies == 0 || depth < 10 {
                    return null_eval;
                }
                search_info.nmp_plies = ply + (depth - r) / 3;
                let null_eval =
                    pvs(depth - r, beta - 1, beta, &mut Vec::new(), search_info, board, false, halt.clone());
                search_info.nmp_plies = 0;
                if null_eval >= beta {
                    return null_eval;
                }
            }
        }
    }

    // Just generate psuedolegal moves to save computation time on legality for moves that will be
    // pruned
    let mut moves = generate_psuedolegal_moves(board, MGT::All);
    let mut legal_moves_searched = 0;
    moves.score_move_list(board, table_move, &search_info.killer_moves[ply as usize]);
    search_info.search_stats.nodes_searched += 1;
    let moves = MovePicker::new(board, ply, table_move, &search_info.killer_moves);

    // Start of search
    for m in moves {
        let mut new_b = board.to_owned();
        let is_quiet = m.is_quiet(board);
        let s = m.to_lan();

        if !is_root && !is_pv_node && best_score >= -NEAR_CHECKMATE {
            if is_quiet {
                // Late move pruning (LMP)
                // By now all quiets have been searched.
                // TODO: Move the !is_pv_node to this check so SEE can be done on all nodes
                if depth < 6 && !in_check && legal_moves_searched > (3 + depth * depth) / 2 {
                    break;
                }

                // Quiet SEE pruning
                if depth <= 8 && !see(board, m, -50 * depth) {
                    continue;
                }
            } else {
                // Capture SEE pruning
                if depth <= 6 && !see(board, m, -15 * depth * depth) {
                    continue;
                }
            }
        }

        new_b.make_move(m);
        if new_b.in_check(board.to_move) {
            continue;
        }
        legal_moves_searched += 1;
        let mut node_pvs = Vec::new();
        let mut eval = -INFINITY;

        // LMR
        let do_full_search;
        if depth > 2 && legal_moves_searched > 1 {
            let mut r = 1;
            if is_quiet || !is_pv_node {
                r = reduction(depth, legal_moves_searched);
                if !is_pv_node {
                    r += 1;
                }
                // r += i32::from(!is_pv_node);
                if cut_node {
                    r += 2;
                }
                // if is_quiet && !see(&new_b, m, -50 * depth) {
                //     depth += 1;
                // }
            }
            r = r.clamp(1, depth - 1);
            eval = -pvs(depth - r, -alpha - 1, -alpha, &mut Vec::new(), search_info, &new_b, !cut_node, halt.clone());
            do_full_search = eval > alpha && r != 1;
        } else {
            do_full_search = !is_pv_node || legal_moves_searched < 2;
        }

        if do_full_search {
            node_pvs.clear();
            eval = -pvs(depth - 1, -alpha - 1, -alpha, &mut node_pvs, search_info, &new_b, !cut_node, halt.clone());
        }

        if is_pv_node && (legal_moves_searched == 1 || (eval > alpha && (is_root || eval < beta))) {
            node_pvs.clear();
            eval = -pvs(depth - 1, -beta, -alpha, &mut node_pvs, search_info, &new_b, false, halt.clone());
        }

        // There's some funky stuff going on here with returning alpha or best_score
        // They should be the same but aren't for some reason...
        // Also function performs considerably better when only updating the best move when it's above beta
        // which makes slightly more sense I think?
        // The case might be that the beta and alpha checks should only happen when the best score is raised
        // Currently it does better when you don't update the best move at all away from a null
        // move which makes zero sense...
        // Currently it is literally better not to update best move from Move::NULL than try to
        // update it at all, making me think my transposition table is goofed real good

        if eval > best_score {
            best_score = eval;
            best_move = m;

            if eval > alpha {
                alpha = eval;
                best_move = m;
                pv.clear();
                pv.push(m);
                pv.append(&mut node_pvs);
                entry_flag = EntryFlag::Exact;
            }

            if alpha >= beta {
                search_info
                    .transpos_table
                    .write()
                    .unwrap()
                    .insert(board.zobrist_hash, TableEntry::new(depth, ply, EntryFlag::BetaCutOff, eval, m));

                let capture = board.piece_at(m.dest_square());
                // Store a killer move if it is not a capture, but good enough to cause a beta cutoff
                if capture.is_none() {
                    store_killer_move(ply, m, search_info);
                }
                return alpha;
            }
        }
    }

    if legal_moves_searched == 0 {
        // Checkmate
        if board.in_check(board.to_move) {
            // Distance from root is returned in order for other recursive calls to determine
            // shortest viable checkmate path
            return -CHECKMATE + ply;
        }
        return STALEMATE;
    }

    search_info
        .transpos_table
        .write()
        .unwrap()
        .insert(board.zobrist_hash, TableEntry::new(depth, ply, entry_flag, alpha, best_move));

    // TODO: Fail soft instead of this mess...
    best_score
}
