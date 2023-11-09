use std::ptr;

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

#[derive(Clone)]
pub struct Magics {
    #[allow(dead_code)]
    rook_table: Vec<Bitboard>,
    rook_magics: [SMagic; 64],
    #[allow(dead_code)]
    bishop_table: Vec<Bitboard>,
    bishop_magics: [SMagic; 64],
}

impl Default for Magics {
    #[cold]
    fn default() -> Self {
        unsafe {
            let mut rook_magics = [SMagic::init(); 64];
            let mut rook_table = Vec::with_capacity(ROOK_M_SIZE);
            gen_magic_board(ROOK_M_SIZE, &R_DELTAS, rook_magics.as_mut_ptr(), rook_table.as_mut_ptr());

            let mut bishop_magics = [SMagic::init(); 64];
            let mut bishop_table = Vec::with_capacity(BISHOP_M_SIZE);
            gen_magic_board(BISHOP_M_SIZE, &B_DELTAS, bishop_magics.as_mut_ptr(), bishop_table.as_mut_ptr());

            Self {
                rook_table,
                rook_magics,
                bishop_table,
                bishop_magics,
            }
        }
    }
}

impl Magics {
    pub fn bishop_attacks(&self, occupied: Bitboard, square: Square) -> Bitboard {
        let mut occupied = occupied.0;
        let magic_entry: &SMagic = &self.bishop_magics[square];
        occupied &= magic_entry.mask;
        occupied = occupied.wrapping_mul(magic_entry.magic);
        occupied = occupied.wrapping_shr(magic_entry.shift);
        unsafe { Bitboard(*(magic_entry.ptr as *const u64).add(occupied as usize)) }
    }

    pub fn rook_attacks(&self, occupied: Bitboard, square: Square) -> Bitboard {
        let mut occupied = occupied.0;
        let magic_entry: &SMagic = &self.rook_magics[square];
        occupied &= magic_entry.mask;
        occupied = occupied.wrapping_mul(magic_entry.magic);
        occupied = occupied.wrapping_shr(magic_entry.shift);
        unsafe { Bitboard(*(magic_entry.ptr as *const u64).add(occupied as usize)) }
    }
}

/// Structure inside a `MagicTable` for a specific hash. For a certain square,
/// contains a mask,  magic number, number to shift by, and a pointer into the array slice
/// where the position is held.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct SMagic {
    ptr: usize,
    mask: u64,
    magic: u64,
    shift: u32,
}

impl SMagic {
    pub const fn init() -> Self {
        SMagic {
            ptr: 0,
            mask: 0,
            magic: 0,
            shift: 0,
        }
    }
}

/// Temporary struct used to create an actual `SMagic` Object.
#[derive(Clone, Copy)]
struct PreSMagic {
    start: usize,
    len: usize,
    mask: u64,
    magic: u64,
    shift: u32,
}

impl PreSMagic {
    pub fn init() -> PreSMagic {
        PreSMagic {
            start: 0,
            len: 0,
            mask: 0,
            magic: 0,
            shift: 0,
        }
    }

    // creates an array of PreSMagic
    pub fn init64() -> [PreSMagic; 64] {
        [PreSMagic::init(); 64]
    }

    // Helper method to compute the next index
    pub fn next_idx(&self) -> usize {
        self.start + self.len
    }
}

