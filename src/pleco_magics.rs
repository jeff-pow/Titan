use std::{mem, ptr};

use rand::thread_rng;

use crate::{
    attack_boards::{FILE_A, FILE_H, RANK1, RANK8},
    bit_hacks::{dist, distance, get_file_bitboard, get_rank_bitboard},
    moves::{
        coordinates,
        Direction::{self, *},
    },
};

#[derive(Clone, Copy)]
pub struct MagicEntry {
    pub mask: u64,
    pub magic: u64,
    pub shift: u8,
    pub offset: u32,
}

impl MagicEntry {
    const fn new() -> Self {
        MagicEntry {
            mask: 0,
            magic: 0,
            shift: 0,
            offset: 0,
        }
    }

    const fn init64() -> [MagicEntry; 64] {
        [MagicEntry::new(); 64]
    }
}

pub const ROOK_TABLE_SIZE: usize = 102400;
const ROOK_MOVEMENT_DELTAS: [Direction; 4] = [North, South, East, West];
static mut ROOK_MAGICS: [MagicEntry; 64] = MagicEntry::init64();
static mut ROOK_MOVES: [u64; ROOK_TABLE_SIZE] = [0; ROOK_TABLE_SIZE];

pub const BISHOP_TABLE_SIZE: usize = 5248;
const BISHOP_MOVEMENT_DELTAS: [Direction; 4] = [NorthWest, NorthEast, SouthEast, SouthWest];
static mut BISHOP_MAGICS: [MagicEntry; 64] = MagicEntry::init64();
static mut BISHOP_MOVES: [u64; BISHOP_TABLE_SIZE] = [0; BISHOP_TABLE_SIZE];

pub fn magic_index(entry: &MagicEntry, blockers: u64) -> usize {
    let blockers = blockers & entry.mask;
    let hash = blockers.wrapping_mul(entry.magic);
    let index = (hash >> entry.shift) as usize;
    entry.offset as usize + index
}

pub fn rook_attacks(square: usize, blockers: u64) -> u64 {
    unsafe {
        let magic = &ROOK_MAGICS[square];
        ROOK_MOVES[magic_index(magic, blockers)]
    }
}

pub fn bishop_attacks(square: usize, blockers: u64) -> u64 {
    unsafe {
        let magic = &BISHOP_MAGICS[square];
        BISHOP_MOVES[magic_index(magic, blockers)]
    }
}

pub fn gen_magics() {
    unsafe {
        gen_magic_board(
            BISHOP_TABLE_SIZE,
            &BISHOP_MOVEMENT_DELTAS,
            BISHOP_MOVES.as_mut_ptr(),
            BISHOP_MAGICS.as_mut_ptr(),
        );
        println!("Done with bishops");
        gen_magic_board(
            ROOK_TABLE_SIZE,
            &ROOK_MOVEMENT_DELTAS,
            ROOK_MOVES.as_mut_ptr(),
            ROOK_MAGICS.as_mut_ptr(),
        );
        println!("Done with rooks");
    };
}

#[derive(Clone, Copy)]
struct PreSMagic {
    start: usize,
    len: usize,
    mask: u64,
    magic: u64,
    shift: u32,
}

impl PreSMagic {
    pub const fn init() -> PreSMagic {
        PreSMagic {
            start: 0,
            len: 0,
            mask: 0,
            magic: 0,
            shift: 0,
        }
    }

    // creates an array of PreSMagic
    pub unsafe fn init64() -> [PreSMagic; 64] {
        [PreSMagic::init(); 64]
    }

    // Helper method to compute the next index
    pub fn next_idx(&self) -> usize {
        self.start + self.len
    }
}

// Simple Pcg64Mcg implementation
pub struct Rng(u128);

impl Default for Rng {
    fn default() -> Self {
        Self(0xE926E6210D9E3486 | 1)
    }
}

impl Rng {
    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_mul(0x2360ED051FC65DA44385DF649FCCF645);
        let rot = (self.0 >> 122) as u32;
        let xsl = (self.0 >> 64) as u64 ^ self.0 as u64;
        xsl.rotate_right(rot)
    }

    /// Method returns u64s with an average of 8-12 bits active, the desirable range for magic numbers
    fn next_magic(&mut self) -> u64 {
        self.next_u64() & self.next_u64() & self.next_u64()
    }
}

unsafe fn gen_magic_board(
    table_size: usize,
    deltas: &[Direction; 4],
    attacks: *mut u64,
    static_magics: *mut MagicEntry,
) {
    let mut pre_sq_table = PreSMagic::init64();

    let mut occupancy = [0u64; 4096];
    let mut reference = [0u64; 4096];
    let mut age = [0i32; 4096];

    let mut size;

    let mut b;

    let mut current = 0;
    let mut i: usize;

    let mut rng = Rng::default();

    for s in 0..64_u8 {
        let mut magic: u64;
        let edges =
            ((RANK1 | RANK8) & !get_rank_bitboard(s)) | ((FILE_A | FILE_H) & !get_file_bitboard(s));
        let mask = sliding_attack(deltas, s, 0) & !edges;
        let shift = 64 - mask.count_ones();
        b = 0;
        size = 0;

        loop {
            occupancy[size] = b;
            reference[size] = sliding_attack(deltas, s, b);
            size += 1;
            b = b.wrapping_sub(mask) & mask;
            if b == 0 {
                break;
            }
        }

        pre_sq_table[s as usize].len = size;

        if s < 63 {
            pre_sq_table[s as usize + 1].start = pre_sq_table[s as usize].next_idx();
        }

        loop {
            loop {
                magic = rng.next_magic();
                let i = magic.wrapping_mul(mask).wrapping_shr(56);
                if i.count_ones() >= 6 {
                    break;
                }
            }
            current += 1;
            i = 0;

            while i < size {
                let index = (occupancy[i] & mask)
                    .wrapping_mul(magic)
                    .wrapping_shr(shift) as usize;

                if age[index] < current {
                    age[index] = current;
                    *attacks.add(pre_sq_table[s as usize].start + index) = reference[i];
                } else if *attacks.add(pre_sq_table[s as usize].start + index) != reference[i] {
                    break;
                }
                i += 1;
            }
            if i >= size {
                break;
            }
        }

        pre_sq_table[s as usize].magic = magic;
        pre_sq_table[s as usize].mask = mask;
        pre_sq_table[s as usize].shift = shift;
    }
    let mut size = 0;
    for i in 0..64 {
        let beginptr = attacks.add(size);

        let staticptr = static_magics.add(i);
        let table_i = MagicEntry {
            mask: pre_sq_table[i].mask,
            magic: pre_sq_table[i].magic,
            shift: pre_sq_table[i].shift as u8,
            offset: beginptr as u32,
        };
        ptr::copy::<MagicEntry>(&table_i, staticptr, 1);
        size += pre_sq_table.len();
    }
    assert_eq!(size, table_size);
}

fn sliding_attack(deltas: &[Direction; 4], square: u8, occupied: u64) -> u64 {
    debug_assert!(square < 64);
    let mut attack = 0;
    for dir in deltas.iter() {
        let mut s = (square as i8 + *dir as i8) as u8;
        while s < 64 && distance(s, ((s as i8) - (*dir as i8)) as u8) == 1 {
            attack |= 1_u64.wrapping_shl(s as u32);
            if occupied & 1u64.wrapping_shl(s as u32) != 0 {
                break;
            }
            s = ((s as i8) + *dir as i8) as u8;
        }
    }
    attack
}