use crate::board::board::Board;
use crate::moves::attack_boards::pawn_attacks;
use crate::moves::{movegenerator::generate_psuedolegal_captures, moves::Move};
use crate::search::pvs::STALEMATE;
use crate::types::bitboard::Bitboard;
use crate::types::pieces::PieceName;
use crate::types::square::Square;

use super::pvs::{score_move_list, sort_next_move};
use super::{eval::eval, pvs::MAX_SEARCH_DEPTH, SearchInfo};

pub fn quiescence(
    ply: i8,
    mut alpha: i32,
    beta: i32,
    best_moves: &mut Vec<Move>,
    search_info: &mut SearchInfo,
    board: &Board,
) -> i32 {
    //
    // Draw if a position has occurred three times
    if board.is_draw() {
        return STALEMATE;
    }

    search_info.sel_depth = search_info.sel_depth.max(ply);
    search_info.search_stats.nodes_searched += 1;
    let eval = eval(board);
    if ply >= MAX_SEARCH_DEPTH {
        return eval;
    }

    // Give the engine the chance to stop capturing here if it results in a better end result than continuing the chain of capturing
    if eval >= beta {
        return beta;
    }
    if eval > alpha {
        alpha = eval;
    }

    let mut moves = generate_psuedolegal_captures(board);
    score_move_list(ply, board, &mut moves, Move::NULL, search_info);

    for i in 0..moves.len {
        let mut best_node_moves = Vec::new();
        let mut new_b = board.to_owned();
        sort_next_move(&mut moves, i);
        let m = moves.get_move(i);
        new_b.make_move(m);
        // Just generate psuedolegal moves to save computation time on checks for moves that will be
        // pruned
        if new_b.side_in_check(board.to_move) {
            continue;
        }

        if (eval + board.piece_on_square(m.dest_square()).unwrap().value() + 200 < alpha)
            && m.promotion().is_none()
            // && (net_piece_value(board, board.to_move.opp())
            && (board.material_val[board.to_move.opp() as usize]
                - board.piece_on_square(m.dest_square()).unwrap().value()
                > 1300)
        {
            continue;
        }

        if is_bad_capture(board, m) && m.promotion().is_none() {
            continue;
        }

        let eval = -quiescence(
            ply + 1,
            -beta,
            -alpha,
            &mut best_node_moves,
            search_info,
            &new_b,
        );

        if eval >= beta {
            return beta;
        }

        if eval > alpha {
            alpha = eval;

            best_moves.clear();
            best_moves.push(*m);
            best_moves.append(&mut best_node_moves);
        }
    }

    alpha
}

fn is_bad_capture(board: &Board, m: &Move) -> bool {
    let moving_piece = board.piece_on_square(m.origin_square()).unwrap();
    let capture = board.piece_on_square(m.dest_square()).unwrap();
    if moving_piece == PieceName::Pawn {
        return false;
    }

    if capture.value() >= moving_piece.value() - 50 {
        return false;
    }

    if is_pawn_recapture(board, m.dest_square()) && capture.value() + 200 - moving_piece.value() < 0
    {
        return true;
    }

    false
}

fn is_pawn_recapture(board: &Board, sq: Square) -> bool {
    let attacker = board.to_move.opp();
    let pawn_attacks = pawn_attacks(sq, board.to_move);
    if pawn_attacks & board.bitboards[attacker as usize][PieceName::Pawn as usize]
        != Bitboard::EMPTY
    {
        return true;
    }
    false
}
