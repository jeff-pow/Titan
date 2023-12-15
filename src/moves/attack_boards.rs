use crate::const_array;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::types::{bitboard::Bitboard, pieces::Color, square::Square};

const FILE_A_U64: u64 = 0x101010101010101;
// const FILE_B_U64: u64 = 0x101010101010101 << 1;
// const FILE_C_U64: u64 = 0x101010101010101 << 2;
// const FILE_D_U64: u64 = 0x101010101010101 << 3;
// const FILE_E_U64: u64 = 0x101010101010101 << 4;
// const FILE_F_U64: u64 = 0x101010101010101 << 5;
// const FILE_G_U64: u64 = 0x101010101010101 << 6;
const FILE_H_U64: u64 = 0x101010101010101 << 7;

pub const FILE_A: Bitboard = Bitboard(FILE_A_U64);
pub const FILE_B: Bitboard = Bitboard(FILE_A_U64 << 1);
pub const FILE_C: Bitboard = Bitboard(FILE_A_U64 << 2);
pub const FILE_D: Bitboard = Bitboard(FILE_A_U64 << 3);
pub const FILE_E: Bitboard = Bitboard(FILE_A_U64 << 4);
pub const FILE_F: Bitboard = Bitboard(FILE_A_U64 << 5);
pub const FILE_G: Bitboard = Bitboard(FILE_A_U64 << 6);
pub const FILE_H: Bitboard = Bitboard(FILE_A_U64 << 7);

const RANK1_U64: u64 = 0b11111111;
// const RANK2_U64: u64 = 0b11111111 << 8;
// const RANK3_U64: u64 = 0b11111111 << 16;
// const RANK4_U64: u64 = 0b11111111 << 24;
// const RANK5_U64: u64 = 0b11111111 << 32;
// const RANK6_U64: u64 = 0b11111111 << 40;
// const RANK7_U64: u64 = 0b11111111 << 48;
// const RANK8_U64: u64 = 0b11111111 << 56;

pub const RANK1: Bitboard = Bitboard(0b11111111);
pub const RANK2: Bitboard = Bitboard(RANK1_U64 << 8);
pub const RANK3: Bitboard = Bitboard(RANK1_U64 << 16);
pub const RANK4: Bitboard = Bitboard(RANK1_U64 << 24);
pub const RANK5: Bitboard = Bitboard(RANK1_U64 << 32);
pub const RANK6: Bitboard = Bitboard(RANK1_U64 << 40);
pub const RANK7: Bitboard = Bitboard(RANK1_U64 << 48);
pub const RANK8: Bitboard = Bitboard(RANK1_U64 << 56);

pub const KING_ATTACKS: [Bitboard; 64] = const_array!(|sq, 64| {
    let sq = 1 << sq;
    // Create a bitboard out of the square
    let mut bb = sq;
    // Put in the bits above and below - These won't have any effect if they are outside of the range
    // of the board
    bb |= sq << 8 | sq >> 8;
    // Then literally shake your column of bits back and forth to get diagonals and horizontal moves
    bb |= (bb & !FILE_A_U64) >> 1 | (bb & !FILE_H_U64) << 1;
    // Remove the square the piece is currently on from possible attacks
    Bitboard(bb ^ sq)
});

pub const KNIGHT_ATTACKS: [Bitboard; 64] = const_array!(|sq, 64| {
    let sq = 1 << sq;
    let mut bb = sq;
    // Get squares two rows above and below current occupied square
    let vert = sq << 16 | sq >> 16;
    // Shake those bits back and forth as long as it wouldn't end up in another row
    bb |= (vert & !FILE_A_U64) >> 1 | (vert & !FILE_H_U64) << 1;
    // Get squares two columns to the left and right of current occupied square. Constants ensure you
    // won't go to a different row
    let horizontal = (sq & 0x3f3f3f3f3f3f3f3f) << 2 | (sq & 0xfcfcfcfcfcfcfcfc) >> 2;
    // Shake those bits back and forth - can't go out of bounds vertically
    bb |= horizontal << 8 | horizontal >> 8;
    // Remove current occupied square from final attack board
    Bitboard(bb ^ sq)
});
#[rustfmt::skip]
pub(crate) fn gen_king_attack_boards() -> [Bitboard; 64] {
    let mut arr = [Bitboard::EMPTY; 64];
        arr.iter_mut().enumerate().for_each(|(sq, bitboard)| {
            let x = Square(sq as u32).file();
            let y = Square(sq as u32).rank();
            if y >= 1 {
                if x >= 1 { *bitboard |= Bitboard(1 << (sq as u32 - 9)); }
                *bitboard |= Bitboard(1 << (sq as u32 - 8));
                if x <= 6 { *bitboard |= Bitboard(1 << (sq as u32 - 7)); }
            }

            if x >= 1 { *bitboard |= Bitboard(1 << (sq as u32 - 1)); }
            if x <= 6 { *bitboard |= Bitboard(1 << (sq as u32 + 1)); }

            if y <= 6 {
                if x >= 1 { *bitboard |= Bitboard(1 << (sq as u32 + 7)); }
                *bitboard |= Bitboard(1 << (sq as u32 + 8));
                if x <= 6 { *bitboard |= Bitboard(1 << (sq as u32 + 9)); }
            }
        });
    arr
}

