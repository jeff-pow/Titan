use std::{
    mem::{size_of, transmute},
    ptr::from_ref,
    sync::atomic::{AtomicI16, AtomicU16, AtomicU64, AtomicU8, Ordering},
};

use crate::{
    board::board::Board, moves::moves::Move, search::search::NEAR_CHECKMATE, types::pieces::Piece,
};

/**
* The transposition table stores a list of previously seen results from searches in the chess
* engine. In the desire to support parallel searches, the table is made thread safe by storing data
* in AtomicU64s. Since each entry takes up two AtomicU64s, write races could occur. This is
* resolved by using a method where the two AtomicU64s that are related to each other are xor-ed
* before being stored or retrieved from the table, and since xor loses no information, if after
* reading from the table the hash from the table entry matches the board hash being looked at, the
* rest of the information in the entry is known to be valid as well.
**/

#[derive(Clone, Copy, Debug, Default)]
#[repr(C)]
/// Storing a 32 bit move in the transposition table is a waste of space, as 16 bits contains all
/// you need. However, 32 bits is nice for extra information such as what piece moved, so moves are
/// truncated before being placed in transposition table, and extracted back into 32 bits before
/// being returned to caller
pub struct TableEntry {
    depth: u8,
    age_pv_bound: u8,
    key: u16,
    search_score: i16,
    best_move: u16,
    static_eval: i16,
}

impl TableEntry {
    pub const fn static_eval(self) -> i32 {
        self.static_eval as i32
    }

    pub const fn key(self) -> u16 {
        self.key
    }

    pub const fn depth(self) -> i32 {
        self.depth as i32
    }

    pub fn flag(self) -> EntryFlag {
        match self.age_pv_bound & 0b11 {
            0 => EntryFlag::None,
            1 => EntryFlag::AlphaUnchanged,
            2 => EntryFlag::BetaCutOff,
            3 => EntryFlag::Exact,
            _ => unreachable!(),
        }
    }

    fn age(self) -> u64 {
        u64::from(self.age_pv_bound) >> 3
    }

    pub fn was_pv(self) -> bool {
        (self.age_pv_bound & 0b0000_0100) != 0
    }

    pub fn search_score(self) -> i32 {
        i32::from(self.search_score)
    }

    pub fn best_move(self, b: &Board) -> Move {
        let m = Move(u32::from(self.best_move));
        if b.piece_at(m.from()) == Piece::None {
            Move::NULL
        } else {
            let p = b.piece_at(m.from()) as u32;
            Move(u32::from(self.best_move) | p << 16)
        }
    }
}

impl From<TableEntry> for InternalEntry {
    fn from(value: TableEntry) -> Self {
        unsafe { transmute(value) }
    }
}

