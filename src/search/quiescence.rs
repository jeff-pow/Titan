use crate::board::board::Board;
use crate::engine::transposition::{EntryFlag, TranspositionTable};
use crate::moves::movegenerator::MGT;
use crate::moves::movelist::{MoveList, MoveListEntry};
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
    if board.is_draw() || td.is_repetition(board) {
        return STALEMATE;
    }

    if IS_PV {
        td.sel_depth = td.sel_depth.max(td.ply);
    }

    // Halt search if we are going to overflow the search stack
    if td.ply >= MAX_SEARCH_DEPTH {
        return board.evaluate();
    }

    // Probe transposition table for best move and eval
    let mut table_move = Move::NULL;
    let entry = tt.get(board.zobrist_hash, td.ply);
    if let Some(e) = entry {
        if !IS_PV
            && e.flag() != EntryFlag::None
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
    let stand_pat = if let Some(entry) = entry { entry.static_eval() } else { board.evaluate() };
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

    let mut moves = MoveList::default();
    if in_check {
        board.generate_moves(MGT::All, &mut moves)
    } else {
        board.generate_moves(MGT::CapturesOnly, &mut moves)
    };
    moves.score_moves(board, table_move, td.stack[td.ply].killer_move, td);

    let mut best_score = if in_check { -INFINITY } else { stand_pat };

    let mut best_move = Move::NULL;
    let mut moves_searched = 0;

    for MoveListEntry { m, .. } in moves {
        let mut node_pv = PV::default();
        let mut new_b = *board;

        // Static exchange pruning - If we fail to immediately recapture a depth dependent
        // threshold, don't bother searching the move. Ensure we either have a legal evasion or
        // aren't in check before pruning
        if (!in_check || moves_searched > 1) && !board.see(m, 1) {
            continue;
        }

        if !new_b.make_move::<true>(m) {
            continue;
        }
        tt.prefetch(new_b.zobrist_hash);
        td.hash_history.push(new_b.zobrist_hash);
        td.stack[td.ply].played_move = m;
        td.nodes_searched += 1;
        moves_searched += 1;
        td.ply += 1;

        // TODO: Implement delta pruning

        let eval = -quiescence::<IS_PV>(-beta, -alpha, &mut node_pv, td, tt, &new_b);

        td.ply -= 1;
        td.hash_history.pop();

        if eval > best_score {
            best_score = eval;

            if eval > alpha {
                best_move = m;
                alpha = eval;
                if IS_PV {
                    pv.update(best_move, node_pv);
                }
            }

            if alpha >= beta {
                tt.prefetch(board.zobrist_hash);
                break;
            }
        }
    }

    // TODO: Try only fail low or fail high flags
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
