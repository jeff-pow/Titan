use std::time::{Duration, Instant};

use crate::board::board::Board;
use crate::engine::transposition::{EntryFlag, TableEntry};
use crate::moves::movegenerator::generate_psuedolegal_moves;
use crate::moves::moves::Move;
use crate::search::alpha_beta::alpha_beta;
use crate::search::pvs::{print_search_stats, INFINITY, MAX_SEARCH_DEPTH};
use crate::types::pieces::{QUEEN_PTS, ROOK_PTS};

use super::eval::eval;
use super::killers::store_killer_move;
use super::quiescence::quiescence;
use super::{SearchInfo, SearchType};

pub fn search(search_info: &mut SearchInfo) -> Move {
    let mut best_move = Move::NULL;
    let mut pv_moves = Vec::new();

    let mut recommended_time = Duration::ZERO;
    match search_info.search_type {
        SearchType::Time => {
            recommended_time = search_info
                .game_time
                .recommended_time(search_info.board.to_move);
        }
        SearchType::Depth => (),
        SearchType::Infinite => {
            search_info.iter_max_depth = MAX_SEARCH_DEPTH;
            search_info.max_depth = MAX_SEARCH_DEPTH;
        }
    }

    search_info.search_stats.start = Instant::now();
    search_info.iter_max_depth = 2;
    let mut prev_iter_score = 0;

    while search_info.iter_max_depth <= search_info.max_depth {
        search_info.sel_depth = search_info.iter_max_depth;

        let board = &search_info.board.to_owned();
        let g = mtdf(
            prev_iter_score,
            search_info.iter_max_depth,
            &mut pv_moves,
            board,
            search_info,
        );
        prev_iter_score = g;
        if !pv_moves.is_empty() {
            best_move = pv_moves[0];
        }
        print_search_stats(search_info, g, &pv_moves);

        if search_info.search_type == SearchType::Time
            && search_info
                .game_time
                .reached_termination(search_info.search_stats.start, recommended_time)
        {
            break;
        }
        search_info.iter_max_depth += 1;
    }

    assert_ne!(best_move, Move::NULL);

    best_move
}

fn mtdf(
    guess: i32,
    depth: i8,
    pv: &mut Vec<Move>,
    board: &Board,
    search_info: &mut SearchInfo,
) -> i32 {
    let mut g = guess;
    let mut upperbound = INFINITY;
    let mut lowerbound = -INFINITY;
    loop {
        let beta = if g == lowerbound { g + 1 } else { g };
        g = alpha_beta(depth, beta - 1, beta, pv, search_info, board);
        if g < beta {
            upperbound = g
        } else {
            lowerbound = g
        };
        if lowerbound >= upperbound {
            break;
        }
    }
    g
}