/// Movement chords are defined by a combination of three cardinal directions - ex West West North
#[derive(EnumIter, Copy, Clone, Debug, PartialEq, Eq)]
#[allow(clippy::upper_case_acronyms)]
enum KnightMovement {
    WWN = 6,
    WNN = 15,
    ENN = 17,
    EEN = 10,
    EES = -6,
    ESS = -15,
    WSS = -17,
    WWS = -10,
}

/// Converts a direction of moves into a tuple of x,y movement
impl KnightMovement {
    fn deltas(&self) -> (i32, i32) {
        match self {
            KnightMovement::WWN => (-2, 1),
            KnightMovement::WNN => (-1, 2),
            KnightMovement::ENN => (1, 2),
            KnightMovement::EEN => (2, 1),
            KnightMovement::EES => (2, -1),
            KnightMovement::ESS => (1, -2),
            KnightMovement::WSS => (-1, -2),
            KnightMovement::WWS => (-2, -1),
        }
    }
}

#[rustfmt::skip]
pub(crate) fn gen_knight_attack_boards() -> [Bitboard; 64] {
    let mut arr = [Bitboard::EMPTY; 64];
        arr.iter_mut().enumerate().for_each(|(sq, bitboard)| {
            let current_rank = Square(sq as u32).rank();
            let current_file = Square(sq as u32).file();
            for mv in KnightMovement::iter() {
                let (dir_x, dir_y) = mv.deltas();
                if !(0..8).contains(&(current_file as i32 + dir_x)) {
                    continue;
                }
                if !(0..8).contains(&(current_rank as i32 + dir_y)) {
                    continue;
                }
                let new_index = (sq as i32 + mv as i32) as u32;

                if (0..64).contains(&new_index) {
                    *bitboard |= Square(new_index).bitboard();
                } else {
                    continue;
                }
            }
        });
    arr
}

pub(crate) fn gen_pawn_attack_boards() -> [[Bitboard; 64]; 2] {
    let mut arr = [[Bitboard::EMPTY; 64]; 2];
    for sq in Square::iter() {
        arr[Color::White][sq] =
            Bitboard((sq.bitboard() & !FILE_A).0 << 7 | ((sq.bitboard() & !FILE_H).0 << 9));

        arr[Color::Black][sq] =
            Bitboard((sq.bitboard() & !FILE_A).0 >> 9 | ((sq.bitboard() & !FILE_H).0 >> 7));
    }
    arr
}

pub const PAWN_ATTACKS: [[Bitboard; 64]; 2] = [
    const_array!(|sq, 64| Bitboard(
        ((1 << sq) & !FILE_A_U64) << 7 | (((1 << sq) & !FILE_H_U64) << 9)
    )),
    const_array!(|sq, 64| Bitboard(
        ((1 << sq) & !FILE_A_U64) >> 9 | (((1 << sq) & !FILE_H_U64) >> 7)
    )),
];

#[macro_export]
/// Credit for this macro goes to akimbo
macro_rules! const_array {
    (| $i:ident, $size:literal | $($r:tt)+) => {{
        let mut $i = 0;
        let mut res = [{$($r)+}; $size];
        while $i < $size - 1 {
            $i += 1;
            res[$i] = {$($r)+};
        }
        res
    }}
}

// #[macro_export]
// macro_rules! const_array {
//     ($size:expr, |$i:ident| $calc:expr) => {{
//         const SIZE: usize = $size;
//         let mut arr = [0; SIZE];
//         let mut $i = 0;
//         while $i < SIZE {
//             arr[$i] = $calc;
//             $i += 1;
//         }
//         arr
//     }};
// }

#[cfg(test)]
mod test_attack_boards {
    use crate::{
        moves::movegenerator::MG,
        types::{pieces::Color, square::Square},
    };

    #[test]
    fn test_pawn_attacks() {
        let p_sq = Square(40);
        assert_eq!(MG.pawn_attacks(p_sq, Color::Black), Square(33).bitboard());
        assert_eq!(MG.pawn_attacks(p_sq, Color::White), Square(49).bitboard());

        let p_sq = Square(19);
        assert_eq!(
            MG.pawn_attacks(p_sq, Color::Black),
            (Square(10).bitboard() | Square(12).bitboard())
        );
        assert_eq!(
            MG.pawn_attacks(p_sq, Color::White),
            (Square(26).bitboard() | Square(28).bitboard())
        );
    }
}
