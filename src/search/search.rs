use std::cmp::{max, min};
use std::sync::atomic::Ordering;
use std::time::Instant;

use crate::board::board::Board;
use crate::engine::transposition::{EntryFlag, TranspositionTable};
use crate::moves::movegenerator::MGT;
use crate::moves::movelist::{MoveListEntry, BAD_CAPTURE};
use crate::moves::moves::Move;
use crate::search::SearchStack;
use crate::spsa::{
    ASP_DIVISOR, ASP_MIN_DEPTH, CAPT_SEE, DBL_EXT_MARGIN, DELTA_EXPANSION, EXT_DEPTH,
    EXT_TT_DEPTH_MARGIN, INIT_ASP, LMP_DEPTH, LMP_IMP_BASE, LMP_IMP_FACTOR, LMP_NOT_IMP_BASE,
    LMP_NOT_IMP_FACTOR, LMR_MIN_MOVES, MAX_DBL_EXT, NMP_BASE_R, NMP_DEPTH, NMP_DEPTH_DIVISOR,
    NMP_EVAL_DIVISOR, NMP_EVAL_MIN, QUIET_SEE, RFP_BETA_FACTOR, RFP_DEPTH, RFP_IMPROVING_FACTOR,
    SEE_DEPTH,
};

use super::history_table::MAX_HIST_VAL;
use super::quiescence::quiescence;
use super::thread::ThreadData;
use super::{get_reduction, SearchType, PV};

pub const CHECKMATE: i32 = 25000;
pub const STALEMATE: i32 = 0;
pub const NEAR_CHECKMATE: i32 = CHECKMATE - 1000;
pub const INFINITY: i32 = 30000;
pub const MAX_SEARCH_DEPTH: i32 = 100;

pub fn search(td: &mut ThreadData, print_uci: bool, board: Board, tt: &TranspositionTable) -> Move {
    td.game_time.search_start = Instant::now();
    td.root_color = board.to_move;
    td.nodes_searched = 0;
    td.stack = SearchStack::default();

    let best_move = iterative_deepening(td, &board, print_uci, tt);

    assert_ne!(best_move, Move::NULL);

    best_move
}

pub(crate) fn iterative_deepening(
    td: &mut ThreadData,
    board: &Board,
    print_uci: bool,
    tt: &TranspositionTable,
) -> Move {
    let mut pv = PV::default();
    let mut best_move = Move::NULL;
    let mut prev_score = -INFINITY;

    for depth in 1..=td.max_depth {
        td.iter_max_depth = depth;
        td.ply = 0;
        td.sel_depth = 0;

        prev_score = aspiration_windows(td, &mut pv, prev_score, board, tt);

        best_move = pv.line[0];

        if print_uci {
            td.print_search_stats(prev_score, &pv, tt);
        }

        if td.search_type == SearchType::Time && td.game_time.soft_termination() {
            break;
        }

        if td.halt.load(Ordering::SeqCst) {
            break;
        }
    }

    assert_ne!(best_move, Move::NULL);
    best_move
}

fn aspiration_windows(
    td: &mut ThreadData,
    pv: &mut PV,
    prev_score: i32,
    board: &Board,
    tt: &TranspositionTable,
) -> i32 {
    let mut alpha = -INFINITY;
    let mut beta = INFINITY;
    // Asp window should start wider if score is more extreme
    let mut delta = INIT_ASP.val() + prev_score * prev_score / ASP_DIVISOR.val();

    let mut depth = td.iter_max_depth;

    if td.iter_max_depth >= ASP_MIN_DEPTH.val() {
        alpha = alpha.max(prev_score - delta);
        beta = beta.min(prev_score + delta);
    }

    loop {
        assert_eq!(0, td.ply);
        let score = alpha_beta::<true>(depth, alpha, beta, pv, td, tt, board, false);

        if score <= alpha {
            beta = (alpha + beta) / 2;
            alpha = max(score - delta, -INFINITY);
            // If move/position proves to not be as good as we thought, we need to do a full depth
            // search to get the best idea possible of its actual score.
            depth = td.iter_max_depth;
        } else if score >= beta {
            beta = min(score + delta, INFINITY);
            // If window is better than beta, we have a potentially untrustworthy best move that we
            // want to prove is safe quickly, so we reduce depth.
            depth -= 1;
        } else {
            return score;
        }
        delta += delta * DELTA_EXPANSION.val() / 3;
    }
}

