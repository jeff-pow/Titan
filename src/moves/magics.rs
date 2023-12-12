use crate::{
    moves::attack_boards::{FILE_A, FILE_H, RANK1, RANK8},
    types::{bitboard::Bitboard, square::Square},
};

use super::moves::{Direction, Direction::*};

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
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;

        // self.0 ^= self.0 << 12;
        // self.0 ^= self.0 >> 25;
        // self.0 ^= self.0 << 27;
        // self.0 *= 0x2545F4914F6CDD1D;
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

#[derive(Clone)]
pub struct Magics {
    rook_table: Vec<Bitboard>,
    bishop_table: Vec<Bitboard>,
}

fn index(entry: &MagicEntry, occupied: Bitboard) -> usize {
    let blockers = occupied.0 & entry.mask.0;
    let hash = blockers.wrapping_mul(entry.magic);
    let index = (hash >> entry.shift) as usize;
    entry.offset + index
    // unsafe { entry.offset + _pext_u64(occupied.0, entry.mask.0) as usize }
}

/// https://analog-hors.github.io/site/magic-bitboards/
impl Magics {
    pub fn bishop_attacks(&self, occupied: Bitboard, sq: Square) -> Bitboard {
        let magic = &BISHOP_MAGICS[sq];
        self.bishop_table[index(magic, occupied)]
    }

    pub fn rook_attacks(&self, occupied: Bitboard, sq: Square) -> Bitboard {
        let magic = &ROOK_MAGICS[sq];
        self.rook_table[index(magic, occupied)]
    }
}

/// This assumes the magics are already known and placed in a const array
impl Default for Magics {
    fn default() -> Self {
        let mut rook_table = Vec::with_capacity(ROOK_M_SIZE);
        let mut bishop_table = Vec::with_capacity(BISHOP_M_SIZE);

        for sq in Square::iter() {
            let mut table = create_table(sq, R_DELTAS);
            rook_table.append(&mut table);

            let mut table = create_table(sq, B_DELTAS);
            bishop_table.append(&mut table);
        }

        assert_eq!(ROOK_M_SIZE, rook_table.len());
        assert_eq!(BISHOP_M_SIZE, bishop_table.len());

        Self { rook_table, bishop_table }
    }
}

/// Extracts move bitboards using known constants
fn create_table(sq: Square, deltas: [Direction; 4]) -> Vec<Bitboard> {
    let magic_entry = if deltas.contains(&North) { ROOK_MAGICS[sq] } else { BISHOP_MAGICS[sq] };
    let idx_bits = 64 - magic_entry.shift;
    let mut table = vec![Bitboard::EMPTY; 1 << idx_bits];
    let mut blockers = Bitboard::EMPTY;
    loop {
        let moves = sliding_attack(deltas, sq, blockers);
        let idx = index(&magic_entry, blockers) - magic_entry.offset;

        table[idx] = moves;

        // Carry-Rippler trick to iterate through all subsections of blockers
        blockers.0 = blockers.0.wrapping_sub(magic_entry.mask.0) & magic_entry.mask.0;
        if blockers == Bitboard::EMPTY {
            break;
        }
    }
    table
}

/// Returns a bitboards of sliding attacks given an array of 4 deltas/
/// Does not include the original position/
/// Includes occupied bits if it runs into them, but stops before going further.
fn sliding_attack(deltas: [Direction; 4], sq: Square, occupied: Bitboard) -> Bitboard {
    let mut attack = Bitboard::EMPTY;
    for dir in deltas {
        let mut s = sq.shift(dir);
        'inner: while s.is_valid() && s.dist(s.shift(dir.opp())) == 1 {
            attack |= s.bitboard();
            if occupied & s.bitboard() != Bitboard::EMPTY {
                break 'inner;
            }
            s = s.shift(dir);
        }
    }
    attack
}

