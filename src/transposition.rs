use crate::{chess_move::Move, search::search::Score};
use std::{
    mem::size_of,
    num::NonZeroU16,
    sync::atomic::{AtomicI16, AtomicU16, AtomicU64, AtomicU8, Ordering},
};

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

#[derive(Clone, Copy, Debug)]
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
    raw_eval: i16,
}

impl TableEntry {
    pub const EMPTY: Self = Self {
        depth: 0,
        age_pv_bound: 0,
        key: 0,
        search_score: Score::NONE as i16,
        best_move: 0,
        raw_eval: Score::NONE as i16,
    };

    /// An entry is unoccupied while it still holds the initial `EMPTY` value: no key and no score.
    pub const fn is_unoccupied(self) -> bool {
        self.key == 0 && self.search_score == Score::NONE as i16
    }

    pub fn raw_eval(self) -> Option<i32> {
        (i32::from(self.raw_eval) != Score::NONE).then_some(i32::from(self.raw_eval))
    }

    pub fn search_score(self) -> Option<i32> {
        (i32::from(self.search_score) != Score::NONE).then_some(i32::from(self.search_score))
    }

    const fn key(self) -> u16 {
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

    #[expect(unused)]
    pub const fn was_pv(self) -> bool {
        (self.age_pv_bound & 0b0000_0100) != 0
    }

    pub fn best_move(self) -> Option<Move> {
        (self.best_move != 0).then(|| Move(NonZeroU16::new(self.best_move).unwrap()))
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn from_storage(mut self, ply: usize) -> Self {
        if let Some(score) = self.search_score() {
            self.search_score = if Score::is_win(score) {
                (score - ply as i32) as i16
            } else if Score::is_loss(score) {
                (score + ply as i32) as i16
            } else {
                score as i16
            };
        }
        self
    }

    pub fn to_storage(mut self, ply: usize) -> Self {
        if let Some(score) = self.search_score() {
            self.search_score = if Score::is_win(score) {
                (score + ply as i32) as i16
            } else if Score::is_loss(score) {
                (score - ply as i32) as i16
            } else {
                score as i16
            }
        }
        self
    }
}

#[derive(Default)]
struct U64Wrapper(AtomicU64);
impl Clone for U64Wrapper {
    fn clone(&self) -> Self {
        Self(AtomicU64::new(self.0.load(Ordering::Relaxed)))
    }
}

#[repr(C)]
struct InternalEntry {
    depth: AtomicU8,
    age_pv_bound: AtomicU8,
    key: AtomicU16,
    search_score: AtomicI16,
    best_move: AtomicU16,
    raw_eval: AtomicI16,
}

impl Default for InternalEntry {
    fn default() -> Self {
        Self {
            depth: AtomicU8::new(0),
            age_pv_bound: AtomicU8::new(0),
            key: AtomicU16::new(0),
            search_score: AtomicI16::new(Score::NONE as i16),
            best_move: AtomicU16::new(0),
            raw_eval: AtomicI16::new(Score::NONE as i16),
        }
    }
}

impl Clone for InternalEntry {
    fn clone(&self) -> Self {
        Self {
            depth: AtomicU8::new(self.depth.load(Ordering::Relaxed)),
            age_pv_bound: AtomicU8::new(self.age_pv_bound.load(Ordering::Relaxed)),
            key: AtomicU16::new(self.key.load(Ordering::Relaxed)),
            search_score: AtomicI16::new(self.search_score.load(Ordering::Relaxed)),
            best_move: AtomicU16::new(self.best_move.load(Ordering::Relaxed)),
            raw_eval: AtomicI16::new(self.raw_eval.load(Ordering::Relaxed)),
        }
    }
}

impl InternalEntry {
    fn read(&self) -> TableEntry {
        TableEntry {
            depth: self.depth.load(Ordering::Relaxed),
            age_pv_bound: self.age_pv_bound.load(Ordering::Relaxed),
            key: self.key.load(Ordering::Relaxed),
            search_score: self.search_score.load(Ordering::Relaxed),
            best_move: self.best_move.load(Ordering::Relaxed),
            raw_eval: self.raw_eval.load(Ordering::Relaxed),
        }
    }

    fn write(&self, entry: TableEntry) {
        self.depth.store(entry.depth, Ordering::Relaxed);
        self.age_pv_bound.store(entry.age_pv_bound, Ordering::Relaxed);
        self.key.store(entry.key, Ordering::Relaxed);
        self.search_score.store(entry.search_score, Ordering::Relaxed);
        self.best_move.store(entry.best_move, Ordering::Relaxed);
        self.raw_eval.store(entry.raw_eval, Ordering::Relaxed);
    }

    fn depth(&self) -> i32 {
        i32::from(self.depth.load(Ordering::Relaxed))
    }

    fn age(&self) -> u64 {
        u64::from(self.age_pv_bound.load(Ordering::Relaxed)) >> 3
    }
}

#[repr(C, align(32))]
#[derive(Clone)]
struct InternalTTBucket {
    data: [InternalEntry; 3],
}

impl Default for InternalTTBucket {
    fn default() -> Self {
        Self { data: std::array::from_fn(|_| InternalEntry::default()) }
    }
}

#[derive(Clone)]
pub struct TranspositionTable {
    vec: Box<[InternalTTBucket]>,
    age: U64Wrapper,
}

pub const TARGET_TABLE_SIZE_MB: usize = 16;
const BYTES_PER_MB: usize = 1024 * 1024;
const ENTRY_SIZE: usize = size_of::<InternalTTBucket>();
const MAX_AGE: u64 = (1 << 5) - 1;

impl TranspositionTable {
    pub fn prefetch(&self, hash: u64) {
        #[cfg(target_arch = "x86_64")]
        use std::arch::x86_64::{_mm_prefetch, _MM_HINT_T0};
        unsafe {
            let index = index(hash, self.vec.len());
            let entry = self.vec.get_unchecked(index);
            _mm_prefetch::<_MM_HINT_T0>(std::ptr::from_ref::<InternalTTBucket>(entry).cast());
        }
    }

    pub fn new(mb: usize) -> Self {
        let target_size = mb * BYTES_PER_MB;
        let table_capacity = (target_size / ENTRY_SIZE).max(1);
        Self { vec: vec![InternalTTBucket::default(); table_capacity].into_boxed_slice(), age: U64Wrapper::default() }
    }

    pub fn clear(&self) {
        self.vec.iter().for_each(|bucket| {
            for entry in &bucket.data {
                entry.write(TableEntry::EMPTY);
            }
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
        m: Option<Move>,
        depth: i32,
        flag: EntryFlag,
        search_score: i32,
        ply: usize,
        is_pv: bool,
        raw_eval: i32,
    ) {
        let idx = index(hash, self.vec.len());
        let key = hash as u16;
        let age = self.age();
        let age_pv_bound = (age << 3) as u8 | u8::from(is_pv) << 2 | flag as u8;

        let bucket = unsafe { self.vec.get_unchecked(idx) };
        let data = [bucket.data[0].read(), bucket.data[1].read(), bucket.data[2].read()];

        // First, try to find unoccupied entry. Next, prefer a slot for the same position, and finally, least valuable entry.
        let victim = data
            .iter()
            .position(|e| e.is_unoccupied())
            .or_else(|| data.iter().position(|e| e.key() == key))
            .unwrap_or_else(|| {
                data.iter()
                    .enumerate()
                    .min_by_key(|(_, e)| e.depth() - 2 * relative_age(e.age(), age))
                    .map(|(i, _)| i)
                    .unwrap()
            });

        let old = data[victim];

        // Overwrite less valuable entries: a same-position entry from the current search is only
        // replaced by an exact entry or by something at least as deep.
        if old.age() == age
            && old.key() == key
            && flag != EntryFlag::Exact
            && depth.max(0) + 5 + 2 * i32::from(is_pv) <= old.depth()
        {
            return;
        }

        // Don't overwrite a best move with a null move
        let best_move = if m.is_none() && old.key() == key { old.best_move } else { m.map_or(0, u16::from) };

        bucket.data[victim].write(
            TableEntry {
                depth: depth.max(0) as u8,
                age_pv_bound,
                key,
                search_score: search_score as i16,
                best_move,
                raw_eval: raw_eval as i16,
            }
            .to_storage(ply),
        );
    }

    pub fn get(&self, hash: u64, ply: usize) -> Option<TableEntry> {
        let idx = index(hash, self.vec.len());
        let key = hash as u16;

        let bucket = unsafe { self.vec.get_unchecked(idx) };
        for internal in &bucket.data {
            let entry = internal.read();
            if entry.key() != key {
                continue;
            }
            if !entry.is_unoccupied() {
                return Some(entry.from_storage(ply));
            }
        }
        None
    }

    pub(crate) fn permille_usage(&self) -> usize {
        let mut used = 0;
        for bucket in self.vec.iter().take(1000) {
            // We only consider entries meaningful if their age is current
            // and their depth is > 0. 0 depth entries are from qsearch and should not
            // be counted.
            used += bucket.data.iter().filter(|e| e.depth() > 0 && e.age() == self.age()).count();
        }
        // 1000 sampled buckets hold 3000 entries. Normalize to permille.
        used / 3
    }
}

fn index(hash: u64, table_capacity: usize) -> usize {
    ((u128::from(hash) * (table_capacity as u128)) >> 64) as usize
}

/// Generations since `entry_age`; 0 means the entry was stored by the current search.
const fn relative_age(entry_age: u64, current_age: u64) -> i32 {
    ((current_age + MAX_AGE + 1 - entry_age) & MAX_AGE) as i32
}

#[cfg(test)]
mod transpos_tests {
    use crate::{
        board::{Board, STARTING_FEN},
        chess_move::{Move, MoveType},
        search::search::Score,
        transposition::{EntryFlag, TranspositionTable},
        types::square::Square,
    };

    #[test]
    fn transpos_table() {
        let b = Board::from_fen(STARTING_FEN);
        let table = TranspositionTable::new(64);
        let entry = table.get(b.hash(), 4);
        assert!(entry.is_none());

        let m = Move::new(Square(12), Square(28), MoveType::Normal);
        table.store(b.hash(), Some(m), 0, EntryFlag::Exact, 25, 4, false, 25);
        let entry = table.get(b.hash(), 2);
        assert_eq!(25, entry.unwrap().raw_eval().unwrap());
        assert_eq!(m, entry.unwrap().best_move().unwrap());
    }

    #[test]
    fn search_scores() {
        let m = Move::new(Square(12), Square(28), MoveType::Normal);
        let table = TranspositionTable::new(64);

        let search_score = 37;
        table.store(0, Some(m), 0, EntryFlag::Exact, search_score, 4, false, 25);
        let entry = table.get(0, 2);
        assert_eq!(search_score, entry.unwrap().search_score().unwrap());

        table.clear();
        let ply = 15;
        let mated_score = -Score::CHECKMATE + ply as i32;
        table.store(0, Some(m), 0, EntryFlag::Exact, mated_score, ply, false, 25);
        let entry = table.get(0, 2);
        assert_eq!(-Score::CHECKMATE + 2, entry.unwrap().search_score().unwrap());

        table.clear();
        let ply = 12;
        let found_mate = Score::CHECKMATE - ply as i32;
        table.store(0, Some(m), 0, EntryFlag::Exact, found_mate, ply, false, 25);
        let entry = table.get(0, 4);
        assert_eq!(Score::CHECKMATE - 4, entry.unwrap().search_score().unwrap());
    }

    /// Hashes that all map to the same bucket of a 1 MiB table (index = hash >> 49).
    fn same_bucket(i: u64) -> u64 {
        0x1000_0000_0000_2000 + i
    }

    #[test]
    fn bucket_retention() {
        let table = TranspositionTable::new(1);
        let m = Move::new(Square(12), Square(28), MoveType::Normal);

        // Three different positions hashing to the same bucket must all be stored.
        for (i, depth) in [10, 5, 3].into_iter().enumerate() {
            let d = depth;
            table.store(same_bucket(i as u64), Some(m), d, EntryFlag::AlphaUnchanged, d, 0, false, d);
        }
        for (i, depth) in [10, 5, 3].into_iter().enumerate() {
            assert_eq!(depth, table.get(same_bucket(i as u64), 0).unwrap().depth());
        }

        // A fourth store must evict the shallowest entry, not the first slot.
        table.store(same_bucket(3), Some(m), 12, EntryFlag::AlphaUnchanged, 12, 0, false, 12);
        assert_eq!(12, table.get(same_bucket(3), 0).unwrap().depth());
        assert_eq!(10, table.get(same_bucket(0), 0).unwrap().depth());
        assert_eq!(5, table.get(same_bucket(1), 0).unwrap().depth());
        assert!(table.get(same_bucket(2), 0).is_none());
    }
}
