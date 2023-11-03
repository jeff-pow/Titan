use std::mem;

use crate::{board::board::Board, moves::moves::Move, search::search::NEAR_CHECKMATE};

pub struct TableEntry {
    depth: i16,
    flag: EntryFlag,
    eval: i32,
    best_move: ShortMove,
}

#[derive(PartialEq)]
pub enum EntryFlag {
    Exact,
    AlphaUnchanged,
    BetaCutOff,
}

impl TableEntry {
    pub fn new(depth: i32, ply: i32, flag: EntryFlag, eval: i32, best_move: Move) -> Self {
        let mut v = eval;

        if eval.abs() > NEAR_CHECKMATE {
            let sign = eval.signum();
            v = (eval * sign + ply) * sign;
        }

        Self {
            depth: depth as i16,
            flag,
            eval: v,
            best_move: ShortMove::from_move(best_move),
        }
    }

    pub fn get(&self, depth: i32, ply: i32, alpha: i32, beta: i32, board: &Board) -> (Option<i32>, Move) {
        let mut eval: Option<i32> = None;
        if self.depth as i32 >= depth {
            match self.flag {
                EntryFlag::Exact => {
                    let mut value = self.eval;

                    if value.abs() > NEAR_CHECKMATE {
                        let sign = value.signum();
                        value = self.eval * sign - ply
                    }

                    eval = Some(value);
                }
                EntryFlag::AlphaUnchanged => {
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
        // let best_move = Move::from_short_move(self.best_move, board);
        let best_move = self.best_move.to_move(board);
        (eval, best_move)
    }
}

pub struct TranspositionTable {
    arr: Vec<TableEntry>,
}

impl TranspositionTable {
    pub fn new() -> Self {
        let entry_size = mem::size_of::<TableEntry>();
        Self {
            arr: Vec::with_capacity(BYTES / entry_size),
        }
    }

    pub fn clear(&mut self) {
        self.arr.clear();
    }

    pub fn push(&self, hash: u64, m: Move, depth: i32, flag: EntryFlag, eval: i32, ply: i32) {
        let key = hash >> 48;
        let idx = hash as usize & self.arr.len() - 1;
        let entry = TableEntry::new(depth, ply, flag, eval, m);
    }
}

const TARGET_TABLE_SIZE_MB: usize = 64;
const BYTES_PER_MB: usize = 1024 * 1024;
const BYTES: usize = TARGET_TABLE_SIZE_MB * BYTES_PER_MB;

/// Storing a 32 bit move in the transposition table is a waste of space, as 16 bits contains all
/// you need. However, 32 bits is nice for extra information such as what piece moved, so moves are
/// truncated before being placed in transposition table, and extracted back into 32 bits before
/// being returned to caller
#[derive(Clone, Copy, PartialEq)]
struct ShortMove(u16);

impl ShortMove {
    fn from_move(m: Move) -> Self {
        Self(m.as_u16())
    }

    fn to_move(self, board: &Board) -> Move {
        let m = Move::raw(self.0 as u32);
        if m == Move::NULL {
            m
        } else {
            Move::raw(self.0 as u32 | board.piece_at(m.origin_square()).expect("There is a piece here").idx() as u32)
        }
    }
}