#[rustfmt::skip]
const BISHOP_MAGICS: &[MagicEntry; 64] = &[
    MagicEntry { mask: Bitboard(0x0040201008040200), magic: 0x800B021009020184, shift: 58, offset: 0 },
    MagicEntry { mask: Bitboard(0x0000402010080400), magic: 0x2020710C01104108, shift: 59, offset: 64 },
    MagicEntry { mask: Bitboard(0x0000004020100A00), magic: 0x00A2060C01221000, shift: 59, offset: 96 },
    MagicEntry { mask: Bitboard(0x0000000040221400), magic: 0x0204042086050800, shift: 59, offset: 128 },
    MagicEntry { mask: Bitboard(0x0000000002442800), magic: 0x8010882000100008, shift: 59, offset: 160 },
    MagicEntry { mask: Bitboard(0x0000000204085000), magic: 0x2400900420008000, shift: 59, offset: 192 },
    MagicEntry { mask: Bitboard(0x0000020408102000), magic: 0x000100882188090A, shift: 59, offset: 224 },
    MagicEntry { mask: Bitboard(0x0002040810204000), magic: 0x0002840401040200, shift: 58, offset: 256 },
    MagicEntry { mask: Bitboard(0x0020100804020000), magic: 0x0044100210010210, shift: 59, offset: 320 },
    MagicEntry { mask: Bitboard(0x0040201008040000), magic: 0x0100200104210040, shift: 59, offset: 352 },
    MagicEntry { mask: Bitboard(0x00004020100A0000), magic: 0x40C0210204004008, shift: 59, offset: 384 },
    MagicEntry { mask: Bitboard(0x0000004022140000), magic: 0x0100040410900001, shift: 59, offset: 416 },
    MagicEntry { mask: Bitboard(0x0000000244280000), magic: 0x000204042000A088, shift: 59, offset: 448 },
    MagicEntry { mask: Bitboard(0x0000020408500000), magic: 0x4020121110480142, shift: 59, offset: 480 },
    MagicEntry { mask: Bitboard(0x0002040810200000), magic: 0x0401084510105000, shift: 59, offset: 512 },
    MagicEntry { mask: Bitboard(0x0004081020400000), magic: 0x2101102518080408, shift: 59, offset: 544 },
    MagicEntry { mask: Bitboard(0x0010080402000200), magic: 0x08602030A1021084, shift: 59, offset: 576 },
    MagicEntry { mask: Bitboard(0x0020100804000400), magic: 0x0014044928408C00, shift: 59, offset: 608 },
    MagicEntry { mask: Bitboard(0x004020100A000A00), magic: 0x2120801000801440, shift: 57, offset: 640 },
    MagicEntry { mask: Bitboard(0x0000402214001400), magic: 0x000C0802012200C0, shift: 57, offset: 768 },
    MagicEntry { mask: Bitboard(0x0000024428002800), magic: 0x0110800C00A04001, shift: 57, offset: 896 },
    MagicEntry { mask: Bitboard(0x0002040850005000), magic: 0x8002810D0088C001, shift: 57, offset: 1024 },
    MagicEntry { mask: Bitboard(0x0004081020002000), magic: 0x0029200608010488, shift: 59, offset: 1152 },
    MagicEntry { mask: Bitboard(0x0008102040004000), magic: 0x0000200202020210, shift: 59, offset: 1184 },
    MagicEntry { mask: Bitboard(0x0008040200020400), magic: 0x1004040620881088, shift: 59, offset: 1216 },
    MagicEntry { mask: Bitboard(0x0010080400040800), magic: 0x80042020100288A1, shift: 59, offset: 1248 },
    MagicEntry { mask: Bitboard(0x0020100A000A1000), magic: 0x818228051011C442, shift: 57, offset: 1280 },
    MagicEntry { mask: Bitboard(0x0040221400142200), magic: 0x0201004004040002, shift: 55, offset: 1408 },
    MagicEntry { mask: Bitboard(0x0002442800284400), magic: 0x0140840204802001, shift: 55, offset: 1920 },
    MagicEntry { mask: Bitboard(0x0004085000500800), magic: 0x0082012022029001, shift: 57, offset: 2432 },
    MagicEntry { mask: Bitboard(0x0008102000201000), magic: 0x2002060000631044, shift: 59, offset: 2560 },
    MagicEntry { mask: Bitboard(0x0010204000402000), magic: 0xB0050844002C0421, shift: 59, offset: 2592 },
    MagicEntry { mask: Bitboard(0x0004020002040800), magic: 0x0064212000084200, shift: 59, offset: 2624 },
    MagicEntry { mask: Bitboard(0x0008040004081000), magic: 0x2801040223202800, shift: 59, offset: 2656 },
    MagicEntry { mask: Bitboard(0x00100A000A102000), magic: 0x1000413000460400, shift: 57, offset: 2688 },
    MagicEntry { mask: Bitboard(0x0022140014224000), magic: 0x0100020080080080, shift: 55, offset: 2816 },
    MagicEntry { mask: Bitboard(0x0044280028440200), magic: 0x8004008200040104, shift: 55, offset: 3328 },
    MagicEntry { mask: Bitboard(0x0008500050080400), magic: 0x05A2008900420040, shift: 57, offset: 3840 },
    MagicEntry { mask: Bitboard(0x0010200020100800), magic: 0x020418004414AC0B, shift: 59, offset: 3968 },
    MagicEntry { mask: Bitboard(0x0020400040201000), magic: 0x4002004104004400, shift: 59, offset: 4000 },
    MagicEntry { mask: Bitboard(0x0002000204081000), magic: 0x089210042060C690, shift: 59, offset: 4032 },
    MagicEntry { mask: Bitboard(0x0004000408102000), magic: 0x0411010121101004, shift: 59, offset: 4064 },
    MagicEntry { mask: Bitboard(0x000A000A10204000), magic: 0x0202001048000400, shift: 57, offset: 4096 },
    MagicEntry { mask: Bitboard(0x0014001422400000), magic: 0x0000184200804800, shift: 57, offset: 4224 },
    MagicEntry { mask: Bitboard(0x0028002844020000), magic: 0x0C04040408200401, shift: 57, offset: 4352 },
    MagicEntry { mask: Bitboard(0x0050005008040200), magic: 0x012202046A014900, shift: 57, offset: 4480 },
    MagicEntry { mask: Bitboard(0x0020002010080400), magic: 0x0248281800600081, shift: 59, offset: 4608 },
    MagicEntry { mask: Bitboard(0x0040004020100800), magic: 0xC281881081009086, shift: 59, offset: 4640 },
    MagicEntry { mask: Bitboard(0x0000020408102000), magic: 0x00040104022201B2, shift: 59, offset: 4672 },
    MagicEntry { mask: Bitboard(0x0000040810204000), magic: 0x10060610B20801A0, shift: 59, offset: 4704 },
    MagicEntry { mask: Bitboard(0x00000A1020400000), magic: 0x4500029400880402, shift: 59, offset: 4736 },
    MagicEntry { mask: Bitboard(0x0000142240000000), magic: 0x405000C042022008, shift: 59, offset: 4768 },
    MagicEntry { mask: Bitboard(0x0000284402000000), magic: 0x94C0188A10240006, shift: 59, offset: 4800 },
    MagicEntry { mask: Bitboard(0x0000500804020000), magic: 0x0822108210410040, shift: 59, offset: 4832 },
    MagicEntry { mask: Bitboard(0x0000201008040200), magic: 0x8060040408105082, shift: 59, offset: 4864 },
    MagicEntry { mask: Bitboard(0x0000402010080400), magic: 0x8011500095004000, shift: 59, offset: 4896 },
    MagicEntry { mask: Bitboard(0x0002040810204000), magic: 0x28183200B0019000, shift: 58, offset: 4928 },
    MagicEntry { mask: Bitboard(0x0004081020400000), magic: 0x0009010042022022, shift: 59, offset: 4992 },
    MagicEntry { mask: Bitboard(0x000A102040000000), magic: 0x0123005104009202, shift: 59, offset: 5024 },
    MagicEntry { mask: Bitboard(0x0014224000000000), magic: 0x4000000000208811, shift: 59, offset: 5056 },
    MagicEntry { mask: Bitboard(0x0028440200000000), magic: 0x1154200040050100, shift: 59, offset: 5088 },
    MagicEntry { mask: Bitboard(0x0050080402000000), magic: 0x0000000910010202, shift: 59, offset: 5120 },
    MagicEntry { mask: Bitboard(0x0020100804020000), magic: 0x80C240D889410600, shift: 59, offset: 5152 },
    MagicEntry { mask: Bitboard(0x0040201008040200), magic: 0x4020021002008010, shift: 58, offset: 5184 },
];

