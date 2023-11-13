use crate::{
    moves::attack_boards::{FILE_A, FILE_H, RANK1, RANK8},
    types::{bitboard::Bitboard, square::Square},
};

use super::moves::{Direction, Direction::*};

/// Credit for this magic bitboards implementation goes to the rustic chess engine by mvanthoor
/// https://github.com/mvanthoor/rustic/

/// Simple Pcg64Mcg implementation
// No repetitions as 100B iterations
pub struct Rng(u64);

impl Default for Rng {
    fn default() -> Self {
        Self(0xE926E6210D9E3487)
    }
}

impl Rng {
    pub fn next_u64(&mut self) -> u64 {
        self.0 ^= self.0 << 21;
        self.0 ^= self.0 >> 35;
        self.0 ^= self.0 << 4;
        self.0
    }

    /// Method returns u64s with an average of 8 bits active, the desirable range for magic numbers
    pub fn next_magic(&mut self) -> u64 {
        self.next_u64() & self.next_u64() & self.next_u64()
    }
}

/// Size of the magic rook table.
pub const ROOK_M_SIZE: usize = 102_400;
const R_DELTAS: [Direction; 4] = [North, South, East, West];

/// Size of the magic bishop table.
pub const BISHOP_M_SIZE: usize = 5248;
const B_DELTAS: [Direction; 4] = [SouthEast, SouthWest, NorthEast, NorthWest];

#[derive(Clone, Copy, Default)]
struct MagicEntry {
    mask: Bitboard,
    magic: u64,
    shift: u8,
    offset: usize,
}

#[derive(Clone)]
pub struct Magics {
    rook_table: Vec<Bitboard>,
    rook_magics: [MagicEntry; 64],
    bishop_table: Vec<Bitboard>,
    bishop_magics: [MagicEntry; 64],
}

fn index(entry: &MagicEntry, occupied: Bitboard) -> usize {
    let blockers = occupied & entry.mask;
    let hash = blockers.0.wrapping_mul(entry.magic);
    let index = (hash >> entry.shift) as usize;
    entry.offset + index
}

impl Magics {
    pub fn bishop_attacks(&self, occupied: Bitboard, sq: Square) -> Bitboard {
        let magic = &self.bishop_magics[sq];
        self.bishop_table[index(magic, occupied)]
    }

    pub fn rook_attacks(&self, occupied: Bitboard, sq: Square) -> Bitboard {
        let magic = &self.rook_magics[sq];
        self.rook_table[index(magic, occupied)]
    }
}

impl Default for Magics {
    fn default() -> Self {
        let mut rng = Rng::default();
        let mut rook_table = Vec::with_capacity(ROOK_M_SIZE);
        let mut rook_magics = [MagicEntry::default(); 64];
        let mut bishop_table = Vec::with_capacity(BISHOP_M_SIZE);
        let mut bishop_magics = [MagicEntry::default(); 64];

        for sq in Square::iter() {
            let rook_bits = blockers(R_DELTAS, sq).count_bits() as u8;
            let (entry, mut table) = find_magic(sq, rook_bits, R_DELTAS, &mut rng, rook_table.len());
            rook_magics[sq] = entry;
            rook_table.append(&mut table);

            let bishop_bits = blockers(B_DELTAS, sq).count_bits() as u8;
            let (entry, mut table) = find_magic(sq, bishop_bits, B_DELTAS, &mut rng, bishop_table.len());
            bishop_magics[sq] = entry;
            bishop_table.append(&mut table);
        }

        assert_eq!(ROOK_M_SIZE, rook_table.len());
        assert_eq!(BISHOP_M_SIZE, bishop_table.len());

        Self {
            rook_table,
            rook_magics,
            bishop_table,
            bishop_magics,
        }
    }
}

fn find_magic(
    sq: Square,
    idx_bits: u8,
    deltas: [Direction; 4],
    rng: &mut Rng,
    offset: usize,
) -> (MagicEntry, Vec<Bitboard>) {
    let edges = ((RANK1 | RANK8) & !(sq.get_rank_bitboard())) | ((FILE_A | FILE_H) & !(sq.get_file_bitboard()));
    let mask = blockers(deltas, sq) & !edges;
    loop {
        let magic = rng.next_magic();
        let shift = 64 - idx_bits;
        let magic_entry = MagicEntry {
            mask,
            magic,
            shift,
            offset,
        };
        if let Some(table) = make_table(deltas, sq, &magic_entry) {
            return (magic_entry, table);
        }
    }
}

fn make_table(deltas: [Direction; 4], sq: Square, magic_entry: &MagicEntry) -> Option<Vec<Bitboard>> {
    let idx_bits = 64 - magic_entry.shift;
    let mut table = vec![Bitboard::EMPTY; 1 << idx_bits];
    let mut blockers = Bitboard::EMPTY;
    loop {
        let moves = sliding_attack(deltas, sq, blockers);
        let table_entry = &mut table[index(magic_entry, blockers)];
        if *table_entry == Bitboard::EMPTY {
            *table_entry = moves;
        } else if *table_entry != moves {
            return None;
        }

        blockers.0 = blockers.0.wrapping_sub(magic_entry.mask.0) & magic_entry.mask.0;
        if blockers == Bitboard::EMPTY {
            break;
        }
    }
    Some(table)
}

/// Returns a bitboards of sliding attacks given an array of 4 deltas/
/// Does not include the original position/
/// Includes occupied bits if it runs into them, but stops before going further.
fn sliding_attack(deltas: [Direction; 4], sq: Square, occupied: Bitboard) -> Bitboard {
    let mut attack = Bitboard::EMPTY;
    for dir in deltas {
        let mut s = sq;
        while occupied & s.bitboard() == Bitboard::EMPTY {
            if let Some(shifted) = s.checked_shift(dir) {
                attack |= s.bitboard();
                s = shifted;
            } else {
                break;
            }
        }
    }
    attack
}

fn blockers(deltas: [Direction; 4], sq: Square) -> Bitboard {
    let mut blockers = Bitboard::EMPTY;
    for dir in deltas {
        let mut s = sq;
        while let Some(shifted) = s.checked_shift(dir) {
            blockers |= s.bitboard();
            s = shifted;
        }
    }
    blockers
}
