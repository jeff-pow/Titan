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
    key: u64,
    depth: u8,
    other: u8,
    score: i16,
    best_move: u16,
    static_eval: i16,
}

impl TableEntry {
    pub fn static_eval(self) -> i32 {
        self.static_eval as i32
    }

    pub fn key(self) -> u64 {
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
        self.score as i32
    }

    pub fn best_move(self, b: &Board) -> Move {
        let m = Move(self.best_move as u32);
        // The reasoning here is if there is indeed a piece at the square in question, we can extract it.
        // Otherwise use 0b111 which isn't a flag at all, and will thus not show equivalent to any
        // generated moves. If the move is null, it won't be generated, and won't be falsely scored either
        let p = b.piece_at(m.origin_square()).map_or(0b111, |p| p as u32);
        Move(self.best_move as u32 | (p & 0b111) << 16)
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
        Self(AtomicU64::new(self.0.load(Ordering::Relaxed)))
    }
}

#[derive(Default)]
struct InternalEntry {
    zobrist_hash: AtomicU64,
    remainder: AtomicU64,
}

impl Clone for InternalEntry {
    fn clone(&self) -> Self {
        Self {
            zobrist_hash: AtomicU64::new(self.zobrist_hash.load(Ordering::Relaxed)),
            remainder: AtomicU64::new(self.remainder.load(Ordering::Relaxed)),
        }
    }
}

impl From<TableEntry> for (u64, u64) {
    fn from(value: TableEntry) -> Self {
        let (mut zobrist, remainder): (u64, u64) = unsafe { transmute(value) };
        zobrist ^= remainder;
        (zobrist, remainder)
    }
}

impl From<InternalEntry> for TableEntry {
    fn from(value: InternalEntry) -> Self {
        let (mut zobrist, remainder): (u64, u64) = unsafe { transmute(value) };
        zobrist ^= remainder;
        unsafe { transmute((zobrist, remainder)) }
    }
}

#[derive(Clone)]
pub struct TranspositionTable {
    vec: Box<[InternalEntry]>,
    age: U64Wrapper,
}

pub const TARGET_TABLE_SIZE_MB: usize = 64;
const BYTES_PER_MB: usize = 1024 * 1024;
const TARGET_BYTES: usize = TARGET_TABLE_SIZE_MB * BYTES_PER_MB;
const ENTRY_SIZE: usize = mem::size_of::<TableEntry>();
const TABLE_CAPACITY: usize = TARGET_BYTES / ENTRY_SIZE;

impl TranspositionTable {
    pub fn clear(&self) {
        for x in self.vec.iter() {
            x.zobrist_hash.store(0, Ordering::Relaxed);
            x.remainder.store(0, Ordering::Relaxed);
        }
    }

    /// Size here is the desired size in MB
    pub fn new(size: usize) -> Self {
        let target_size = size * 1024 * 1024;
        let table_capacity = target_size / ENTRY_SIZE;
        println!("{} elements in hash table", table_capacity);
        Self {
            vec: vec![InternalEntry::default(); table_capacity].into_boxed_slice(),
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

    #[allow(clippy::too_many_arguments)]
    pub fn store(
        &self,
        hash: u64,
        m: Move,
        depth: i32,
        flag: EntryFlag,
        mut score: i32,
        ply: i32,
        is_pv: bool,
        static_eval: i32,
    ) {
        let idx = index(hash);
        let key = hash;

        let old_entry = unsafe { TableEntry::from(self.vec.get_unchecked(idx).clone()) };

        // Conditions from Alexandria
        if old_entry.age() != self.age()
            || old_entry.key != key
            || flag == EntryFlag::Exact
            || depth as usize + 5 + 2 * usize::from(is_pv) > old_entry.depth as usize
        {
            // Don't overwrite a best move with a null move
            let best_m = if m == Move::NULL && key == old_entry.key {
                old_entry.best_move
            } else {
                m.as_u16()
            };

            if score > NEAR_CHECKMATE {
                score += ply;
            } else if score < -NEAR_CHECKMATE {
                score -= ply;
            }

            let entry = TableEntry {
                key,
                depth: depth as u8,
                other: (self.age() << 2) as u8 | flag as u8,
                score: score as i16,
                best_move: best_m,
                static_eval: static_eval as i16,
            };

            let (zobrist_hash, remainder) = entry.into();
            unsafe {
                self.vec
                    .get_unchecked(idx)
                    .zobrist_hash
                    .store(zobrist_hash, Ordering::Relaxed);
                self.vec
                    .get_unchecked(idx)
                    .remainder
                    .store(remainder, Ordering::Relaxed);
            }
        }
    }

    pub fn get(&self, hash: u64, ply: i32) -> Option<TableEntry> {
        let idx = index(hash);
        let key = hash;

        let mut entry = unsafe { TableEntry::from(self.vec.get_unchecked(idx).clone()) };

        if entry.key != key {
            return None;
        }

        if entry.score > NEAR_CHECKMATE as i16 {
            entry.score -= ply as i16;
        } else if entry.score < -NEAR_CHECKMATE as i16 {
            entry.score += ply as i16;
        }

        Some(entry)
    }
}

fn index(hash: u64) -> usize {
    ((u128::from(hash) * (TABLE_CAPACITY as u128)) >> 64) as usize
}

#[cfg(test)]
mod transpos_tests {

    #[test]
    fn transpos_table() {
        // let b = build_board(STARTING_FEN);
        // let table = TranspositionTable::default();
        // let (eval, m) = table.tt_entry_get(0, 0, -500, 500, &b, false, false);
        // assert!(eval.is_none());
        // assert_eq!(m, Move::NULL);
        //
        // let m = Move::new(Square(12), Square(28), PieceName::Pawn);
        // table.store(b.zobrist_hash, m, 4, EntryFlag::Exact, 25, 0, false);
        // let entry = table.tt_entry_get(b.zobrist_hash, 2);
        // assert_eq!(25, entry.;
        // assert_eq!(m, m1);
    }
}
