use std::sync::atomic::Ordering;

use crate::board::board::Board;
use crate::engine::transposition::{EntryFlag, TranspositionTable};
use crate::moves::movelist::MoveListEntry;
use crate::moves::movepicker::MovePicker;
use crate::moves::moves::Move;
use crate::search::search::STALEMATE;

use super::search::MAX_SEARCH_DEPTH;
use super::search::{CHECKMATE, INFINITY};
use super::thread::ThreadData;
use super::PV;

pub(super) fn quiescence<const IS_PV: bool>(
    mut alpha: i32,
    beta: i32,
    pv: &mut PV,
    td: &mut ThreadData,
    tt: &TranspositionTable,
    board: &Board,
) -> i32 {
    // Stop if we have reached hard time limit or decided else where it is time to stop
    if td.halt.load(Ordering::Relaxed) {
        td.halt.store(true, Ordering::Relaxed);
        return 0;
    }

    if td.nodes.check_time() && td.thread_idx == 0 && td.hard_stop() {
        td.halt.store(true, Ordering::Relaxed);
        return 0;
    }

    if board.is_draw() || td.is_repetition(board) {
        return STALEMATE;
    }

    td.sel_depth = td.sel_depth.max(td.ply);

    // Halt search if we are going to overflow the search stack
    if td.ply >= MAX_SEARCH_DEPTH {
        return board.evaluate(td.accumulators.top());
    }

    // Probe transposition table for best move and eval
    let mut table_move = Move::NULL;
    let entry = tt.get(board.zobrist_hash, td.ply);
    if let Some(e) = entry {
        if !IS_PV
            && match e.flag() {
                EntryFlag::None => false,
                EntryFlag::AlphaUnchanged => e.search_score() <= alpha,
                EntryFlag::BetaCutOff => e.search_score() >= beta,
                EntryFlag::Exact => true,
            }
        {
            return e.search_score();
        }
        table_move = e.best_move(board);
    }

    // Give the engine the chance to stop capturing here if it results in a better end result than continuing the chain of capturing
    let stand_pat = if let Some(entry) = entry {
        entry.static_eval()
    } else {
        board.evaluate(td.accumulators.top())
    };
    td.stack[td.ply].static_eval = stand_pat;
    // Store eval in tt if it wasn't previously found in tt
    if entry.is_none() && !board.in_check {
        tt.store(
            board.zobrist_hash,
            Move::NULL,
            0,
            EntryFlag::None,
            INFINITY,
            td.ply,
            IS_PV,
            stand_pat,
        );
    }
    if stand_pat >= beta {
        return stand_pat;
    }
    let original_alpha = alpha;
    alpha = alpha.max(stand_pat);

    let in_check = board.in_check;
    // Try to find an evasion if we are in check, otherwise just generate captures
    let mut picker = MovePicker::new(table_move, td, 1, !in_check);

    let mut best_score = if in_check { -CHECKMATE } else { stand_pat };

    let mut best_move = Move::NULL;
    let mut moves_searched = 0;

    while let Some(MoveListEntry { m, .. }) = picker.next(board, td) {
        let mut node_pv = PV::default();
        let mut new_b = *board;

        // Static exchange pruning - If we fail to immediately recapture a depth dependent
        // threshold, don't bother searching the move. Ensure we either have a legal evasion or
        // aren't in check before pruning
        // if (!in_check || moves_searched > 1) && !board.see(m, 1) {
        //     continue;
        // }

        if !new_b.make_move::<true>(m) {
            continue;
        }
        tt.prefetch(new_b.zobrist_hash);
        td.accumulators.increment();
        td.accumulators.top().lazy_update(&mut new_b.delta);
        td.hash_history.push(new_b.zobrist_hash);
        td.stack[td.ply].played_move = m;
        td.nodes.increment();
        moves_searched += 1;
        td.ply += 1;

        let eval = -quiescence::<IS_PV>(-beta, -alpha, &mut node_pv, td, tt, &new_b);

        td.ply -= 1;
        td.hash_history.pop();
        td.accumulators.pop();

        if eval > best_score {
            best_score = eval;

            if eval > alpha {
                best_move = m;
                alpha = eval;
                pv.update(best_move, node_pv);
            }

            if alpha >= beta {
                break;
            }
        }
    }

    let entry_flag = if best_score >= beta {
        EntryFlag::BetaCutOff
    } else if best_score > original_alpha {
        EntryFlag::Exact
    } else {
        EntryFlag::AlphaUnchanged
    };

    tt.store(board.zobrist_hash, best_move, 0, entry_flag, best_score, td.ply, IS_PV, stand_pat);

    if in_check && moves_searched == 0 {
        return -CHECKMATE + td.ply;
    }

    best_score
}
