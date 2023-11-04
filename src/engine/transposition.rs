use std::mem;

use crate::{board::board::Board, moves::moves::Move, search::search::NEAR_CHECKMATE};

#[derive(Clone, Copy, Default)]
pub struct TableEntry {
    key: u16,
    depth: i16,
    flag: EntryFlag,
    eval: i16,
    best_move: ShortMove,
}

impl TableEntry {
    pub fn key(&self) -> u16 {
        self.key
    }

    pub fn depth(&self) -> i32 {
        self.depth as i32
    }

    pub fn flag(&self) -> EntryFlag {
        self.flag
    }

    pub fn eval(&self) -> i32 {
        self.eval as i32
    }

    pub fn best_move(&self, b: &Board) -> Move {
        self.best_move.to_move(b)
    }
}

#[derive(Clone, Copy, Default, PartialEq)]
pub enum EntryFlag {
    #[default]
    None,
    Exact,
    AlphaUnchanged,
    BetaCutOff,
}

#[derive(Clone)]
pub struct TranspositionTable {
    vec: Vec<TableEntry>,
}

impl TranspositionTable {
    pub fn clear(&mut self) {
        self.vec.clear();
    }

    pub fn push(&mut self, hash: u64, m: Move, depth: i32, flag: EntryFlag, eval: i32, ply: i32) {
        let idx = index(hash);
        let key = hash as u16;

        let mut value = eval as i16;
        if value.abs() > NEAR_CHECKMATE as i16 {
            value += value.signum() * ply as i16;
        }

        // if m != Move::NULL || key != self.vec[idx].key {
        //     self.vec[idx].best_move = ShortMove::from_move(m);
        // }

        let entry = TableEntry {
            key,
            depth: depth as i16,
            flag,
            eval: value,
            best_move: ShortMove::from_move(m),
        };

        self.vec[idx] = entry;

        // if flag == EntryFlag::Exact || key != self.vec[idx].key {
        //     self.vec[idx] = entry;
        // }
    }

    pub fn tt_entry_get(&self, hash: u64, ply: i32) -> Option<TableEntry> {
        let idx = index(hash);
        let key = hash as u16;
        let mut entry = self.vec[idx];

        if entry.key != key {
            return None;
        }
        entry.eval -= if entry.eval.abs() > NEAR_CHECKMATE as i16 {
            entry.eval.signum() * ply as i16
        } else {
            0
        };
        Some(entry)
    }

    pub fn get(&self, ply: i32, depth: i32, alpha: i32, beta: i32, board: &Board) -> (Option<i32>, Move) {
        let idx = index(board.zobrist_hash);
        let key = board.zobrist_hash as u16;
        let entry = &self.vec[idx];

        if key != entry.key {
            return (None, Move::NULL);
        }

        let mut value = entry.eval as i32;
        if value.abs() > NEAR_CHECKMATE {
            value -= value.signum() * ply;
        }

        let eval = if depth <= entry.depth as i32
            && match entry.flag {
                EntryFlag::None => false,
                EntryFlag::Exact => true,
                EntryFlag::AlphaUnchanged => value <= alpha,
                EntryFlag::BetaCutOff => value >= beta,
            } {
            Some(value)
        } else {
            None
        };
        (eval, entry.best_move.to_move(board))
    }
}

impl Default for TranspositionTable {
    fn default() -> Self {
        Self {
            vec: vec![TableEntry::default(); TABLE_CAPACITY],
        }
    }
}

// Seen in virithidas and Alexandria
fn index(hash: u64) -> usize {
    ((u128::from(hash) * (TABLE_CAPACITY as u128)) >> 64) as usize
}

const TARGET_TABLE_SIZE_MB: usize = 64;
const BYTES_PER_MB: usize = 1024 * 1024;
const BYTES: usize = TARGET_TABLE_SIZE_MB * BYTES_PER_MB;
const ENTRY_SIZE: usize = mem::size_of::<TableEntry>();
const TABLE_CAPACITY: usize = BYTES / ENTRY_SIZE;

/// Storing a 32 bit move in the transposition table is a waste of space, as 16 bits contains all
/// you need. However, 32 bits is nice for extra information such as what piece moved, so moves are
/// truncated before being placed in transposition table, and extracted back into 32 bits before
/// being returned to caller
#[derive(Clone, Copy, Default, PartialEq)]
struct ShortMove(u16);

impl ShortMove {
    fn from_move(m: Move) -> Self {
        Self(m.as_u16())
    }

    fn to_move(self, board: &Board) -> Move {
        let m = Move(self.0 as u32);
        if m == Move::NULL {
            m
        } else {
            Move(self.0 as u32 | (board.piece_at(m.origin_square()).unwrap().idx() << 16) as u32)
        }
    }
}

#[cfg(test)]
mod transpos_tests {
    use crate::{
        board::fen::{build_board, STARTING_FEN},
        engine::transposition::EntryFlag,
        moves::moves::Move,
        types::{pieces::PieceName, square::Square},
    };

    use super::TranspositionTable;

    #[test]
    fn transpos_table() {
        let b = build_board(STARTING_FEN);
        let mut table = TranspositionTable::default();
        let (eval, m) = table.get(0, 0, -500, 500, &b);
        assert!(eval.is_none());
        assert_eq!(m, Move::NULL);

        let m = Move::new(Square(12), Square(28), PieceName::Pawn);
        table.push(b.zobrist_hash, m, 4, EntryFlag::Exact, 25, 3);
        let (eval, m1) = table.get(2, 2, -250, 250, &b);
        assert_eq!(25, eval.unwrap());
        assert_eq!(m, m1);
    }
}
