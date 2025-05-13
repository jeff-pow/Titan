use std::time::Instant;

use crate::board::Board;
use crate::chess_move::Move;
use crate::movelist::MAX_MOVES;
use crate::movepicker::MovePicker;
use crate::search::SearchStack;
use crate::thread::ThreadData;
use crate::transposition::{EntryFlag, TranspositionTable};
use crate::types::pieces::Piece;
use crate::utils::boxed;
use arrayvec::ArrayVec;

pub const MAX_PLY: usize = 128;

pub struct Score;
impl Score {
    pub const STALEMATE: i32 = 0;
    pub const CHECKMATE: i32 = 32000;
    pub const INFINITY: i32 = 32001;
    pub const NONE: i32 = 32002;

    pub const MATE_IN_MAX_PLY: i32 = Self::CHECKMATE - MAX_PLY as i32;
    pub const MATED_IN_MAX_PLY: i32 = -Self::CHECKMATE + MAX_PLY as i32;

    pub const fn mated_in(ply: usize) -> i32 {
        -Self::CHECKMATE + ply as i32
    }

    pub const fn mate_in(ply: usize) -> i32 {
        Self::CHECKMATE - ply as i32
    }

    pub const fn mate_found(score: i32) -> bool {
        score.abs() >= Self::MATE_IN_MAX_PLY
    }

    pub const fn is_win(score: i32) -> bool {
        score >= Self::MATE_IN_MAX_PLY
    }

    pub const fn is_loss(score: i32) -> bool {
        score <= Self::MATED_IN_MAX_PLY
    }

    pub fn clamp_score(score: i32) -> i32 {
        score.clamp(Self::MATED_IN_MAX_PLY + 1, Self::MATE_IN_MAX_PLY - 1)
    }
}

