use crate::board::board::Board;
use crate::moves::movegenerator::generate_moves;
use crate::moves::{movegenerator::generate_psuedolegal_captures, moves::Move};
use crate::search::pvs::STALEMATE;
use crate::types::bitboard::Bitboard;
use crate::types::pieces::{value, PieceName};
use crate::types::square::Square;

use super::pvs::{CHECKMATE, INFINITY};
use super::{eval::eval, pvs::MAX_SEARCH_DEPTH, SearchInfo};

pub fn quiescence(
    ply: i8,
    mut alpha: i32,
    beta: i32,
    pvs: &mut Vec<Move>,
    search_info: &mut SearchInfo,
    board: &Board,
) -> i32 {
    if board.is_draw() {
        return STALEMATE;
    }

    search_info.sel_depth = search_info.sel_depth.max(ply);
    search_info.search_stats.nodes_searched += 1;
    if ply >= MAX_SEARCH_DEPTH {
        return eval(board);
    }

    // Give the engine the chance to stop capturing here if it results in a better end result than continuing the chain of capturing
    // TODO: Experiment with removing these
    let eval = eval(board);
    if eval >= beta {
        return eval;
    }
    alpha = alpha.max(eval);

    let in_check = board.side_in_check(board.to_move);
    let mut moves = if in_check {
        generate_moves(board)
    } else {
        generate_psuedolegal_captures(board)
    };
    let mut legal_moves_searched = 0;
    moves.score_move_list(ply, board, Move::NULL, search_info);
    let mut best_score = -INFINITY;

    for i in 0..moves.len {
        let mut node_pvs = Vec::new();
        let mut new_b = board.to_owned();
        moves.sort_next_move(i);
        let m = moves.get_move(i);
        new_b.make_move(m);
        if new_b.side_in_check(board.to_move) {
            continue;
        }
        legal_moves_searched += 1;

        // TODO: Implement delta pruning here

        if is_bad_capture(board, m) && m.promotion().is_none() {
            continue;
        }

        let eval = -quiescence(ply + 1, -beta, -alpha, &mut node_pvs, search_info, &new_b);

        if eval >= beta {
            return eval;
        }

        if eval > alpha {
            alpha = eval;
            pvs.clear();
            pvs.push(*m);
            pvs.append(&mut node_pvs);
        }
        if eval > best_score {
            best_score = eval;

            if eval > alpha {
                alpha = eval;
                pvs.clear();
                pvs.push(*m);
                pvs.append(&mut node_pvs);
            }
            if alpha >= beta {
                return alpha;
            }
        }
    }
    if legal_moves_searched == 0 {
        if in_check {
            return -CHECKMATE + ply as i32;
        }
        return STALEMATE;
    }

    alpha
}

fn is_bad_capture(board: &Board, m: &Move) -> bool {
    let moving_piece = board.piece_at(m.origin_square()).unwrap();
    let capture = board.piece_at(m.dest_square());
    if moving_piece == PieceName::Pawn {
        return false;
    }

    if value(capture) >= moving_piece.value() - 50 {
        return false;
    }

    if is_pawn_recapture(board, m.dest_square()) && value(capture) + 200 - moving_piece.value() < 0 {
        return true;
    }

    false
}

fn is_pawn_recapture(board: &Board, sq: Square) -> bool {
    let attacker = board.to_move.opp();
    let pawn_attacks = board.mg.pawn_attacks(sq, board.to_move);
    if pawn_attacks & board.bitboards[attacker as usize][PieceName::Pawn as usize] != Bitboard::EMPTY {
        return true;
    }
    false
}
