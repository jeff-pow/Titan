use rustc_hash::FxHashMap;
use std::time::Instant;

use std::cmp::{max, min};

use crate::board::{board::Board, zobrist::check_for_3x_repetition};
use crate::engine::transposition::{add_to_history, remove_from_history, EntryFlag, TableEntry};
use crate::moves::movegenerator::generate_moves;
use crate::moves::moves::Move;
use crate::moves::moves::Promotion;
use crate::types::pieces::piece_value;

use super::eval::eval;
use super::game_time::GameTime;
use super::quiescence::quiescence;
use super::search_stats::{self, SearchStats};
use super::{SearchInfo, SearchType};

pub const IN_CHECKMATE: i32 = 100000;
pub const STALEMATE: i32 = 0;
pub const NEAR_CHECKMATE: i32 = IN_CHECKMATE - 1000;
pub const INFINITY: i32 = 9999999;
pub const MAX_SEARCH_DEPTH: i8 = 30;

#[inline(always)]
fn dist_from_root(max_depth: i32, current_depth: i32) -> i32 {
    max_depth - current_depth
}

pub fn search(search_info: &mut SearchInfo) -> Move {
    let mut max_depth = 1;
    let mut best_move = Move::invalid();
    let mut best_moves = Vec::new();
    let determine_game_time = search_info.search_type == SearchType::Time;
    if determine_game_time {
        let board = &search_info.board;
        let history_len = board.history.len();
        search_info
            .game_time
            .update_recommended_time(board.to_move, history_len);
    }

    if search_info.search_type == SearchType::Depth {
        max_depth = search_info.depth;
    }

    let alpha_start = -INFINITY;
    let beta_start = INFINITY;

    search_info.search_stats.start = Instant::now();
    let mut current_depth = 1;
    while current_depth <= MAX_SEARCH_DEPTH && current_depth <= max_depth {
        if search_info
            .game_time
            .reached_termination(search_info.search_stats.start)
        {
            break;
        }
        search_info.depth = current_depth;
        let eval = alpha_beta(
            current_depth,
            0,
            alpha_start,
            beta_start,
            &mut best_moves,
            search_info,
        );

        if !best_moves.is_empty() {
            best_move = best_moves[0];
        }
        current_depth += 1;
    }

    best_move
}

fn alpha_beta(
    mut depth: i8,
    mut ply: i8,
    mut alpha: i32,
    beta: i32,
    best_moves: &mut Vec<Move>,
    search_info: &mut SearchInfo,
) -> i32 {
    let is_root = ply == 0;
    let mut principal_var_search = false;
    if ply >= MAX_SEARCH_DEPTH {
        return eval(&search_info.board);
    }

    let is_check = search_info.board.side_in_check(search_info.board.to_move);

    if is_check {
        depth += 1;
    }

    if depth <= 0 {
        return quiescence(ply, alpha, beta, best_moves, search_info);
    }

    search_info.search_stats.nodes_searched += 1;

    let (table_value, table_move) = {
        let hash = search_info.board.zobrist_hash;
        let entry = search_info.transpos_table.get(&hash);
        if let Some(entry) = entry {
            entry.get(depth, ply, alpha, beta)
        } else {
            (None, Move::invalid())
        }
    };
    if let Some(eval) = table_value {
        if !is_root {
            return eval;
        }
    }

    let mut moves = generate_moves(&search_info.board);
    if table_move != Move::invalid() {
        if let Some(index) = moves.iter().position(|&m| m == table_move) {
            moves.swap(index, 0);
        } else {
            panic!("Table move wasn't in generated move list...")
        }
    }
    moves[1..].sort_unstable_by_key(|m| score_move(&search_info.board, m));
    moves.reverse();

    if moves.is_empty() {
        // Checkmate
        if search_info.board.side_in_check(search_info.board.to_move) {
            // Distance from root is returned in order for other recursive calls to determine
            // shortest viable checkmate path
            return -IN_CHECKMATE + ply as i32;
        }
        return STALEMATE;
    }

    let mut best_eval = -INFINITY;
    let mut entry_flag = EntryFlag::AlphaCutOff;
    let mut best_move = Move::invalid();

    for m in moves.iter() {
        let mut node_best_moves = Vec::new();

        let mut eval = -INFINITY;

        // Draw if a position has occurred three times
        if check_for_3x_repetition(&search_info.board) {
            return 0;
        }

        if principal_var_search {
            eval = -alpha_beta(
                depth - 1,
                ply + 1,
                -alpha - 1,
                -alpha,
                &mut node_best_moves,
                search_info,
            );

            // Check if we failed the PVS.
            if (eval > alpha) && (eval < beta) {
                eval = -alpha_beta(
                    depth - 1,
                    ply + 1,
                    -beta,
                    -alpha,
                    &mut node_best_moves,
                    search_info,
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
            );
        }

        if eval > best_eval {
            best_eval = eval;
            best_move = *m;
        }

        if eval >= beta {
            search_info.transpos_table.insert(
                search_info.board.zobrist_hash,
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

    search_info.transpos_table.insert(
        search_info.board.zobrist_hash,
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
    score += match m.promotion() {
        Some(Promotion::Queen) => 900,
        Some(Promotion::Rook) => 500,
        Some(Promotion::Bishop) => 300,
        Some(Promotion::Knight) => 300,
        None => 0,
    };
    score += score;
    score
}

#[allow(dead_code)]
/// Counts and times the action of generating moves to a certain depth. Prints this information
pub fn time_move_generation(board: &Board, depth: i32) {
    for i in 1..=depth {
        let start = Instant::now();
        print!("{}", count_moves(i, board));
        let elapsed = start.elapsed();
        print!(" moves generated in {:?} ", elapsed);
        println!("at a depth of {i}");
    }
}

/// https://www.chessprogramming.org/Perft
pub fn perft(board: &Board, depth: i32) -> usize {
    let mut total = 0;
    let moves = generate_moves(board);
    for m in &moves {
        let mut new_b = board.to_owned();
        new_b.make_move(m);
        let count = count_moves(depth - 1, &new_b);
        total += count;
        println!("{}: {}", m.to_lan(), count);
    }
    println!("\nNodes searched: {}", total);
    total
}

/// Recursively counts the number of moves down to a certain depth
fn count_moves(depth: i32, board: &Board) -> usize {
    if depth == 0 {
        return 1;
    }
    let mut count = 0;
    let moves = generate_moves(board);
    for m in &moves {
        let mut new_b = board.to_owned();
        new_b.make_move(m);
        count += count_moves(depth - 1, &new_b);
    }
    count
}