pub fn start_search(td: &mut ThreadData, print_uci: bool, board: Board, tt: &TranspositionTable) {
    td.search_start = Instant::now();
    td.nodes_table = boxed();
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
    let mut prev_score = Score::NONE;
    let mut depth = 1;

    loop {
        td.sel_depth = 0;
        td.iter_depth = depth;

        assert_eq!(0, td.ply);
        assert_eq!(0, td.accumulators.top);

        prev_score = aspiration_windows(td, board, tt, prev_score, depth);

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

pub fn aspiration_windows(
    td: &mut ThreadData,
    board: &Board,
    tt: &TranspositionTable,
    prev_score: i32,
    depth: i32,
) -> i32 {
    let mut alpha = -Score::INFINITY;
    let mut beta = Score::INFINITY;
    let mut delta = 10;

    if depth >= 4 {
        alpha = (prev_score - delta).max(-Score::CHECKMATE);
        beta = (prev_score + delta).min(Score::CHECKMATE);
    }

    loop {
        assert_eq!(0, td.ply);
        let score = negamax::<true>(td, tt, board, alpha, beta, depth, false);

        if td.halt() {
            return score;
        }

        if score <= alpha {
            beta = i32::midpoint(alpha, beta);
            alpha = (score - delta).max(-Score::INFINITY);
        } else if score >= beta {
            beta = (score + delta).min(Score::INFINITY);
        } else {
            return score;
        }

        delta += 4 * delta / 9;
    }
}

fn negamax<const PV: bool>(
    td: &mut ThreadData,
    tt: &TranspositionTable,
    board: &Board,
    mut alpha: i32,
    beta: i32,
    depth: i32,
    cut_node: bool,
) -> i32 {
    let is_root = td.ply == 0;
    let in_check = board.in_check();

    let excluded_move = td.stack[td.ply].excluded;
    let singular_search = excluded_move.is_some();

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
        if board.is_draw() || td.is_repetition(board) {
            return Score::STALEMATE;
        }

        // Mate Distance Pruning - Determines if there is a faster path to checkmate
        // than evaluating the current node, and if there is, it returns early
        let alpha = alpha.max(Score::mated_in(td.ply));
        let beta = beta.min(Score::mate_in(td.ply));
        if alpha >= beta {
            return alpha;
        }
    }

    if depth <= 0 {
        return qsearch::<PV>(td, tt, board, alpha, beta);
    }

    td.nodes.increment();

    let mut tt_move = Move::NULL;
    let entry = tt.get(board.zobrist_hash, td.ply);
    if let Some(entry) = entry {
        tt_move = entry.best_move();

        if !PV
            && !singular_search
            && depth <= entry.depth()
            && match entry.flag() {
                EntryFlag::None => false,
                EntryFlag::AlphaUnchanged => entry.search_score() <= alpha,
                EntryFlag::BetaCutOff => entry.search_score() >= beta,
                EntryFlag::Exact => true,
            }
        {
            return entry.search_score();
        }
    }

    let correction = td.pawn_corr_hist.get(board.stm, board.pawn_hash());

    let raw_eval;
    let static_eval;
    let eval;
    if in_check {
        raw_eval = Score::NONE;
        static_eval = Score::NONE;
        eval = Score::NONE;
    } else if singular_search {
        raw_eval = td.stack[td.ply].static_eval;
        static_eval = raw_eval;
        eval = static_eval;
    } else {
        raw_eval = td.accumulators.evaluate(board);
        static_eval = raw_eval + correction;
        eval = static_eval;
    }
    td.stack[td.ply].static_eval = static_eval;

    // TODO: Add a conditional check to make sure neither of the previous two ply's moves were null moves
    let improving = !in_check && td.ply > 1 && static_eval > td.stack[td.ply - 2].static_eval;

    if !PV
        && !in_check
        && !singular_search
        && depth < 9
        && eval >= beta
        && static_eval - 93 * depth + i32::from(improving) * 30 * depth >= beta
    {
        return Score::clamp_score((static_eval + beta) / 2);
    }

    if !in_check
        && cut_node
        && !singular_search
        && depth >= 2
        && !Score::is_loss(beta)
        && td.stack[td.ply - 1].played_move != Move::NULL
        && board.has_non_pawns(board.stm)
        && eval >= beta
    {
        tt.prefetch(board.hash_after(Move::NULL));

        let r = 4 + depth / 4 + ((static_eval - beta) / 173).min(4);
        let copy = board.make_null_move();

        td.stack[td.ply].played_move = Move::NULL;
        td.stack[td.ply].moved_piece = Piece::None;
        td.ply += 1;
        td.hash_history.push(copy.zobrist_hash);

        let score = -negamax::<false>(td, tt, &copy, -beta, -beta + 1, depth - r, false);

        td.ply -= 1;
        td.hash_history.pop();

        if td.halt() {
            return 0;
        }

        if score >= beta {
            if Score::mate_found(score) {
                return beta;
            }
            return score;
        }
    }

    td.stack[td.ply + 1].killer_move = None;
    td.stack[td.ply + 2].cutoffs = 0;

    let mut tacticals_tried = ArrayVec::<_, { MAX_MOVES }>::new();
    let mut quiets_tried = ArrayVec::<_, { MAX_MOVES }>::new();

    let mut moves_searched = 0;
    let mut best_score = -Score::INFINITY;
    let mut best_move = Move::NULL;
    let original_alpha = alpha;
    let mut picker = MovePicker::new(tt_move, td, -197, true);
    while let Some(m) = picker.next(board, td) {
        if !board.is_legal(m) || Some(m) == excluded_move {
            continue;
        }

        if !is_root && !Score::is_loss(best_score) {
            let margin = if m.is_tactical(board) { -93 } else { -41 } * depth;
            if depth < 12 && !board.see(m, margin) {
                continue;
            }
        }

        tt.prefetch(board.hash_after(Some(m)));

        let extension = if !is_root
            && !singular_search
            && td.ply < 2 * td.iter_depth as usize
            && Some(m) == tt_move
            && depth >= 8
            && entry.is_some_and(|e| {
                e.depth() >= depth - 3
                    && matches!(e.flag(), EntryFlag::Exact | EntryFlag::BetaCutOff)
                    && !Score::mate_found(e.search_score())
            }) {
            let entry = entry.unwrap();

            let ext_beta = entry.search_score() - 21 * depth / 16;
            let ext_depth = (depth - 1) / 2;

            td.stack[td.ply].excluded = Some(m);
            let score = negamax::<false>(td, tt, board, ext_beta - 1, ext_beta, ext_depth, cut_node);
            td.stack[td.ply].excluded = None;

            if score < ext_beta {
                1 + i32::from(!PV && score < ext_beta - 18)
            } else if entry.search_score() >= beta {
                -2
            } else {
                0
            }
        } else {
            0
        };

        let copy = board.make_move(m);

        td.accumulators.push(m, board.piece_at(m.from()), board.piece_at(m.to()));
        td.hash_history.push(copy.zobrist_hash);
        td.stack[td.ply].played_move = Some(m);
        td.stack[td.ply].moved_piece = board.piece_at(m.from());
        td.ply += 1;

        let new_depth = depth + extension - 1;

        let mut score = -Score::INFINITY;

        let base_reduction = td.lmr.base_reduction(depth, moves_searched);

        if depth > 2 && moves_searched > 1 + i32::from(is_root) && m.is_quiet(board) {
            let d = (new_depth - base_reduction).clamp(1, new_depth);

            score = -negamax::<false>(td, tt, &copy, -alpha - 1, -alpha, d, true);
        } else if !PV || moves_searched > 0 {
            score = -negamax::<false>(td, tt, &copy, -alpha - 1, -alpha, new_depth, !cut_node);
        }

        if PV && (moves_searched == 0 || score > alpha) {
            score = -negamax::<true>(td, tt, &copy, -beta, -alpha, new_depth, false);
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
        if singular_search {
            return alpha;
        }

        best_score = if in_check { Score::mated_in(td.ply) } else { Score::STALEMATE }
    }

    let flag = if best_score >= beta {
        EntryFlag::BetaCutOff
    } else if best_score > original_alpha {
        EntryFlag::Exact
    } else {
        EntryFlag::AlphaUnchanged
    };

    if !singular_search {
        tt.store(board.zobrist_hash, best_move, depth, flag, best_score, td.ply, PV, raw_eval);
    }

    if !(in_check
        || best_move.is_some_and(|m| m.is_tactical(board))
        || Score::mate_found(best_score)
        || (flag == EntryFlag::AlphaUnchanged && best_score >= static_eval)
        || (flag == EntryFlag::BetaCutOff && best_score <= static_eval))
    {
        td.pawn_corr_hist.update(board.stm, board.pawn_hash(), best_score - static_eval, depth);
    }

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

    if board.is_draw() || td.is_repetition(board) {
        return Score::STALEMATE;
    }

    td.nodes.increment();

    let mut tt_move = Move::NULL;
    if let Some(entry) = tt.get(board.zobrist_hash, td.ply) {
        tt_move = entry.best_move();

        if match entry.flag() {
            EntryFlag::None => false,
            EntryFlag::AlphaUnchanged => entry.search_score() <= alpha,
            EntryFlag::BetaCutOff => entry.search_score() >= beta,
            EntryFlag::Exact => true,
        } {
            return entry.search_score();
        }
    }

    let mut best_score = -Score::INFINITY;
    let mut futility = Score::NONE;
    let mut raw_eval = Score::NONE;

    if !in_check {
        raw_eval = td.accumulators.evaluate(board);
        let static_eval = raw_eval + td.pawn_corr_hist.get(board.stm, board.pawn_hash());
        best_score = static_eval;

        if best_score >= beta {
            return best_score;
        }
        alpha = alpha.max(best_score);
        futility = static_eval + 175;
    }

    let mut picker = MovePicker::new(tt_move, td, -197, in_check);
    let mut best_move = Move::NULL;
    let mut moves_searched = 0;

    while let Some(m) = picker.next(board, td) {
        if !board.is_legal(m) {
            continue;
        }
        // If we were in check, we know there's at least one legal move so we can skip the remaining quiets
        picker.skip_quiets();

        if !Score::is_loss(best_score) && m.is_tactical(board) && !in_check && futility <= alpha && !board.see(m, 1) {
            best_score = best_score.max(futility);
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

    if moves_searched == 0 && in_check {
        return Score::mated_in(td.ply);
    }

    let flag = if best_score >= beta { EntryFlag::BetaCutOff } else { EntryFlag::AlphaUnchanged };
    tt.store(board.zobrist_hash, best_move, 0, flag, best_score, td.ply, PV, raw_eval);

    best_score
}
