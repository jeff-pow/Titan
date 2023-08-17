use std::cmp::{max, min};
use std::time::{Duration, Instant};

use crate::board::lib::Board;
use crate::engine::transposition::{EntryFlag, TableEntry};
use crate::moves::lib::Move;
use crate::moves::lib::Promotion;
use crate::moves::movegenerator::generate_psuedolegal_moves;
use crate::moves::movelist::MoveList;
use crate::types::pieces::{piece_value, PieceName};

use super::eval::eval;
use super::quiescence::quiescence;
use super::{SearchInfo, SearchType};

pub const IN_CHECKMATE: i32 = 100000;
pub const STALEMATE: i32 = 0;
pub const NEAR_CHECKMATE: i32 = IN_CHECKMATE - 1000;
pub const INFINITY: i32 = 9999999;
pub const MAX_SEARCH_DEPTH: i8 = 100;

pub fn search(search_info: &mut SearchInfo) -> Move {
    let max_depth;
    let mut best_move = Move::NULL;
    let mut best_moves = Vec::new();

    let mut recommended_time = Duration::ZERO;
    match search_info.search_type {
        SearchType::Time => {
            recommended_time = search_info
                .game_time
                .recommended_time(search_info.board.to_move);
            max_depth = MAX_SEARCH_DEPTH;
        }
        SearchType::Depth => {
            max_depth = search_info.depth;
        }
    }

    let alpha_start = -INFINITY;
    let beta_start = INFINITY;

    search_info.search_stats.start = Instant::now();
    let mut current_depth = 1;
    let mut eval;

    while current_depth <= max_depth {
        search_info.depth = current_depth;
        search_info.sel_depth = current_depth;

        eval = alpha_beta(
            current_depth,
            0,
            alpha_start,
            beta_start,
            &mut best_moves,
            search_info,
            &search_info.board.to_owned(),
        );

        if !best_moves.is_empty() {
            best_move = best_moves[0];
        }
        print!(
            "info time {} seldepth {} depth {} nodes {} nps {} score cp {} pv ",
            search_info.search_stats.start.elapsed().as_millis(),
            search_info.sel_depth,
            current_depth,
            search_info.search_stats.nodes_searched,
            search_info.search_stats.nodes_searched as f64
                / search_info.search_stats.start.elapsed().as_secs_f64(),
            eval
        );
        for m in best_moves.iter() {
            print!("{} ", m.to_lan());
        }
        println!();
        if search_info.search_type == SearchType::Time
            && search_info
                .game_time
                .reached_termination(search_info.search_stats.start, recommended_time)
        {
            break;
        }
        current_depth += 1;
    }

    assert_ne!(best_move, Move::NULL);

    best_move
}

fn alpha_beta(
    mut depth: i8,
    ply: i8,
    mut alpha: i32,
    mut beta: i32,
    best_moves: &mut Vec<Move>,
    search_info: &mut SearchInfo,
    board: &Board,
) -> i32 {
    let is_root = ply == 0;
    let mut principal_var_search = false;
    search_info.sel_depth = search_info.sel_depth.max(ply);
    // Needed since the function can calculate extensions in cases where it finds itself in check
    if ply >= MAX_SEARCH_DEPTH {
        return eval(board);
    }

    if board.is_draw() {
        return STALEMATE;
    }

    // Code determines if there is a faster path to checkmate than evaluating the current node, and
    // if there is, it returns early
    alpha = max(alpha, -IN_CHECKMATE + ply as i32);
    beta = min(beta, IN_CHECKMATE - ply as i32);
    if alpha >= beta {
        return alpha;
    }

    let is_check = board.side_in_check(board.to_move);

    if is_check {
        depth += 1;
    }

    if depth <= 0 {
        return quiescence(ply, alpha, beta, best_moves, search_info, board);
    }

    search_info.search_stats.nodes_searched += 1;

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

    let mut moves = generate_psuedolegal_moves(board);
    let mut legal_moves = 0;
    score_move_list(board, &mut moves, table_move);

    let mut best_eval = -INFINITY;
    let mut entry_flag = EntryFlag::AlphaCutOff;
    let mut best_move = Move::NULL;

    for i in 0..moves.len {
        let mut new_b = board.to_owned();
        sort_next_move(&mut moves, i);
        let m = moves.get_move(i);
        new_b.make_move(m);
        // Just generate psuedolegal moves to save computation time on legality for moves that will be
        // pruned
        if new_b.side_in_check(board.to_move) {
            continue;
        }
        legal_moves += 1;

        let mut node_best_moves = Vec::new();

        let mut eval;
        // Draw if a position has occurred three times

        if principal_var_search {
            eval = -alpha_beta(
                depth - 1,
                ply + 1,
                -alpha - 1,
                -alpha,
                &mut node_best_moves,
                search_info,
                &new_b,
            );

            // Redo search with full window if principal variation search failed
            if (eval > alpha) && (eval < beta) {
                eval = -alpha_beta(
                    depth - 1,
                    ply + 1,
                    -beta,
                    -alpha,
                    &mut node_best_moves,
                    search_info,
                    &new_b,
                );
            }
        } else {
            eval = -alpha_beta(
                depth - 1,
                ply + 1,
                -beta,
                -alpha,
                &mut node_best_moves,
                search_info,
                &new_b,
            );
        }

        if eval > best_eval {
            best_eval = eval;
            best_move = *m;
        }

        if eval >= beta {
            search_info.transpos_table.insert(
                board.zobrist_hash,
                TableEntry::new(depth, ply, EntryFlag::BetaCutOff, beta, best_move),
            );
            // TODO: Killer moves here (rstc)
            return beta;
        }

        if eval > alpha {
            alpha = eval;
            entry_flag = EntryFlag::Exact;
            principal_var_search = true;
            best_moves.clear();
            best_moves.push(*m);
            best_moves.append(&mut node_best_moves);
        }
    }

    if legal_moves == 0 {
        // Checkmate
        if board.side_in_check(board.to_move) {
            // Distance from root is returned in order for other recursive calls to determine
            // shortest viable checkmate path
            return -IN_CHECKMATE + ply as i32;
        }
        return STALEMATE;
    }

    search_info.transpos_table.insert(
        board.zobrist_hash,
        TableEntry::new(depth, ply, entry_flag, alpha, best_move),
    );

    alpha
}

pub(super) fn score_move(board: &Board, m: &Move) -> i32 {
    let mut score = 0;
    let piece_moving = board
        .piece_on_square(m.origin_square())
        .expect("There should be a piece here");
    let capture = board.piece_on_square(m.dest_square());
    if let Some(capture) = capture {
        score += 10 * piece_value(capture) - piece_value(piece_moving);
    }
    score
        + match m.promotion() {
            Some(Promotion::Queen) => piece_value(PieceName::Queen),
            Some(Promotion::Rook) => piece_value(PieceName::Rook),
            Some(Promotion::Bishop) => piece_value(PieceName::Bishop),
            Some(Promotion::Knight) => piece_value(PieceName::Knight),
            None => 0,
        }
}

const TT_BONUS: i32 = 500;
pub fn score_move_list(board: &Board, moves: &mut MoveList, table_move: Move) {
    for i in 0..moves.len {
        let (m, m_score) = moves.get_mut(i);
        let mut score = score_move(board, m);
        if table_move != Move::NULL && table_move == *m {
            score += TT_BONUS;
        }
        *m_score = score;
    }
}

pub fn sort_next_move(moves: &mut MoveList, idx: usize) {
    for i in (idx + 1)..moves.len {
        if moves.get_score(i) > moves.get_score(i) {
            moves.swap(idx, i);
        }
    }
}
