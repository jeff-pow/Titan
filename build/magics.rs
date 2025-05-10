use std::env;
use std::io::Write;
use std::path::Path;
use std::{fs::File, io::BufWriter};
use Direction::{East, North, NorthEast, NorthWest, South, SouthEast, SouthWest, West};

/// Cardinal directions from the point of view of white side
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Direction {
    North = 8,
    NorthWest = 7,
    West = -1,
    SouthWest = -9,
    South = -8,
    SouthEast = -7,
    East = 1,
    NorthEast = 9,
}

const R_DELTAS: [Direction; 4] = [North, South, East, West];
const B_DELTAS: [Direction; 4] = [SouthEast, SouthWest, NorthEast, NorthWest];

pub fn generate_magics() {
    const SEED: u64 = 18165708672197979913;
    let out_dir = env::var("OUT_DIR").unwrap();
    let magics_path = Path::new(&out_dir).join("magic_tables.rs");
    let out = File::create(magics_path).unwrap();

    gen_magics(SEED, &mut BufWriter::new(out));
}

/// Xorshift64 <https://en.wikipedia.org/wiki/Xorshift>
#[derive(Copy, Clone)]
pub struct Rng(u64);

impl Default for Rng {
    fn default() -> Self {
        Self(0xE926_E621_0D9E_3487)
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

pub const ROOK_M_SIZE: usize = 102_400;
pub const BISHOP_M_SIZE: usize = 5248;

#[derive(Clone, Copy, Default, Debug)]
struct MagicEntry {
    mask: u64,
    magic: u64,
    shift: u32,
    offset: usize,
}

impl MagicEntry {
    const fn index(&self, occupied: u64) -> usize {
        (((occupied & self.mask).wrapping_mul(self.magic)) >> self.shift) as usize + self.offset
    }
}

/// Returns a bitboards of sliding attacks given an array of 4 deltas/
/// Does not include the original position/
/// Includes occupied bits if it runs into them, but stops before going further.
fn sliding_attack(deltas: [Direction; 4], square: usize, occupied: u64) -> u64 {
    let mut attack = 0;
    for d in deltas {
        let mut sq = square;
        loop {
            let new_sq = (sq as i32 + d as i32) as usize;
            let file_diff = file(new_sq) as i32 - file(sq) as i32;
            if !(-1..=1).contains(&file_diff) || new_sq > 63 {
                break;
            }
            sq = new_sq;
            attack |= 1 << sq;
            if occupied & (1 << sq) != 0 {
                break;
            }
        }
    }

    attack
}

const RANK_1: u64 = 0xff;
const RANK_8: u64 = 0xff00000000000000;

const FILE_A: u64 = 0x0101_0101_0101_0101;
const FILE_H: u64 = 0x0101_0101_0101_0101 << 7;

/// Function generates magic numbers when they are not known.
pub fn gen_magics(seed: u64, buffer: &mut BufWriter<File>) {
    let mut rng = Rng(seed);
    let mut rook_table = Vec::with_capacity(ROOK_M_SIZE);
    let mut rook_magics = [MagicEntry::default(); 64];
    let mut bishop_table = Vec::with_capacity(BISHOP_M_SIZE);
    let mut bishop_magics = [MagicEntry::default(); 64];

    for sq in 0..64 {
        let rank = rank(sq);
        let file = file(sq);
        let file_bitboard = FILE_A << file;
        let rank_bitboard = RANK_1 << (rank * 8);
        let edges = ((RANK_1 | RANK_8) & !(rank_bitboard)) | ((FILE_A | FILE_H) & !(file_bitboard));

        let rook_bits = sliding_attack(R_DELTAS, sq, 0);
        let mask = rook_bits & !edges;
        let (mut entry, mut table) = find_magic(mask, sq, R_DELTAS, &mut rng);
        entry.offset = rook_table.len();
        rook_magics[sq] = entry;
        rook_table.append(&mut table);

        let bishop_bits = sliding_attack(B_DELTAS, sq, 0);
        let mask = bishop_bits & !edges;
        let (mut entry, mut table) = find_magic(mask, sq, B_DELTAS, &mut rng);
        entry.offset = bishop_table.len();
        bishop_magics[sq] = entry;
        bishop_table.append(&mut table);
    }

    writeln!(buffer, "#[rustfmt::skip]").unwrap();
    writeln!(buffer, "pub const BISHOP_MAGICS: &[MagicEntry; 64] = &[",).unwrap();
    for entry in bishop_magics {
        writeln!(
            buffer,
            "    MagicEntry {{ mask: 0x{:016X}, magic: 0x{:016X}, shift: {}, offset: {} }},",
            entry.mask, entry.magic, entry.shift, entry.offset
        )
        .unwrap();
    }
    writeln!(buffer, "];").unwrap();

    writeln!(buffer, "#[rustfmt::skip]").unwrap();
    writeln!(buffer, "pub const ROOK_MAGICS: &[MagicEntry; 64] = &[",).unwrap();
    for entry in rook_magics {
        writeln!(
            buffer,
            "    MagicEntry {{ mask: 0x{:016X}, magic: 0x{:016X}, shift: {}, offset: {} }},",
            entry.mask, entry.magic, entry.shift, entry.offset
        )
        .unwrap();
    }
    writeln!(buffer, "];").unwrap();

    //    writeln!(
    //        buffer,
    //        "#[derive(Clone, Copy, Default, Debug)]
    //struct MagicEntry {{
    //    mask: u64,
    //    magic: u64,
    //    shift: u32,
    //    offset: usize,
    //}}
    //
    //impl MagicEntry {{
    //    const fn index(&self, occupied: u64) -> usize {{
    //        (((occupied & self.mask).wrapping_mul(self.magic)) >> self.shift) as usize + self.offset
    //    }}
    //}}
    //        "
    //    )
    //    .unwrap();

    writeln!(buffer, "#[rustfmt::skip]").unwrap();
    writeln!(buffer, "pub static ROOK_TABLE: &[Bitboard; {}] = &[", ROOK_M_SIZE).unwrap();
    for val in rook_table.iter() {
        writeln!(buffer, "    Bitboard(0x{:016X}),", val).unwrap(); // Wrap u64 in Bitboard()
    }
    writeln!(buffer, "];\n").unwrap();

    writeln!(buffer, "#[rustfmt::skip]").unwrap();
    writeln!(buffer, "pub static BISHOP_TABLE: &[Bitboard; {}] = &[", BISHOP_M_SIZE).unwrap();
    for val in bishop_table.iter() {
        writeln!(buffer, "    Bitboard(0x{:016X}),", val).unwrap(); // Wrap u64 in Bitboard()
    }
    writeln!(buffer, "];\n").unwrap();

    assert_eq!(ROOK_M_SIZE, rook_table.len());
    assert_eq!(BISHOP_M_SIZE, bishop_table.len());
}

/// Function finds a magic valid for a given square
fn find_magic(mask: u64, sq: usize, deltas: [Direction; 4], rng: &mut Rng) -> (MagicEntry, Vec<u64>) {
    loop {
        let mut magic;
        loop {
            magic = rng.next_magic();
            if (magic.wrapping_mul(mask)).wrapping_shr(56).count_ones() >= 6 {
                break;
            }
        }

        let shift = 64 - mask.count_ones();
        let magic_entry = MagicEntry { mask, magic, shift, offset: 0 };
        if let Some(table) = make_table(deltas, sq, &magic_entry) {
            return (magic_entry, table);
        }
    }
}

/// Function tries to make a table with a given magic number
fn make_table(deltas: [Direction; 4], sq: usize, magic_entry: &MagicEntry) -> Option<Vec<u64>> {
    let idx_bits = 64 - magic_entry.shift;
    let mut table = vec![0; 1 << idx_bits];
    let mut blockers = 0;
    loop {
        let moves = sliding_attack(deltas, sq, blockers);
        let idx = magic_entry.index(blockers);

        if table[idx] == 0 {
            table[idx] = moves;
        } else if table[idx] != moves {
            return None;
        }

        // Carry-Rippler trick to iterate through all subsections of blockers
        blockers = blockers.wrapping_sub(magic_entry.mask) & magic_entry.mask;
        if blockers == 0 {
            break;
        }
    }
    Some(table)
}

fn rank(sq: usize) -> usize {
    sq >> 3
}

fn file(sq: usize) -> usize {
    sq & 0b111
}
