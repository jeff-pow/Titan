use std::time::Instant;

use crate::board::Board;
use crate::chess_move::Move;
use crate::movelist::{MoveListEntry, MAX_MOVES};
use crate::movepicker::MovePicker;
use crate::search::SearchStack;
use crate::thread::ThreadData;
use crate::transposition::{EntryFlag, TranspositionTable};
use arrayvec::ArrayVec;

pub const MAX_PLY: usize = 128;

pub const STALEMATE: i32 = 0;
pub const CHECKMATE: i32 = 32000;
pub const INFINITY: i32 = 32001;
pub const NONE: i32 = 32002;

pub const MATE_IN_MAX_PLY: i32 = CHECKMATE - MAX_PLY as i32;
pub const MATED_IN_MAX_PLY: i32 = -CHECKMATE + MAX_PLY as i32;

pub fn mated_in(ply: usize) -> i32 {
    -CHECKMATE + ply as i32
}

pub fn mate_in(ply: usize) -> i32 {
    CHECKMATE - ply as i32
}

pub fn is_mate(score: i32) -> bool {
    score.abs() >= MATE_IN_MAX_PLY
}

pub const fn is_win(score: i32) -> bool {
    score >= MATE_IN_MAX_PLY
}

pub const fn is_loss(score: i32) -> bool {
    score <= MATED_IN_MAX_PLY
}

pub fn start_search(td: &mut ThreadData, print_uci: bool, board: Board, tt: &TranspositionTable) {
    td.search_start = Instant::now();
    td.nodes_table = [[0; 64]; 64];
    td.stack = SearchStack::default();
    td.pv.reset();
    td.accumulators.clear(board.new_accumulator());

    iterative_deepening(td, &board, print_uci, tt);
}

/// Rather than sticking to a fixed depth for search, gradually ramping up the search depth by one
/// level until time expires actually saves time. This method relies on earlier depth searches
/// finishing quickly, building up important structures like transposition and history tables along
/// the way. As a result, for more expensive depths, we already have a good idea of the best move
/// and can maximize the efficacy of alpha beta pruning.
pub fn iterative_deepening(td: &mut ThreadData, board: &Board, print_uci: bool, tt: &TranspositionTable) {
    let mut prev_score = NONE;
    let mut depth = 1;

    loop {
        td.sel_depth = 0;

        assert_eq!(0, td.ply);
        assert_eq!(0, td.accumulators.top);

        prev_score = negamax::<true>(td, tt, board, -INFINITY, INFINITY, depth);

        assert_eq!(0, td.accumulators.top);

        if td.halt() {
            break;
        }

        if td.soft_stop(depth, prev_score) {
            td.set_halt(true);
            break;
        }

        if print_uci {
            td.print_search_stats(prev_score, tt, depth);
        }

        depth += 1;
    }

    if print_uci {
        td.print_search_stats(prev_score, tt, depth);
    }
}

