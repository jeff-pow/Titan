use std::{
    mem::{self, transmute},
    sync::atomic::{AtomicU64, Ordering},
};

use crate::{board::board::Board, moves::moves::Move, search::search::NEAR_CHECKMATE};

#[derive(Clone, Copy, Debug, Default)]
#[repr(C)]
/// Storing a 32 bit move in the transposition table is a waste of space, as 16 bits contains all
/// you need. However, 32 bits is nice for extra information such as what piece moved, so moves are
/// truncated before being placed in transposition table, and extracted back into 32 bits before
/// being returned to caller
pub struct TableEntry {
    depth: u8,
    other: u8,
    key: u16,
    eval: i16,
    best_move: u16,
}

impl TableEntry {
    pub fn key(self) -> u16 {
        self.key
    }

    pub fn depth(self) -> i32 {
        self.depth as i32
    }

    pub fn flag(self) -> EntryFlag {
        match self.other & 0b11 {
            0 => EntryFlag::None,
            1 => EntryFlag::AlphaUnchanged,
            2 => EntryFlag::BetaCutOff,
            3 => EntryFlag::Exact,
            _ => unreachable!(),
        }
    }

    fn age(self) -> u64 {
        self.other as u64 >> 2
    }

    pub fn eval(self) -> i32 {
        self.eval as i32
    }

    pub fn best_move(self, b: &Board) -> Move {
        let m = Move(self.best_move as u32);
        // The reasoning here is if there is indeed a piece at the square in question, we can extract it.
        // Otherwise use 0b111 which isn't a flag at all, and will thus not show equivalent to any
        // generated moves. If the move is null, it won't be generated, and won't be falsely scored either
        let p = b.piece_at(m.origin_square()).map_or(0b111, |p| p.idx());
        Move(self.best_move as u32 | (p as u32 & 0b111) << 16)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum EntryFlag {
    #[default]
    None,
    AlphaUnchanged,
    BetaCutOff,
    Exact,
}

#[derive(Default)]
struct U64Wrapper(AtomicU64);
impl Clone for U64Wrapper {
    fn clone(&self) -> Self {
        Self(AtomicU64::new(self.0.load(std::sync::atomic::Ordering::Relaxed)))
    }
}

#[derive(Clone)]
pub struct TranspositionTable {
    vec: Box<[U64Wrapper]>,
    age: U64Wrapper,
}

impl TranspositionTable {
    pub fn clear(&self) {
        for x in self.vec.iter() {
            x.0.store(0, Ordering::Relaxed);
        }
    }

    fn new() -> Self {
        Self {
            vec: vec![U64Wrapper::default(); TABLE_CAPACITY].into_boxed_slice(),
            age: U64Wrapper::default(),
        }
    }

    fn age(&self) -> u64 {
        self.age.0.load(Ordering::Relaxed)
    }

    pub fn age_up(&self) {
        // Keep age under 63 b/c that is the max age that fits in a table entry
        self.age.0.store(63.min(self.age() + 1), Ordering::Relaxed);
    }

    pub fn store(&self, hash: u64, m: Move, depth: i32, flag: EntryFlag, eval: i32, ply: i32) {
        let idx = index(hash);
        let key = hash as u16;

        let old_entry: TableEntry = unsafe { transmute(self.vec[idx].0.load(Ordering::Relaxed)) };

        if self.age() != old_entry.age()
            || old_entry.key != key
            || flag == EntryFlag::Exact && old_entry.flag() != EntryFlag::Exact
            || depth as usize + 3 > old_entry.depth as usize
        {
            // Don't overwrite a best move with a null move

            let best_m = if m == Move::NULL && key == old_entry.key {
                old_entry.best_move
            } else {
                m.as_u16()
            };

            let mut eval = eval;

            eval += if eval.abs() > NEAR_CHECKMATE {
                eval.signum() * ply
            } else {
                0
            };

            let entry = TableEntry {
                key,
                depth: depth as u8,
                other: (self.age() << 2) as u8 | flag as u8,
                eval: eval as i16,
                best_move: best_m,
            };

            let number: u64 = unsafe { transmute(entry) };

            if let Some(x) = self.vec.get(idx) {
                x.0.store(number, Ordering::SeqCst)
            }
        }
    }

    pub fn tt_entry_get(&self, hash: u64, ply: i32) -> Option<TableEntry> {
        let idx = index(hash);
        let key = hash as u16;

        let mut entry: TableEntry = unsafe { transmute(self.vec[idx].0.load(Ordering::Relaxed)) };

        if entry.key != key {
            return None;
        }

        entry.eval -= if entry.eval.abs() >= NEAR_CHECKMATE as i16 {
            entry.eval.signum() * ply as i16
        } else {
            0
        };

        Some(entry)
    }

    #[allow(dead_code)]
    fn get(&self, ply: i32, depth: i32, alpha: i32, beta: i32, board: &Board) -> (Option<i32>, Move) {
        let idx = index(board.zobrist_hash);
        let key = board.zobrist_hash as u16;

        let wrapper = self.vec[idx].clone();
        let entry: TableEntry = unsafe { transmute(wrapper.0.load(Ordering::SeqCst)) };

        if key != entry.key {
            return (None, Move::NULL);
        }

        let mut value = entry.eval as i32;
        if value.abs() > NEAR_CHECKMATE {
            value -= value.signum() * ply;
        }

        let eval = if depth <= entry.depth as i32
            && match entry.flag() {
                EntryFlag::None => false,
                EntryFlag::Exact => true,
                EntryFlag::AlphaUnchanged => value <= alpha,
                EntryFlag::BetaCutOff => value >= beta,
            } {
            Some(value)
        } else {
            None
        };
        (eval, entry.best_move(board))
    }
}

impl Default for TranspositionTable {
    fn default() -> Self {
        println!("{} elements in hash table", TABLE_CAPACITY);
        Self::new()
    }
}

// Seen in virithidas and Alexandria
fn index(hash: u64) -> usize {
    ((u128::from(hash) * (TABLE_CAPACITY as u128)) >> 64) as usize
}

const TARGET_TABLE_SIZE_MB: usize = 256;
const BYTES_PER_MB: usize = 1024 * 1024;
const TARGET_BYTES: usize = TARGET_TABLE_SIZE_MB * BYTES_PER_MB;
const ENTRY_SIZE: usize = mem::size_of::<TableEntry>();
const TABLE_CAPACITY: usize = TARGET_BYTES / ENTRY_SIZE;

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
        let table = TranspositionTable::default();
        let (eval, m) = table.get(0, 0, -500, 500, &b);
        assert!(eval.is_none());
        assert_eq!(m, Move::NULL);

        let m = Move::new(Square(12), Square(28), PieceName::Pawn);
        table.store(b.zobrist_hash, m, 4, EntryFlag::Exact, 25, 0);
        let (eval, m1) = table.get(2, 2, -250, 250, &b);
        assert_eq!(25, eval.unwrap());
        assert_eq!(m, m1);
    }
}
