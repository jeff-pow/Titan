use std::mem;

use crate::{board::board::Board, moves::moves::Move, search::search::NEAR_CHECKMATE};
use rustc_hash::FxHashMap;

pub struct TableEntry {
    depth: i16,
    flag: EntryFlag,
    eval: i32,
    best_move: ShortMove,
}

#[derive(Clone, Copy, PartialEq)]
pub struct ShortMove(u16);

impl ShortMove {
    pub fn from_move(m: Move) -> Self {
        Self(m.as_u16())
    }

    pub fn as_u32(&self) -> u32 {
        self.0 as u32
    }
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
                    // eval = Some(eval.min(beta))
                }
                EntryFlag::BetaCutOff => {
                    if self.eval >= beta {
                        eval = Some(beta);
                    }
                    // eval = Some(eval.max(alpha))
                }
            }
        }
        let best_move = Move::from_short_move(self.best_move, board);
        (eval, best_move)
    }
}

const TARGET_TABLE_SIZE_MB: usize = 64;
const BYTES_PER_MB: usize = 1024 * 1024;
pub fn get_table() -> FxHashMap<u64, TableEntry> {
    let entry_size = mem::size_of::<TableEntry>();
    FxHashMap::with_capacity_and_hasher(TARGET_TABLE_SIZE_MB * BYTES_PER_MB / entry_size, Default::default())
}
