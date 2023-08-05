use std::collections::HashMap;
use std::time::Instant;

use crate::board::Board;
use crate::moves::{generate_moves, Move, Promotion};
use crate::pieces::piece_value;
use crate::zobrist::{
    add_to_triple_repetition_map, check_for_3x_repetition, get_transposition,
    remove_from_triple_repetition_map,
};
use std::cmp::{max, min};

pub const IN_CHECK_MATE: i32 = 100000;
pub const INFINITY: i32 = 9999999;

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
        let mut new_b = *board;
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
        let mut new_b = *board;
        new_b.make_move(m);
        count += count_moves(depth - 1, &new_b);
    }
    count
}

pub struct Search {
    best_moves: Vec<Move>,
    transpos_table: HashMap<u64, i32>,
}

/// Generates the optimal move for a given position using alpha beta pruning and basic transposition tables.
pub fn search(board: &Board, depth: i32, triple_repetitions: &mut HashMap<u64, u8>) -> Move {
    let mut best_move = Move::invalid();
    let mut transpos_table = HashMap::new();

    for i in 1..=depth {
        let start = Instant::now();
        let mut alpha = -INFINITY;
        let beta = INFINITY;

        let mut moves = generate_moves(board);
        moves.sort_by_key(|m| score_move(board, m));
        moves.reverse();

        for m in &moves {
            let mut new_b = *board;
            new_b.make_move(m);
            add_to_triple_repetition_map(&new_b, triple_repetitions);

            let eval = -search_helper(
                &new_b,
                i - 1,
                1,
                -beta,
                -alpha,
                triple_repetitions,
                &mut transpos_table,
            );

            remove_from_triple_repetition_map(&new_b, triple_repetitions);

            if eval >= beta {
                break;
            }
            if eval > alpha {
                alpha = eval;
                best_move = *m;
            }
        }
        let elapsed_time = start.elapsed().as_millis();
        println!("info time {} depth {}", elapsed_time, i);
    }
    best_move
}

/// Helper function for search. My implementation does not record a queue of optimal moves, simply
/// the best one for a current position. Because of this, we only care about the score of each move
/// at the surface level. Which moves lead to this optimial position do not matter, so we just
/// return the evaluation of the best possible position *eventually* down a tree from the first
/// level of moves. The search function is responsible for keeping track of which move is the best
/// based off of these values.
fn search_helper(
    board: &Board,
    depth: i32,
    dist_from_root: i32,
    mut alpha: i32,
    mut beta: i32,
    triple_repetitions: &mut HashMap<u64, u8>,
    transpos_table: &mut HashMap<u64, i32>,
) -> i32 {
    // Return an evaluation of the board if maximum depth has been reached.
    if depth == 0 {
        return get_transposition(board, transpos_table);
    }
    // Stalemate if a board position has appeared three times
    if check_for_3x_repetition(board, triple_repetitions) {
        return 0;
    }
    // Skip move if a path to checkmate has already been found in this path
    alpha = max(alpha, -IN_CHECK_MATE + dist_from_root);
    beta = min(beta, IN_CHECK_MATE - dist_from_root);
    if alpha >= beta {
        return alpha;
    }

    let mut moves = generate_moves(board);
    moves.sort_unstable_by_key(|m| (score_move(board, m)));
    moves.reverse();

    if moves.is_empty() {
        // Checkmate
        if board.under_attack(board.to_move) {
            // Distance from root is returned in order for other recursive calls to determine
            // shortest viable checkmate path
            return -IN_CHECK_MATE + dist_from_root;
        }
        // Stalemate
        return 0;
    }

    for m in &moves {
        let mut new_b = *board;
        new_b.make_move(m);
        add_to_triple_repetition_map(&new_b, triple_repetitions);

        let eval = -search_helper(
            &new_b,
            depth - 1,
            dist_from_root + 1,
            -beta,
            -alpha,
            triple_repetitions,
            transpos_table,
        );

        remove_from_triple_repetition_map(&new_b, triple_repetitions);

        if eval >= beta {
            return beta;
        }
        alpha = max(alpha, eval);
    }
    alpha
}

fn score_move(board: &Board, m: &Move) -> i32 {
    let mut score = 0;
    let piece_moving = board
        .piece_on_square(m.origin_square().into())
        .expect("There should be a piece here");
    let capture = board.piece_on_square(m.dest_square().into());
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
