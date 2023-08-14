use crate::board::board::Board;
use crate::moves::moves::Move;
use crate::search::eval::eval;
use rustc_hash::FxHashMap;

pub struct BoardEntry {
    depth: i8,
    flag: EntryFlag,
    eval: i32,
    best_move: Move,
}

pub enum EntryFlag {
    Exact,
    AlphaCutOff,
    BetaCutOff,
}

/// Attempts to look up a board state in the transposition table. If found, returns the eval, and
/// if not found, places eval in the table before returning eval.
pub fn get_eval(board: &Board, transpos_table: &mut FxHashMap<u64, i32>) -> i32 {
    debug_assert_eq!(board.zobrist_hash, board.generate_hash());
    let hash = board.zobrist_hash;
    *transpos_table.entry(hash).or_insert_with(|| eval(board))
}

pub fn add_to_history(board: &mut Board) {
    let hash = board.zobrist_hash;
    board.history.push(hash);
}

pub fn remove_from_history(board: &mut Board) {
    board.history.pop();
}
