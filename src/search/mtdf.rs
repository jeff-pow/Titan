use std::time::{Duration, Instant};

use crate::board::board::Board;
use crate::engine::transposition::{EntryFlag, TableEntry};
use crate::moves::movegenerator::generate_psuedolegal_moves;
use crate::moves::moves::Move;
use crate::types::pieces::{QUEEN_PTS, ROOK_PTS};

use super::eval::eval;
use super::killers::store_killer_move;
use super::quiescence::quiescence;
use super::{SearchInfo, SearchType};

pub const CHECKMATE: i32 = 100000;
pub const STALEMATE: i32 = 0;
pub const NEAR_CHECKMATE: i32 = CHECKMATE - 1000;
pub const INFINITY: i32 = 9999999;
pub const MAX_SEARCH_DEPTH: i8 = 100;

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

fn print_search_stats(search_info: &SearchInfo, eval: i32, pv: &[Move]) {
    print!(
        "info time {} seldepth {} depth {} nodes {} nps {} score cp {} pv ",
        search_info.search_stats.start.elapsed().as_millis(),
        search_info.sel_depth,
        search_info.iter_max_depth,
        search_info.search_stats.nodes_searched,
        (search_info.search_stats.nodes_searched as f64
            / search_info.search_stats.start.elapsed().as_secs_f64()) as i64,
        eval
    );
    for m in pv.iter() {
        print!("{} ", m.to_lan());
    }
    println!();
}

const FUTIL_MARGIN: i32 = 200;
const FUTIL_DEPTH: i8 = 1;
const EXT_FUTIL_MARGIN: i32 = ROOK_PTS;
const EXT_FUTIL_DEPTH: i8 = 2;
const RAZOR_MARGIN: i32 = QUEEN_PTS;
const RAZORING_DEPTH: i8 = 3;

/// Principal variation search - uses reduced alpha beta windows around a likely best move candidate
/// to refute other variations
pub(super) fn alpha_beta(
    mut depth: i8,
    mut alpha: i32,
    beta: i32,
    pv: &mut Vec<Move>,
    search_info: &mut SearchInfo,
    board: &Board,
) -> i32 {
    let ply = search_info.iter_max_depth - depth;
    let is_root = ply == 0;
    search_info.sel_depth = search_info.sel_depth.max(ply);
    // Needed since the function can calculate extensions in cases where it finds itself in check
    if ply >= MAX_SEARCH_DEPTH {
        if board.side_in_check(board.to_move) {
            let mut node_pvs = Vec::new();
            return quiescence(ply, alpha, beta, &mut node_pvs, search_info, board);
        }
        return eval(board);
    }

    if ply > 0 {
        if board.is_draw() {
            return STALEMATE;
        }

        // Determines if there is a faster path to checkmate than evaluating the current node, and
        // if there is, it returns early
        let alpha = alpha.max(-CHECKMATE + ply as i32);
        let beta = beta.min(CHECKMATE - ply as i32);
        if alpha >= beta {
            return alpha;
        }
    }

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

    let in_check = board.side_in_check(board.to_move);

    if in_check {
        depth += 1;
    }

    if depth <= 0 {
        return quiescence(ply, alpha, beta, pv, search_info, board);
    }

    search_info.search_stats.nodes_searched += 1;

    let mut moves = generate_psuedolegal_moves(board);
    let mut legal_moves = 0;
    moves.score_move_list(ply, board, table_move, search_info);

    let mut score = -INFINITY;
    let mut entry_flag = EntryFlag::AlphaCutOff;
    let mut best_move = Move::NULL;

    if !in_check && depth == FUTIL_DEPTH && search_info.iter_max_depth > FUTIL_DEPTH {
        let eval = eval(board);
        if eval + FUTIL_MARGIN < alpha {
            let mut node_pvs = Vec::new();
            return quiescence(ply, alpha, beta, &mut node_pvs, search_info, board);
        }
    }

    if !in_check && depth == EXT_FUTIL_DEPTH && search_info.iter_max_depth > EXT_FUTIL_DEPTH {
        let eval = eval(board);
        if eval + EXT_FUTIL_MARGIN < alpha {
            let mut node_pvs = Vec::new();
            return quiescence(ply, alpha, beta, &mut node_pvs, search_info, board);
        }
    }

    if !in_check && depth == RAZORING_DEPTH && search_info.iter_max_depth > RAZORING_DEPTH {
        let eval = eval(board);
        if eval + RAZOR_MARGIN < alpha {
            let mut node_pvs = Vec::new();
            return quiescence(ply, alpha, beta, &mut node_pvs, search_info, board);
        }
    }

    // Null pruning I haven't got around to testing yet
    // if !fprune && !board.side_in_check(board.to_move) && null_ok(board) {
    //     let mut node_pvs = Vec::new();
    //     let mut new_b = board.to_owned();
    //     new_b.to_move = new_b.to_move.opp();
    //     let null_eval = -pvs(
    //         depth - 1,
    //         -beta,
    //         -beta + 1,
    //         &mut node_pvs,
    //         search_info,
    //         &new_b,
    //     );
    //     if null_eval >= beta {
    //         return null_eval;
    //     }
    // }

    for i in 0..moves.len {
        let mut new_b = board.to_owned();
        moves.sort_next_move(i);
        let m = moves.get_move(i);
        new_b.make_move(m);
        // Just generate psuedolegal moves to save computation time on legality for moves that will be
        // pruned
        if new_b.side_in_check(board.to_move) {
            continue;
        }
        legal_moves += 1;

        let mut node_pvs = Vec::new();

        let eval = -alpha_beta(depth - 1, -beta, -alpha, &mut node_pvs, search_info, &new_b);

        if eval > score {
            score = eval;
            best_move = *m;
        }

        if eval >= beta {
            search_info.transpos_table.insert(
                board.zobrist_hash,
                TableEntry::new(depth, ply, EntryFlag::BetaCutOff, eval, best_move),
            );
            let capture = board.piece_at(m.dest_square());

            // Store a killer move if it is not a capture, but good enough to cause a beta cutoff
            // anyway
            if capture.is_none() {
                store_killer_move(ply, m, search_info);
            }
            return eval;
        }

        if eval > alpha {
            alpha = eval;
            entry_flag = EntryFlag::Exact;
            // A principal variation has been found, so we can do pvs on the remaining nodes of this level
            pv.clear();
            pv.push(*m);
            pv.append(&mut node_pvs);
        }
    }

    if legal_moves == 0 {
        // Checkmate
        if board.side_in_check(board.to_move) {
            // Distance from root is returned in order for other recursive calls to determine
            // shortest viable checkmate path
            return -CHECKMATE + ply as i32;
        }
        return STALEMATE;
    }

    search_info.transpos_table.insert(
        board.zobrist_hash,
        TableEntry::new(depth, ply, entry_flag, score, best_move),
    );

    score
}

/// Arbitrary value determining if a side is in endgame yet
const ENDGAME_THRESHOLD: i32 = 1750;
#[allow(dead_code)]
fn null_ok(board: &Board) -> bool {
    board.material_val[board.to_move as usize] > ENDGAME_THRESHOLD
        && board.material_val[board.to_move.opp() as usize] > ENDGAME_THRESHOLD
}