#[rustfmt::skip]
const ROOK_MAGICS: &[MagicEntry; 64] = &[
    MagicEntry { mask: Bitboard(0x000101010101017E), magic: 0x188000824000A135, shift: 52, offset: 0 },
    MagicEntry { mask: Bitboard(0x000202020202027C), magic: 0x8840001000402004, shift: 53, offset: 4096 },
    MagicEntry { mask: Bitboard(0x000404040404047A), magic: 0x8180100081200008, shift: 53, offset: 6144 },
    MagicEntry { mask: Bitboard(0x0008080808080876), magic: 0xA080048010000800, shift: 53, offset: 8192 },
    MagicEntry { mask: Bitboard(0x001010101010106E), magic: 0x06000200A0080410, shift: 53, offset: 10240 },
    MagicEntry { mask: Bitboard(0x002020202020205E), magic: 0x8200011004020008, shift: 53, offset: 12288 },
    MagicEntry { mask: Bitboard(0x004040404040403E), magic: 0x9080020001000080, shift: 53, offset: 14336 },
    MagicEntry { mask: Bitboard(0x008080808080807E), magic: 0x0200002040810214, shift: 52, offset: 16384 },
    MagicEntry { mask: Bitboard(0x0001010101017E00), magic: 0x0800802040008000, shift: 53, offset: 20480 },
    MagicEntry { mask: Bitboard(0x0002020202027C00), magic: 0x2043004001028020, shift: 54, offset: 22528 },
    MagicEntry { mask: Bitboard(0x0004040404047A00), magic: 0x0021001040200100, shift: 54, offset: 23552 },
    MagicEntry { mask: Bitboard(0x0008080808087600), magic: 0x0166000810460021, shift: 54, offset: 24576 },
    MagicEntry { mask: Bitboard(0x0010101010106E00), magic: 0x0458800800040080, shift: 54, offset: 25600 },
    MagicEntry { mask: Bitboard(0x0020202020205E00), magic: 0x0986003002000408, shift: 54, offset: 26624 },
    MagicEntry { mask: Bitboard(0x0040404040403E00), magic: 0x0021000441000200, shift: 54, offset: 27648 },
    MagicEntry { mask: Bitboard(0x0080808080807E00), magic: 0x0001000200804100, shift: 53, offset: 28672 },
    MagicEntry { mask: Bitboard(0x00010101017E0100), magic: 0x0D80014000402010, shift: 53, offset: 30720 },
    MagicEntry { mask: Bitboard(0x00020202027C0200), magic: 0x02100A4010402000, shift: 54, offset: 32768 },
    MagicEntry { mask: Bitboard(0x00040404047A0400), magic: 0x0200808010002000, shift: 54, offset: 33792 },
    MagicEntry { mask: Bitboard(0x0008080808760800), magic: 0x1106020010082141, shift: 54, offset: 34816 },
    MagicEntry { mask: Bitboard(0x00101010106E1000), magic: 0x0848008004008008, shift: 54, offset: 35840 },
    MagicEntry { mask: Bitboard(0x00202020205E2000), magic: 0x2014008080040200, shift: 54, offset: 36864 },
    MagicEntry { mask: Bitboard(0x00404040403E4000), magic: 0x0000010100020004, shift: 54, offset: 37888 },
    MagicEntry { mask: Bitboard(0x00808080807E8000), magic: 0x000402000504A054, shift: 53, offset: 38912 },
    MagicEntry { mask: Bitboard(0x000101017E010100), magic: 0x0000800080204000, shift: 53, offset: 40960 },
    MagicEntry { mask: Bitboard(0x000202027C020200), magic: 0x8000500340022000, shift: 54, offset: 43008 },
    MagicEntry { mask: Bitboard(0x000404047A040400), magic: 0x0010900180200080, shift: 54, offset: 44032 },
    MagicEntry { mask: Bitboard(0x0008080876080800), magic: 0x0280104200220008, shift: 54, offset: 45056 },
    MagicEntry { mask: Bitboard(0x001010106E101000), magic: 0x0A84008480080080, shift: 54, offset: 46080 },
    MagicEntry { mask: Bitboard(0x002020205E202000), magic: 0x8001000900040002, shift: 54, offset: 47104 },
    MagicEntry { mask: Bitboard(0x004040403E404000), magic: 0x2400010400100208, shift: 54, offset: 48128 },
    MagicEntry { mask: Bitboard(0x008080807E808000), magic: 0x1423244600010084, shift: 53, offset: 49152 },
    MagicEntry { mask: Bitboard(0x0001017E01010100), magic: 0x1600400222800180, shift: 53, offset: 51200 },
    MagicEntry { mask: Bitboard(0x0002027C02020200), magic: 0x13C00083050041E0, shift: 54, offset: 53248 },
    MagicEntry { mask: Bitboard(0x0004047A04040400), magic: 0x4100100080802004, shift: 54, offset: 54272 },
    MagicEntry { mask: Bitboard(0x0008087608080800), magic: 0x5020800800801000, shift: 54, offset: 55296 },
    MagicEntry { mask: Bitboard(0x0010106E10101000), magic: 0x0911000801001004, shift: 54, offset: 56320 },
    MagicEntry { mask: Bitboard(0x0020205E20202000), magic: 0x0012001002000804, shift: 54, offset: 57344 },
    MagicEntry { mask: Bitboard(0x0040403E40404000), magic: 0x0060082A84004110, shift: 54, offset: 58368 },
    MagicEntry { mask: Bitboard(0x0080807E80808000), magic: 0x4460040082000041, shift: 53, offset: 59392 },
    MagicEntry { mask: Bitboard(0x00017E0101010100), magic: 0x4081008002450020, shift: 53, offset: 61440 },
    MagicEntry { mask: Bitboard(0x00027C0202020200), magic: 0x402050012004C000, shift: 54, offset: 63488 },
    MagicEntry { mask: Bitboard(0x00047A0404040400), magic: 0x1C00200041010010, shift: 54, offset: 64512 },
    MagicEntry { mask: Bitboard(0x0008760808080800), magic: 0x0040400812020020, shift: 54, offset: 65536 },
    MagicEntry { mask: Bitboard(0x00106E1010101000), magic: 0x1002009008060020, shift: 54, offset: 66560 },
    MagicEntry { mask: Bitboard(0x00205E2020202000), magic: 0x8402000804010100, shift: 54, offset: 67584 },
    MagicEntry { mask: Bitboard(0x00403E4040404000), magic: 0x0200C20110040008, shift: 54, offset: 68608 },
    MagicEntry { mask: Bitboard(0x00807E8080808000), magic: 0x30008048811A0004, shift: 53, offset: 69632 },
    MagicEntry { mask: Bitboard(0x007E010101010100), magic: 0x0000350480004100, shift: 53, offset: 71680 },
    MagicEntry { mask: Bitboard(0x007C020202020200), magic: 0x4000400080200280, shift: 54, offset: 73728 },
    MagicEntry { mask: Bitboard(0x007A040404040400), magic: 0x0200200111014300, shift: 54, offset: 74752 },
    MagicEntry { mask: Bitboard(0x0076080808080800), magic: 0x0010080010008080, shift: 54, offset: 75776 },
    MagicEntry { mask: Bitboard(0x006E101010101000), magic: 0x2002001008042200, shift: 54, offset: 76800 },
    MagicEntry { mask: Bitboard(0x005E202020202000), magic: 0x2002000804100200, shift: 54, offset: 77824 },
    MagicEntry { mask: Bitboard(0x003E404040404000), magic: 0x0040210822100400, shift: 54, offset: 78848 },
    MagicEntry { mask: Bitboard(0x007E808080808000), magic: 0x0002010044008200, shift: 53, offset: 79872 },
    MagicEntry { mask: Bitboard(0x7E01010101010100), magic: 0x0827005600244082, shift: 52, offset: 81920 },
    MagicEntry { mask: Bitboard(0x7C02020202020200), magic: 0x2029244002811101, shift: 53, offset: 86016 },
    MagicEntry { mask: Bitboard(0x7A04040404040400), magic: 0x0005402080089202, shift: 53, offset: 88064 },
    MagicEntry { mask: Bitboard(0x7608080808080800), magic: 0x0010210004081001, shift: 53, offset: 90112 },
    MagicEntry { mask: Bitboard(0x6E10101010101000), magic: 0x0002000408102002, shift: 53, offset: 92160 },
    MagicEntry { mask: Bitboard(0x5E20202020202000), magic: 0x0402001008040102, shift: 53, offset: 94208 },
    MagicEntry { mask: Bitboard(0x3E40404040404000), magic: 0x0002000084080102, shift: 53, offset: 96256 },
    MagicEntry { mask: Bitboard(0x7E80808080808000), magic: 0x0018010290224402, shift: 52, offset: 98304 },
];

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

    println!("#[rustfmt::skip]");
    println!("pub const BISHOP_MAGICS: &[MagicEntry; 64] = &[",);
    for entry in bishop_magics {
        println!(
            "    MagicEntry {{ mask: Bitboard(0x{:016X}), magic: 0x{:016X}, shift: {}, offset: {} }},",
            entry.mask.0, entry.magic, entry.shift, entry.offset
        );
    }
    println!("];");

    println!("#[rustfmt::skip]");
    println!("pub const ROOK_MAGICS: &[MagicEntry; 64] = &[",);
    for entry in rook_magics {
        println!(
            "    MagicEntry {{ mask: Bitboard(0x{:016X}), magic: 0x{:016X}, shift: {}, offset: {} }},",
            entry.mask.0, entry.magic, entry.shift, entry.offset
        );
    }
    println!("];");

    assert_eq!(ROOK_M_SIZE, rook_table.len());
    assert_eq!(BISHOP_M_SIZE, bishop_table.len());
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
        let idx = index(magic_entry, blockers);

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
