use crate::board::board::Board;
use crate::engine::transposition::EntryFlag;
use crate::moves::movegenerator::MGT;
use crate::moves::movelist::MoveListEntry;
use crate::moves::moves::Move;
use crate::search::search::STALEMATE;

use super::search::MAX_SEARCH_DEPTH;
use super::search::{CHECKMATE, INFINITY};
use super::{store_pv, thread::ThreadData};

pub fn quiescence(ply: i32, mut alpha: i32, beta: i32, pvs: &mut Vec<Move>, td: &mut ThreadData, board: &Board) -> i32 {
    if board.is_draw() {
        return STALEMATE;
    }

    td.sel_depth = td.sel_depth.max(ply);

    if ply >= MAX_SEARCH_DEPTH {
        return board.evaluate();
    }

    // TODO: Return tt score
    let entry = td.transpos_table.get(board.zobrist_hash, ply);
    let table_move = entry.map_or(Move::NULL, |e| e.best_move(board));

    // Give the engine the chance to stop capturing here if it results in a better end result than continuing the chain of capturing

    let stand_pat = board.evaluate();
    if stand_pat >= beta {
        return stand_pat;
    }
    let original_alpha = alpha;
    alpha = alpha.max(stand_pat);

    let in_check = board.in_check;
    let mut moves = if in_check {
        board.generate_moves(MGT::All)
    } else {
        board.generate_moves(MGT::CapturesOnly)
    };
    moves.score_moves(board, table_move, td.stack[ply].killers, td, ply);

    let mut best_score = if in_check { -INFINITY } else { board.evaluate() };

    let mut best_move = Move::NULL;
    let mut moves_searched = 0;

    for MoveListEntry { m, .. } in moves {
        let mut node_pvs = Vec::new();
        let mut new_b = *board;

        // We want to find at least one evasion so we know we aren't in checkmate, so don't prune
        // moves before then
        if (!in_check || moves_searched > 1) && !board.see(m, 1) {
            continue;
        }

        if !new_b.make_move::<true>(m) {
            continue;
        }
        td.stack[ply].played_move = m;
        td.nodes_searched += 1;
        moves_searched += 1;

        // TODO: Implement delta pruning

        let eval = -quiescence(ply + 1, -beta, -alpha, &mut node_pvs, td, &new_b);

        if eval > best_score {
            best_score = eval;
            // TODO: Only when best_move raises alpha
            best_move = m;
            if eval > alpha {
                alpha = eval;
                store_pv(pvs, &mut node_pvs, m);
            }
            if alpha >= beta {
                break;
            }
        }
    }

    // TODO: Only fail low or fail high flags
    let entry_flag = if best_score >= beta {
        EntryFlag::BetaCutOff
    } else if best_score > original_alpha {
        EntryFlag::Exact
    } else {
        EntryFlag::AlphaUnchanged
    };

    td.transpos_table
        .store(board.zobrist_hash, best_move, 0, entry_flag, best_score, ply, false, stand_pat);

    if in_check && moves_searched == 0 {
        return -CHECKMATE + ply;
    }

    best_score
}