/// Creates the `MagicTable` struct. The table size is relative to the piece for computation,
/// and the deltas are the directions on the board the piece can go.
#[cold]
unsafe fn gen_magic_board(
    table_size: usize,
    deltas: &[Direction; 4],
    static_magics: *mut SMagic,
    attacks: *mut Bitboard,
) {
    // Creates PreSMagic to hold raw numbers. Technically just adds room to stack
    let mut pre_sq_table: [PreSMagic; 64] = PreSMagic::init64();

    // Initializes each PreSMagic
    for table in pre_sq_table.iter_mut() {
        *table = PreSMagic::init();
    }

    // Occupancy tracks occupancy permutations. MAX permutations = subset of 12 bits = 2^12
    // Reference is similar, tracks the sliding moves from a given occupancy
    // Age tracks the best index for a current permutation
    let mut occupancy: [u64; 4096] = [0; 4096];
    let mut reference: [u64; 4096] = [0; 4096];
    let mut age: [i32; 4096] = [0; 4096];

    // Size tracks the size of permutations of the current block
    let mut size: usize;

    // b is used for generating the permutations through ripple - carry
    let mut b: u64;

    // current and i is a placeholder for actually generating correct magic numbers
    let mut current: i32 = 0;
    let mut i: usize;

    // set the first PreSMagic start = 0. Just in case.
    pre_sq_table[0].start = 0;

    // Loop through each square! s is a SQ
    for s in Square::iter() {
        // Magic number for later
        let mut magic: u64;

        // edges is the bitboard representation of the edges s is not on.
        // e.g. sq A1 is on FileA and Rank1, so edges = bitboard of FileH and Rank8
        // mask = occupancy mask of square s
        // let edges: u64 = ((RANK1.0 | RANK8.0) & !get_rank_bitboard(s))
        let edges = ((RANK1 | RANK8) & !(s.get_rank_bitboard())) | ((FILE_A | FILE_H) & !(s.get_file_bitboard()));
        let mask = sliding_attack(deltas, s, Bitboard::EMPTY) & !edges;

        // Shift = number of bits in 64 - bits in mask = log2(size)
        let shift: u32 = 64 - mask.0.count_ones();
        b = 0;
        size = 0;

        // Ripple carry to determine occupancy, reference, and size
        'bit: loop {
            occupancy[size] = b;
            reference[size] = sliding_attack(deltas, s, Bitboard(b)).0;
            size += 1;
            b = ((b).wrapping_sub(mask.0)) & mask.0;
            if b == 0 {
                break 'bit;
            }
        }

        // Set current PreSMagic length to be of size
        pre_sq_table[s].len = size;

        // If there is a next square, set the start of it.
        if s.idx() < 63 {
            pre_sq_table[s.idx() + 1].start = pre_sq_table[s].next_idx();
        }
        // Create our Random Number Generator with a seed
        let mut rng = Rng::default();

        // Loop until we have found our magics!
        'outer: loop {
            // Create a magic with our desired number of bits in the first 8 places
            'first_in: loop {
                magic = rng.next_magic();
                if (magic.wrapping_mul(mask.0)).wrapping_shr(56).count_ones() >= 6 {
                    break 'first_in;
                }
            }
            current += 1;
            i = 0;

            // Filling the attacks Vector up to size digits
            while i < size {
                // Magic part! The index is = ((occupancy[s] & mask) * magic >> shift)
                let index: usize = (occupancy[i] & mask.0).wrapping_mul(magic).wrapping_shr(shift) as usize;

                // Checking to see if we have visited this index already with a lower current number
                if age[index] < current {
                    // If we have visited with lower current, we replace it with this current number,
                    // as this current is higher and has gone through more passes
                    age[index] = current;
                    *attacks.add(pre_sq_table[s].start + index) = Bitboard(reference[i]);
                } else if *attacks.add(pre_sq_table[s].start + index) != Bitboard(reference[i]) {
                    // If a magic maps to the same index but different result, either magic is bad or we are done
                    break;
                }
                i += 1;
            }
            // If we have filled it up to size or greater, we are done
            if i >= size {
                break 'outer;
            }
        }
        // Set the remaining variables for the PreSMagic Struct
        pre_sq_table[s].magic = magic;
        pre_sq_table[s].mask = mask.0;
        pre_sq_table[s].shift = shift;
    }

    // size = running total of total size
    let mut size = 0;
    (0..64).for_each(|i| {
        // begin ptr points to the beginning of the current slice in the vector
        let beginptr = attacks.add(size);

        // points to the static entry
        let staticptr: *mut SMagic = static_magics.add(i);
        let table_i: SMagic = SMagic {
            ptr: beginptr as usize,
            mask: pre_sq_table[i].mask,
            magic: pre_sq_table[i].magic,
            shift: pre_sq_table[i].shift,
        };

        ptr::copy::<SMagic>(&table_i, staticptr, 1);

        // Create the pointer to the slice with begin_ptr / length
        size += pre_sq_table[i].len;
    });
    // Sanity check
    assert_eq!(size, table_size);
}

/// Returns a bitboards of sliding attacks given an array of 4 deltas/
/// Does not include the original position/
/// Includes occupied bits if it runs into them, but stops before going further.
fn sliding_attack(deltas: &[Direction; 4], sq: Square, occupied: Bitboard) -> Bitboard {
    assert!(sq.0 < 64);
    let mut attack = Bitboard::EMPTY;
    for delta in deltas.iter().take(4_usize) {
        // let mut s: u8 = ((square as i16) + (*delta as i16)) as u8;
        let mut s = sq.shift(*delta);
        'inner: while s.is_valid() && s.dist(s.shift(delta.opp())) == 1 {
            attack |= Bitboard(1_u64.wrapping_shl(s.0));
            if occupied & Bitboard(1_u64.wrapping_shl(s.0)) != Bitboard::EMPTY {
                break 'inner;
            }
            s = s.shift(*delta);
        }
    }
    attack
}