/// Principal variation search - uses reduced alpha beta windows around a likely best move candidate
/// to refute other variations
#[allow(clippy::too_many_arguments)]
fn alpha_beta<const IS_PV: bool>(
    mut depth: i32,
    mut alpha: i32,
    beta: i32,
    pv: &mut PV,
    td: &mut ThreadData,
    tt: &TranspositionTable,
    board: &Board,
    cut_node: bool,
) -> i32 {
    let is_root = td.ply == 0;
    let in_check = board.in_check;
    let singular_move = td.stack[td.ply].singular;
    let singular_search = singular_move != Move::NULL;

    if IS_PV {
        td.sel_depth = td.sel_depth.max(td.ply);
    }

    // Stop if we have reached hard time limit or decided else where it is time to stop
    if td.halt.load(Ordering::Relaxed) || td.game_time.hard_termination() {
        td.halt.store(true, Ordering::SeqCst);
        // return board.evaluate();
        return 0;
    }

    if depth <= 0 {
        return quiescence::<IS_PV>(alpha, beta, pv, td, tt, board);
    }

    // Don't prune at root to ensure we have a best move
    if !is_root {
        if board.is_draw() || td.is_repetition(board) {
            // TODO: Try returning 2 - nodes & 3 to avoid 3x rep blindness
            return STALEMATE;
        }

        // Prevent overflows of the search stack
        if td.ply >= MAX_SEARCH_DEPTH {
            return if in_check { 0 } else { board.evaluate() };
        }

        // Determines if there is a faster path to checkmate than evaluating the current node, and
        // if there is, it returns early
        let alpha = alpha.max(-CHECKMATE + td.ply);
        let beta = beta.min(CHECKMATE - td.ply - 1);
        if alpha >= beta {
            return alpha;
        }

        // Extend depth by one if we are in check
        depth += i32::from(in_check);
    }

    let mut tt_move = Move::NULL;
    let mut tt_flag = EntryFlag::None;
    let mut tt_score = -INFINITY;
    let mut tt_depth = -1;
    let entry = tt.get(board.zobrist_hash, td.ply);
    if let Some(entry) = entry {
        tt_flag = entry.flag();
        tt_score = entry.search_score();
        tt_move = entry.best_move(board);
        tt_depth = entry.depth();

        // Don't do TT cutoffs in verification search for singular moves
        if !singular_search
            && !IS_PV
            && !is_root
            && depth <= entry.depth()
            && match tt_flag {
                EntryFlag::None => false,
                EntryFlag::Exact => true,
                EntryFlag::AlphaUnchanged => tt_score <= alpha,
                EntryFlag::BetaCutOff => tt_score >= beta,
            }
        {
            return tt_score;
        }
    } else if depth >= 4 && !IS_PV && !singular_search {
        // IIR (Internal Iterative Deepening) - Reduce depth if a node doesn't have a TT hit and isn't a
        // PV node
        // TODO: Unlink IIR from the entry existing - just check if tt move is null instead
        depth -= 1;
    }

    let mut best_score = -INFINITY;
    let mut best_move = Move::NULL;
    let original_alpha = alpha;

    let static_eval = if in_check {
        -CHECKMATE
    } else if let Some(entry) = entry {
        // Get static eval from transposition table if possible
        entry.static_eval()
    } else {
        board.evaluate()
    };
    td.stack[td.ply].static_eval = static_eval;
    let improving = !in_check && td.ply > 1 && static_eval > td.stack[td.ply - 2].static_eval;

    // TODO: Killers should probably be reset here
    // td.stack[td.ply + 1].killers = [Move::NULL; 2];
    if td.ply < MAX_SEARCH_DEPTH {
        td.stack[td.ply + 1].singular = Move::NULL
    }
    if !is_root {
        td.stack[td.ply].dbl_extns = td.stack[td.ply - 1].dbl_extns;
    }

    // Pre-move loop pruning
    if !is_root && !IS_PV && !in_check && !singular_search {
        // Reverse futility pruning - If we are below beta by a certain amount, we are unlikely to
        // raise it, so we can prune the nodes that would have followed
        if static_eval - RFP_BETA_FACTOR.val() * depth
            + i32::from(improving) * RFP_IMPROVING_FACTOR.val() * depth
            >= beta
            && depth < RFP_DEPTH.val()
            && static_eval.abs() < NEAR_CHECKMATE
        {
            return static_eval;
        }

        // Null move pruning (NMP) - If we can give the opponent a free move and they still can't
        // raise beta, they likely won't be able to, so we can prune the nodes that would have
        // followed
        if board.has_non_pawns(board.to_move)
            && depth >= NMP_DEPTH.val()
            && static_eval >= beta
            && td.stack[td.ply - 1].played_move != Move::NULL
        {
            let mut node_pv = PV::default();
            let mut new_b = *board;

            new_b.make_null_move();
            td.stack[td.ply].played_move = Move::NULL;
            td.hash_history.push(new_b.zobrist_hash);
            td.ply += 1;

            // Reduction
            let r = NMP_BASE_R.val()
                + depth / NMP_DEPTH_DIVISOR.val()
                + min((static_eval - beta) / NMP_EVAL_DIVISOR.val(), NMP_EVAL_MIN.val());
            let mut null_eval = -alpha_beta::<false>(
                depth - r,
                -beta,
                -beta + 1,
                &mut node_pv,
                td,
                tt,
                &new_b,
                !cut_node,
            );

            td.hash_history.pop();
            td.ply -= 1;

            if null_eval >= beta {
                // Ensure we don't return a checkmate score
                if null_eval > NEAR_CHECKMATE {
                    null_eval = beta;
                }
                return null_eval;
            }
        }
    }

    let mut moves = board.generate_moves(MGT::All);
    let mut legal_moves_searched = 0;
    moves.score_moves(board, tt_move, td.stack[td.ply].killers, td);

    let mut quiets_tried = Vec::new();
    let mut tacticals_tried = Vec::new();

    // Start of search
    for MoveListEntry { m, score: hist_score } in moves {
        if m == singular_move {
            continue;
        }
        let is_quiet = !m.is_tactical(board);

        if !is_root && best_score >= -NEAR_CHECKMATE {
            if is_quiet {
                // Late move pruning (LMP)
                // By now all good tactical moves have been searched, so we can prune
                // If eval is improving, we want to search more
                let moves_required = if improving {
                    LMP_IMP_BASE.val() as f32 / 100.
                        + LMP_IMP_FACTOR.val() as f32 / 100. * depth as f32 * depth as f32
                } else {
                    LMP_NOT_IMP_BASE.val() as f32 / 100.
                        + LMP_NOT_IMP_FACTOR.val() as f32 / 100. * depth as f32 * depth as f32
                } as i32;
                if depth < LMP_DEPTH.val() && legal_moves_searched > moves_required {
                    break;
                }
            }
            // Static exchange pruning - If we fail to immediately recapture a depth dependent
            // threshold, don't bother searching the move
            // TODO: Try a depth * depth dependent capture threshold
            let margin =
                if m.is_capture(board) { -CAPT_SEE.val() } else { -QUIET_SEE.val() } * depth;
            if depth < SEE_DEPTH.val() && !board.see(m, margin) {
                continue;
            }
        }

        let mut new_b = *board;
        // Make move filters out illegal moves by returning false if a move was illegal
        if !new_b.make_move::<true>(m) {
            continue;
        }
        tt.prefetch(new_b.zobrist_hash);

        if is_quiet {
            quiets_tried.push(m)
        } else {
            tacticals_tried.push(m)
        };

        let extension = if tt_depth >= depth - EXT_TT_DEPTH_MARGIN.val()
            && tt_flag != EntryFlag::AlphaUnchanged
            && tt_flag != EntryFlag::None
            && m == tt_move
            && !singular_search
            && depth >= EXT_DEPTH.val()
            && !is_root
        {
            let ext_beta = (tt_score - EXT_TT_DEPTH_MARGIN.val() * depth).max(-CHECKMATE);
            let ext_depth = (depth - 1) / 2;
            let mut node_pv = PV::default();

            td.stack[td.ply].singular = m;
            let ext_score = alpha_beta::<false>(
                ext_depth,
                ext_beta - 1,
                ext_beta,
                &mut node_pv,
                td,
                tt,
                board,
                cut_node,
            );
            td.stack[td.ply].singular = Move::NULL;

            if ext_score < ext_beta {
                if td.stack[td.ply].dbl_extns <= MAX_DBL_EXT.val()
                    && !IS_PV
                    && ext_score < ext_beta - DBL_EXT_MARGIN.val()
                {
                    td.stack[td.ply].dbl_extns = td.stack[td.ply - 1].dbl_extns + 1;
                    2
                } else {
                    1
                }
            } else if tt_score >= beta {
                -2 + i32::from(IS_PV)
            } else {
                0
            }
        } else {
            0
        };

        let new_depth = depth + extension;

        td.nodes_searched += 1;
        td.stack[td.ply].played_move = m;
        td.hash_history.push(new_b.zobrist_hash);
        td.ply += 1;
        let mut node_pv = PV::default();

        let eval = if legal_moves_searched == 0 {
            // On the first move, just do a full depth search
            -alpha_beta::<IS_PV>(new_depth - 1, -beta, -alpha, &mut node_pv, td, tt, &new_b, false)
        } else {
            // Late Move Reductions - Search moves after the first with reduced depth and window as
            // they are much less likely to be the best move than the first move selected by the
            // move picker.
            let r = {
                if legal_moves_searched < LMR_MIN_MOVES.val() || depth < 2 {
                    1
                } else {
                    let mut r = get_reduction(depth, legal_moves_searched);
                    r += i32::from(!IS_PV);
                    r += i32::from(!improving);
                    if is_quiet && cut_node {
                        r += 2;
                    }
                    if is_quiet {
                        if hist_score > MAX_HIST_VAL / 2 {
                            r -= 1;
                        } else if hist_score < -MAX_HIST_VAL / 2 {
                            r += 1;
                        }
                    }
                    if m.is_capture(board) && hist_score < BAD_CAPTURE + 100 {
                        r += 1;
                    }
                    min(new_depth, max(r, 1))
                }
            };

            // Start with a zero window reduced search
            let zero_window_reduced_depth = -alpha_beta::<false>(
                new_depth - r,
                -alpha - 1,
                -alpha,
                &mut node_pv,
                td,
                tt,
                &new_b,
                !cut_node,
            );

            // If that search raises alpha and a reduction was applied, re-search at a zero window with full depth
            let zero_window_full_depth = if zero_window_reduced_depth > alpha && r > 1 {
                -alpha_beta::<false>(
                    new_depth - 1,
                    -alpha - 1,
                    -alpha,
                    &mut node_pv,
                    td,
                    tt,
                    &new_b,
                    !cut_node,
                )
            } else {
                zero_window_reduced_depth
            };

            // If the verification score falls between alpha and beta, full window full depth search
            if zero_window_full_depth > alpha && zero_window_full_depth < beta {
                -alpha_beta::<IS_PV>(
                    new_depth - 1,
                    -beta,
                    -alpha,
                    &mut node_pv,
                    td,
                    tt,
                    &new_b,
                    false,
                )
            } else {
                zero_window_full_depth
            }
        };

        legal_moves_searched += 1;
        td.hash_history.pop();
        td.ply -= 1;

        if eval > best_score {
            best_score = eval;

            if eval > alpha {
                alpha = eval;
                best_move = m;
                pv.update(best_move, node_pv);
            }

            if alpha >= beta {
                // Prefetch here since we're going to want to write to the tt for this board in a
                // few lines anyway
                tt.prefetch(board.zobrist_hash);

                if is_quiet {
                    // We don't want to store tactical moves in our killer moves, because they are obviously already
                    // good.
                    // Also don't store killers that we have already stored
                    if td.stack[td.ply].killers[0] != m {
                        td.stack[td.ply].killers[1] = td.stack[td.ply].killers[0];
                        td.stack[td.ply].killers[0] = m;
                    }
                }
                // Update history tables on a beta cutoff
                td.history.update_histories(
                    m,
                    &quiets_tried,
                    &tacticals_tried,
                    board,
                    depth,
                    &td.stack,
                    td.ply,
                );
                break;
            }
        }
    }

    if legal_moves_searched == 0 {
        // TODO: Return alpha if singular verification search
        if board.in_check {
            // Distance from root is returned in order for other recursive calls to determine
            // shortest viable checkmate path
            return -CHECKMATE + td.ply;
        }
        return STALEMATE;
    }

    let entry_flag = if best_score >= beta {
        EntryFlag::BetaCutOff
    } else if best_score > original_alpha {
        EntryFlag::Exact
    } else {
        EntryFlag::AlphaUnchanged
    };

    // Don't save to TT while in a singular extension verification search
    if !singular_search {
        // TODO: Don't update best move if upper bound
        tt.store(
            board.zobrist_hash,
            best_move,
            depth,
            entry_flag,
            best_score,
            td.ply,
            IS_PV,
            static_eval,
        );
    }

    best_score
}