impl From<InternalEntry> for TableEntry {
    fn from(value: InternalEntry) -> Self {
        unsafe { transmute(value) }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum EntryFlag {
    #[default]
    None,
    /// Upper bound on the possible score at a position
    AlphaUnchanged,
    /// Lower bound on the possible score at a position
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
#[repr(C)]
struct InternalEntry {
    depth: AtomicU8,
    age_pv_bound: AtomicU8,
    key: AtomicU16,
    search_score: AtomicI16,
    best_move: AtomicU16,
    static_eval: AtomicI16,
}

impl Clone for InternalEntry {
    fn clone(&self) -> Self {
        Self {
            depth: AtomicU8::new(self.depth.load(Ordering::Relaxed)),
            age_pv_bound: AtomicU8::new(self.age_pv_bound.load(Ordering::Relaxed)),
            key: AtomicU16::new(self.key.load(Ordering::Relaxed)),
            search_score: AtomicI16::new(self.search_score.load(Ordering::Relaxed)),
            best_move: AtomicU16::new(self.best_move.load(Ordering::Relaxed)),
            static_eval: AtomicI16::new(self.static_eval.load(Ordering::Relaxed)),
        }
    }
}

#[derive(Clone)]
pub struct TranspositionTable {
    vec: Box<[InternalEntry]>,
    age: U64Wrapper,
}

pub const TARGET_TABLE_SIZE_MB: usize = 16;
const BYTES_PER_MB: usize = 1024 * 1024;
const ENTRY_SIZE: usize = size_of::<TableEntry>();
const MAX_AGE: u64 = (1 << 5) - 1;

impl TranspositionTable {
    pub fn prefetch(&self, hash: u64) {
        #[cfg(target_arch = "x86_64")]
        use std::arch::x86_64::{_mm_prefetch, _MM_HINT_T0};
        unsafe {
            let index = index(hash, self.vec.len());
            let entry = self.vec.get_unchecked(index);
            _mm_prefetch(from_ref::<InternalEntry>(entry).cast::<i8>(), _MM_HINT_T0);
        }
    }

    pub fn new(mb: usize) -> Self {
        let target_size = mb * BYTES_PER_MB;
        let table_capacity = target_size / ENTRY_SIZE;
        Self {
            vec: vec![InternalEntry::default(); table_capacity].into_boxed_slice(),
            age: U64Wrapper::default(),
        }
    }

    pub fn clear(&self) {
        self.vec.iter().for_each(|x| {
            x.depth.store(0, Ordering::Relaxed);
            x.age_pv_bound.store(0, Ordering::Relaxed);
            x.key.store(0, Ordering::Relaxed);
            x.search_score.store(0, Ordering::Relaxed);
            x.best_move.store(0, Ordering::Relaxed);
            x.static_eval.store(0, Ordering::Relaxed);
        });
        self.age.0.store(0, Ordering::Relaxed);
    }

    fn age(&self) -> u64 {
        self.age.0.load(Ordering::Relaxed)
    }

    pub fn age_up(&self) {
        // Keep age under 31 b/c that is the max age that fits in a table entry
        self.age.0.store((self.age() + 1) & MAX_AGE, Ordering::Relaxed);
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
        let idx = index(hash, self.vec.len());
        let key = hash as u16;

        let old_entry = unsafe { TableEntry::from(self.vec.get_unchecked(idx).clone()) };

        // Conditions from Alexandria
        if old_entry.age() != self.age()
            || old_entry.key() != key
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

            let age_pv_bound = (self.age() << 3) as u8 | u8::from(is_pv) << 2 | flag as u8;
            unsafe {
                self.vec.get_unchecked(idx).key.store(key, Ordering::Relaxed);
                self.vec.get_unchecked(idx).depth.store(depth as u8, Ordering::Relaxed);
                self.vec.get_unchecked(idx).age_pv_bound.store(age_pv_bound, Ordering::Relaxed);
                self.vec.get_unchecked(idx).search_score.store(score as i16, Ordering::Relaxed);
                self.vec.get_unchecked(idx).best_move.store(best_m, Ordering::Relaxed);
                self.vec
                    .get_unchecked(idx)
                    .static_eval
                    .store(static_eval as i16, Ordering::Relaxed);
            }
        }
    }

    pub fn get(&self, hash: u64, ply: i32) -> Option<TableEntry> {
        let idx = index(hash, self.vec.len());
        let key = hash as u16;

        let mut entry = unsafe { TableEntry::from(self.vec.get_unchecked(idx).clone()) };

        if entry.key != key {
            return None;
        }

        if entry.search_score > NEAR_CHECKMATE as i16 {
            entry.search_score -= ply as i16;
        } else if entry.search_score < -NEAR_CHECKMATE as i16 {
            entry.search_score += ply as i16;
        }

        Some(entry)
    }

    pub(crate) fn permille_usage(&self) -> usize {
        self.vec
            .iter()
            .take(1000)
            .map(|e| TableEntry::from(e.clone()))
            // We only consider entries meaningful if their age is current (due to age based overwrites)
            // and their depth is > 0. 0 depth entries are from qsearch and should not be counted.
            .filter(|e| e.depth() > 0 && e.age() == self.age())
            .count()
    }
}

fn index(hash: u64, table_capacity: usize) -> usize {
    ((u128::from(hash) * (table_capacity as u128)) >> 64) as usize
}

#[cfg(test)]
mod transpos_tests {
    use crate::{
        board::fen::{build_board, STARTING_FEN},
        engine::transposition::{EntryFlag, TranspositionTable},
        moves::moves::{Move, MoveType},
        types::{pieces::Piece, square::Square},
    };

    #[test]
    fn transpos_table() {
        let b = build_board(STARTING_FEN);
        let table = TranspositionTable::new(64);
        let entry = table.get(b.zobrist_hash, 4);
        assert!(entry.is_none());

        let m = Move::new(Square(12), Square(28), MoveType::Normal, Piece::WhitePawn);
        table.store(b.zobrist_hash, m, 0, EntryFlag::Exact, 25, 4, false, 25);
        let entry = table.get(b.zobrist_hash, 2);
        assert_eq!(25, entry.unwrap().static_eval());
        assert_eq!(m, entry.unwrap().best_move(&b));
    }
}
