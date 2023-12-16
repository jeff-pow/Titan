use std::{
    fs::File,
    io::Write,
    mem::{size_of, transmute},
};

use crate::{
    moves::attack_boards::{FILE_A, FILE_H, RANK1, RANK8},
    types::{bitboard::Bitboard, square::Square},
};

use super::moves::{Direction, Direction::*};

pub const fn rand_u64(mut prev: u64) -> u64 {
    prev ^= prev << 13;
    prev ^= prev >> 7;
    prev ^= prev << 17;
    prev
}

/// Xorshift64 https://en.wikipedia.org/wiki/Xorshift
#[derive(Copy, Clone)]
pub struct Rng(u64);

impl Default for Rng {
    fn default() -> Self {
        Self(0xE926E6210D9E3487)
    }
}

impl Rng {
    pub fn next_u64(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
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

#[derive(Clone, Copy, Default, Debug)]
struct MagicEntry {
    mask: Bitboard,
    magic: u64,
    shift: u32,
    offset: usize,
}

impl MagicEntry {
    const fn index(&self, occupied: Bitboard) -> usize {
        let blockers = occupied.0 & self.mask.0;
        let hash = blockers.wrapping_mul(self.magic);
        let index = (hash >> self.shift) as usize;
        self.offset + index
        // unsafe { self.offset + _pext_u64(occupied.0, self.mask.0) as usize }
    }
}

// #[derive(Clone)]
// pub struct Magics {
//     pub rook_table: Vec<Bitboard>,
//     rook_magics: [MagicEntry; 64],
//     pub bishop_table: Vec<Bitboard>,
//     bishop_magics: [MagicEntry; 64],
// }
//
// impl Default for Magics {
//     fn default() -> Self {
//         Self {
//             rook_table: Default::default(),
//             rook_magics: [MagicEntry::default(); 64],
//             bishop_table: Default::default(),
//             bishop_magics: [MagicEntry::default(); 64],
//         }
//     }
// }

pub fn bishop_attacks(sq: Square, occupied: Bitboard) -> Bitboard {
    let magic = &BISHOP_MAGICS[sq];
    BISHOP_TABLE[magic.index(occupied)]
}

pub fn rook_attacks(sq: Square, occupied: Bitboard) -> Bitboard {
    let magic = &ROOK_MAGICS[sq];
    ROOK_TABLE[magic.index(occupied)]
}

pub fn queen_attacks(sq: Square, occupied: Bitboard) -> Bitboard {
    bishop_attacks(sq, occupied) | rook_attacks(sq, occupied)
}

/// https://analog-hors.github.io/site/magic-bitboards/
// impl Magics {
//     pub fn bishop_attacks(&self, occupied: Bitboard, sq: Square) -> Bitboard {
//         let magic = &self.bishop_magics[sq];
//         self.bishop_table[index(magic, occupied)]
//     }
//
//     pub fn rook_attacks(&self, occupied: Bitboard, sq: Square) -> Bitboard {
//         let magic = &self.rook_magics[sq];
//         self.rook_table[index(magic, occupied)]
//     }

// pub fn new(bishop_magics: [MagicEntry; 64], rook_magics: [MagicEntry; 64]) -> Self {
//     let mut rook_table = Vec::with_capacity(ROOK_M_SIZE);
//     let mut bishop_table = Vec::with_capacity(BISHOP_M_SIZE);
//
//     for sq in Square::iter() {
//         let mut table = create_table(sq, R_DELTAS);
//         rook_table.append(&mut table);
//
//         let mut table = create_table(sq, B_DELTAS);
//         bishop_table.append(&mut table);
//     }
//
//     assert_eq!(ROOK_M_SIZE, rook_table.len());
//     assert_eq!(BISHOP_M_SIZE, bishop_table.len());
//
//     Self { rook_table, rook_magics, bishop_table, bishop_magics }
// }
// }

// pub const BISHOP_TABLE: &[Bitboard; BISHOP_M_SIZE] = &table::<BISHOP_M_SIZE, false>();
// pub const ROOK_TABLE: &[Bitboard; ROOK_M_SIZE] = &table::<ROOK_M_SIZE, true>();
// pub const ROOK_TABLE: &[Bitboard; ROOK_M_SIZE] = &[Bitboard::EMPTY; ROOK_M_SIZE];

// pub const fn table<const T: usize, const IS_ROOK: bool>() -> [Bitboard; T] {
//     let mut a = [Bitboard::EMPTY; T];
//
//     let deltas = if IS_ROOK { R_DELTAS } else { B_DELTAS };
//     let magics = if IS_ROOK { ROOK_MAGICS } else { BISHOP_MAGICS };
//
//     let mut sq = 0;
//     while sq < 64 {
//         let magic_entry = magics[sq];
//         let mut blockers = Bitboard::EMPTY;
//         loop {
//             let moves = sliding_attack(deltas, Square(sq as u32), blockers);
//             let idx = index(&magic_entry, blockers);
//
//             a[idx] = moves;
//
//             // Carry-Rippler trick to iterate through all subsections of blockers
//             blockers.0 = blockers.0.wrapping_sub(magic_entry.mask.0) & magic_entry.mask.0;
//             if blockers.0 == 0 {
//                 break;
//             }
//         }
//         sq += 1;
//     }
//     a
// }

/// Extracts move bitboards using known constants
// fn create_table(sq: Square, deltas: [Direction; 4], magics: &[Magi]) -> Vec<Bitboard> {
//     let magic_entry =
//         if deltas[0] == North {  } else { BISHOP_MAGICS[sq] };
//     let idx_bits = 64 - magic_entry.shift;
//     let mut table = vec![Bitboard::EMPTY; 1 << idx_bits];
//     let mut blockers = Bitboard::EMPTY;
//     loop {
//         let moves = sliding_attack(deltas, sq, blockers);
//         let idx = index(&magic_entry, blockers) - magic_entry.offset;
//
//         table[idx] = moves;
//
//         // Carry-Rippler trick to iterate through all subsections of blockers
//         blockers.0 = blockers.0.wrapping_sub(magic_entry.mask.0) & magic_entry.mask.0;
//         if blockers == Bitboard::EMPTY {
//             break;
//         }
//     }
//     table
// }

/// Returns a bitboards of sliding attacks given an array of 4 deltas/
/// Does not include the original position/
/// Includes occupied bits if it runs into them, but stops before going further.
const fn sliding_attack(deltas: [Direction; 4], sq: Square, occupied: Bitboard) -> Bitboard {
    let mut attack = 0;
    let mut count = 0;
    while count < deltas.len() {
        let dir = deltas[count];
        let mut s = sq.shift(dir);
        'inner: while s.is_valid() && s.dist(s.shift(dir.opp())) == 1 {
            attack |= 1 << s.0 as usize;
            if occupied.0 & 1 << s.0 as usize != 0 {
                break 'inner;
            }
            s = s.shift(dir);
        }
        count += 1;
    }
    Bitboard(attack)
}

#[allow(dead_code)]
/// Function generates magic numbers when they are not known.
pub fn gen_magics() {
    let mut rng = Rng::default();
    let mut rook_table = Vec::with_capacity(ROOK_M_SIZE);
    let mut rook_magics = [MagicEntry::default(); 64];
    let mut bishop_table = Vec::with_capacity(BISHOP_M_SIZE);
    let mut bishop_magics = [MagicEntry::default(); 64];

    for sq in Square::iter() {
        let edges = ((RANK1 | RANK8) & !(sq.get_rank_bitboard()))
            | ((FILE_A | FILE_H) & !(sq.get_file_bitboard()));

        let rook_bits = sliding_attack(R_DELTAS, sq, Bitboard::EMPTY);
        let mask = rook_bits & !edges;
        let (mut entry, mut table) = find_magic(mask, sq, R_DELTAS, &mut rng);
        entry.offset = rook_table.len();
        rook_magics[sq] = entry;
        rook_table.append(&mut table);

        let bishop_bits = sliding_attack(B_DELTAS, sq, Bitboard::EMPTY);
        let mask = bishop_bits & !edges;
        let (mut entry, mut table) = find_magic(mask, sq, B_DELTAS, &mut rng);
        entry.offset = bishop_table.len();
        bishop_magics[sq] = entry;
        bishop_table.append(&mut table);
    }

    // println!("#[rustfmt::skip]");
    // println!("pub const BISHOP_MAGICS: &[MagicEntry; 64] = &[",);
    // for entry in bishop_magics {
    //     println!(
    //         "    MagicEntry {{ mask: Bitboard(0x{:016X}), magic: 0x{:016X}, shift: {}, offset: {} }},",
    //         entry.mask.0, entry.magic, entry.shift, entry.offset
    //     );
    // }
    // println!("];");
    //
    // println!("#[rustfmt::skip]");
    // println!("pub const ROOK_MAGICS: &[MagicEntry; 64] = &[",);
    // for entry in rook_magics {
    //     println!(
    //         "    MagicEntry {{ mask: Bitboard(0x{:016X}), magic: 0x{:016X}, shift: {}, offset: {} }},",
    //         entry.mask.0, entry.magic, entry.shift, entry.offset
    //     );
    // }
    // println!("];");

    assert_eq!(ROOK_M_SIZE, rook_table.len());
    assert_eq!(BISHOP_M_SIZE, bishop_table.len());
    // let magics = Magics { rook_table, rook_magics, bishop_table, bishop_magics };
    write_bin("./bins/rook_magics.bin", &rook_magics, size_of::<[MagicEntry; 64]>());
    write_bin("./bins/rook_table.bin", &rook_table, size_of::<[Bitboard; ROOK_M_SIZE]>());
    write_bin("./bins/bishop_table.bin", &bishop_table, size_of::<[Bitboard; BISHOP_M_SIZE]>());
    write_bin("./bins/bishop_magics.bin", &bishop_magics, size_of::<[MagicEntry; 64]>());
}

const ROOK_TABLE: [Bitboard; ROOK_M_SIZE] =
    unsafe { transmute(*include_bytes!("../../bins/rook_table.bin")) };
const ROOK_MAGICS: [MagicEntry; 64] =
    unsafe { transmute(*include_bytes!("../../bins/rook_magics.bin")) };
const BISHOP_TABLE: [Bitboard; BISHOP_M_SIZE] =
    unsafe { transmute(*include_bytes!("../../bins/bishop_table.bin")) };
const BISHOP_MAGICS: [MagicEntry; 64] =
    unsafe { transmute(*include_bytes!("../../bins/bishop_magics.bin")) };

pub fn write_bin<T>(file: &str, data: &[T], size: usize) {
    let mut file = File::create(file).unwrap();
    let buf = unsafe { std::slice::from_raw_parts(data.as_ptr().cast::<u8>(), size) };
    let _ = file.write_all(buf);
}

/// Function finds a magic valid for a given square
fn find_magic(
    mask: Bitboard,
    sq: Square,
    deltas: [Direction; 4],
    rng: &mut Rng,
) -> (MagicEntry, Vec<Bitboard>) {
    loop {
        let mut magic;
        loop {
            magic = rng.next_magic();
            if (magic.wrapping_mul(mask.0)).wrapping_shr(56).count_ones() >= 6 {
                break;
            }
        }

        let shift = 64 - mask.count_bits();
        let magic_entry = MagicEntry { mask, magic, shift, offset: 0 };
        if let Some(table) = make_table(deltas, sq, &magic_entry) {
            return (magic_entry, table);
        }
    }
}

/// Function tries to make a table with a given magic number
fn make_table(
    deltas: [Direction; 4],
    sq: Square,
    magic_entry: &MagicEntry,
) -> Option<Vec<Bitboard>> {
    let idx_bits = 64 - magic_entry.shift;
    let mut table = vec![Bitboard::EMPTY; 1 << idx_bits];
    let mut blockers = Bitboard::EMPTY;
    loop {
        let moves = sliding_attack(deltas, sq, blockers);
        let idx = magic_entry.index(blockers);

        if table[idx] == Bitboard::EMPTY {
            table[idx] = moves;
        } else if table[idx] != moves {
            return None;
        }

        // Carry-Rippler trick to iterate through all subsections of blockers
        blockers.0 = blockers.0.wrapping_sub(magic_entry.mask.0) & magic_entry.mask.0;
        if blockers == Bitboard::EMPTY {
            break;
        }
    }
    Some(table)
}
