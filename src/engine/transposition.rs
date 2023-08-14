use crate::board::board::Board;
use crate::moves::moves::Move;
use crate::search::alpha_beta::NEAR_CHECKMATE;
use crate::search::eval::eval;
use rustc_hash::FxHashMap;

pub struct TableEntry {
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

impl TableEntry {
    pub fn new(depth: i8, ply: i8, flag: EntryFlag, eval: i32, best_move: Move) -> Self {
        let mut v = eval;

        if v > NEAR_CHECKMATE {
            v += ply as i32;
        }
        if v < NEAR_CHECKMATE {
            v -= ply as i32;
        }

        Self {
            depth,
            flag,
            eval: v,
            best_move,
        }
    }

    pub fn get(&self, depth: i8, ply: i8, alpha: i32, beta: i32) -> (Option<i32>, Move) {
        let mut eval: Option<i32> = None;
        if self.depth >= depth {
            match self.flag {
                EntryFlag::Exact => {
                    let mut value = self.eval;
                    if value > NEAR_CHECKMATE {
                        value -= ply as i32;
                    }
                    if value < NEAR_CHECKMATE {
                        value += ply as i32;
                    }
                    eval = Some(value);
                }
                EntryFlag::AlphaCutOff => {
                    if self.eval <= alpha {
                        eval = Some(alpha);
                    }
                }
                EntryFlag::BetaCutOff => {
                    if self.eval >= beta {
                        eval = Some(beta);
                    }
                }
            }
        }
        (eval, self.best_move)
    }
}

/// Attempts to look up a board state in the transposition table. If found, returns the eval, and
/// if not found, places eval in the table before returning eval.
pub fn get_eval(
    board: &Board,
    transpos_table: &mut FxHashMap<u64, TableEntry>,
    depth: i8,
    ply: i8,
    entry_flag: EntryFlag,
    best_move: Move,
) -> (Option<i32>, Move) {
    debug_assert_eq!(board.zobrist_hash, board.generate_hash());
    let hash = board.zobrist_hash;
    let entry = *transpos_table.entry(hash).or_insert_with(|| {
        let eval = eval(board);
        TableEntry::new(depth, ply, entry_flag, eval, best_move)
    });
    entry.get(depth, ply, alpha, beta)
}

pub fn add_to_history(board: &mut Board) {
    let hash = board.zobrist_hash;
    board.history.push(hash);
}

pub fn remove_from_history(board: &mut Board) {
    board.history.pop();
}