fn negamax<const PV: bool>(
    td: &mut ThreadData,
    tt: &TranspositionTable,
    board: &Board,
    mut alpha: i32,
    beta: i32,
    depth: i32,
) -> i32 {
    let is_root = td.ply == 0;
    let in_check = board.in_check();

    td.sel_depth = td.sel_depth.max(td.ply);
    if !is_root {
        td.pv.clear_depth(td.ply);
    }

    if td.halt() {
        return 0;
    }

    if td.main_thread() && td.hard_stop() {
        td.set_halt(true);
        return 0;
    }

    if td.ply >= MAX_PLY {
        return if in_check { 0 } else { td.accumulators.evaluate(board) };
    }

    if !is_root {
        if board.is_draw() || td.is_repetition(board, td.ply) {
            return STALEMATE;
        }

        // Mate Distance Pruning - Determines if there is a faster path to checkmate
        // than evaluating the current node, and if there is, it returns early
        let alpha = alpha.max(mated_in(td.ply));
        let beta = beta.min(mate_in(td.ply));
        if alpha >= beta {
            return alpha;
        }
    }

    if depth <= 0 {
        return qsearch::<PV>(td, tt, board, alpha, beta);
    }

    td.nodes.increment();

    let mut tt_move = Move::NULL;
    if let Some(entry) = tt.get(board.zobrist_hash, td.ply) {
        tt_move = entry.best_move();
    }

    let static_eval;
    if in_check {
        static_eval = NONE;
    } else {
        static_eval = td.accumulators.evaluate(board);
    }
    td.stack[td.ply].static_eval = static_eval;

    td.stack[td.ply + 1].killer_move = None;
    td.stack[td.ply + 2].cutoffs = 0;

    let mut tacticals_tried = ArrayVec::<_, { MAX_MOVES }>::new();
    let mut quiets_tried = ArrayVec::<_, { MAX_MOVES }>::new();

    let mut moves_searched = 0;
    let mut best_score = -INFINITY;
    let mut best_move = Move::NULL;
    let original_alpha = alpha;
    let mut picker = MovePicker::new(tt_move, td, -197, false);
    while let Some(MoveListEntry { m, .. }) = picker.next(board, td) {
        if !board.is_legal(m) {
            continue;
        };
        tt.prefetch(board.hash_after(Some(m)));
        let copy = board.make_move(m);

        td.accumulators.push(m, board.piece_at(m.from()), board.piece_at(m.to()));
        td.hash_history.push(copy.zobrist_hash);
        td.stack[td.ply].played_move = Some(m);
        td.stack[td.ply].moved_piece = board.piece_at(m.from());
        td.ply += 1;

        let new_depth = depth - 1;

        let mut score = -INFINITY;

        let base_reduction = td.lmr.base_reduction(depth, moves_searched);

        if depth > 2 && moves_searched > 1 + i32::from(is_root) && m.is_quiet(board) {
            let d = (new_depth - base_reduction).clamp(1, new_depth);

            score = -negamax::<false>(td, tt, &copy, -alpha - 1, -alpha, d);
        } else if !PV || moves_searched > 0 {
            score = -negamax::<false>(td, tt, &copy, -alpha - 1, -alpha, new_depth);
        }

        if PV && (moves_searched == 0 || score > alpha) {
            score = -negamax::<true>(td, tt, &copy, -beta, -alpha, new_depth);
        }

        td.ply -= 1;
        td.hash_history.pop();
        td.accumulators.pop();
        moves_searched += 1;
        if m.is_tactical(board) {
            tacticals_tried.push(m);
        } else {
            quiets_tried.push(m);
        }

        if td.halt() {
            return 0;
        }

        best_score = best_score.max(score);

        if score <= alpha {
            continue;
        }

        best_move = Some(m);
        alpha = score;
        if PV {
            td.pv.append(best_move, td.ply);
        }

        if score < beta {
            continue;
        }

        td.stack[td.ply].cutoffs += 1;

        if m.is_quiet(board) {
            td.stack[td.ply].killer_move = Some(m);
        }
        td.update_histories(m, &quiets_tried, &tacticals_tried, board, depth);

        break;
    }

    if moves_searched == 0 {
        best_score = if in_check { mated_in(td.ply) } else { STALEMATE }
    }

    let flag = if best_score >= beta {
        EntryFlag::BetaCutOff
    } else if best_score > original_alpha {
        EntryFlag::Exact
    } else {
        EntryFlag::AlphaUnchanged
    };
    tt.store(board.zobrist_hash, best_move, depth, flag, best_score, td.ply, PV, static_eval);

    best_score
}

fn qsearch<const PV: bool>(
    td: &mut ThreadData,
    tt: &TranspositionTable,
    board: &Board,
    mut alpha: i32,
    beta: i32,
) -> i32 {
    let in_check = board.in_check();

    td.sel_depth = td.sel_depth.max(td.ply);
    td.pv.clear_depth(td.ply);

    if td.halt() {
        return 0;
    }

    if td.main_thread() && td.hard_stop() {
        td.set_halt(true);
        return 0;
    }

    if td.ply >= MAX_PLY {
        return td.accumulators.evaluate(board);
    }

    if board.is_draw() || td.is_repetition(board, td.ply) {
        return STALEMATE;
    }

    td.nodes.increment();

    let mut tt_move = Move::NULL;
    if let Some(entry) = tt.get(board.zobrist_hash, td.ply) {
        tt_move = entry.best_move();
    }

    let static_eval = td.accumulators.evaluate(board);
    if static_eval >= beta {
        return static_eval;
    }
    alpha = alpha.max(static_eval);

    let mut best_score = if in_check { -CHECKMATE } else { static_eval };
    let mut picker = MovePicker::new(tt_move, td, -197, true);
    let mut best_move = Move::NULL;
    let mut moves_searched = 0;

    while let Some(MoveListEntry { m, .. }) = picker.next(board, td) {
        if !board.is_legal(m) {
            continue;
        }
        tt.prefetch(board.hash_after(Some(m)));
        let copy = board.make_move(m);

        td.accumulators.push(m, board.piece_at(m.from()), board.piece_at(m.to()));
        td.hash_history.push(copy.zobrist_hash);
        td.stack[td.ply].played_move = Some(m);
        td.stack[td.ply].moved_piece = board.piece_at(m.from());
        td.ply += 1;

        let score = -qsearch::<PV>(td, tt, &copy, -beta, -alpha);

        td.ply -= 1;
        td.accumulators.pop();
        td.hash_history.pop();
        moves_searched += 1;

        if td.halt() {
            return 0;
        }

        best_score = best_score.max(score);

        if score <= alpha {
            continue;
        }

        best_move = Some(m);
        alpha = score;
        if PV {
            td.pv.append(best_move, td.ply);
        }

        if score < beta {
            continue;
        }

        break;
    }

    let flag = if best_score >= beta { EntryFlag::BetaCutOff } else { EntryFlag::AlphaUnchanged };
    tt.store(board.zobrist_hash, best_move, 0, flag, best_score, td.ply, PV, static_eval);

    best_score
}
