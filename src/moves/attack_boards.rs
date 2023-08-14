use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::moves::moves::Direction::*;
use crate::types::{bitboard::Bitboard, pieces::Color, square::Square};

const FILE_A_U64: u64 = 0x101010101010101;
const FILE_B_U64: u64 = FILE_A_U64 << 1;
const FILE_C_U64: u64 = FILE_A_U64 << 2;
const FILE_D_U64: u64 = FILE_A_U64 << 3;
const FILE_E_U64: u64 = FILE_A_U64 << 4;
const FILE_F_U64: u64 = FILE_A_U64 << 5;
const FILE_G_U64: u64 = FILE_A_U64 << 6;
const FILE_H_U64: u64 = FILE_A_U64 << 7;

pub const FILE_A: Bitboard = Bitboard(FILE_A_U64);
pub const FILE_B: Bitboard = Bitboard(FILE_B_U64);
pub const FILE_C: Bitboard = Bitboard(FILE_C_U64);
pub const FILE_D: Bitboard = Bitboard(FILE_D_U64);
pub const FILE_E: Bitboard = Bitboard(FILE_E_U64);
pub const FILE_F: Bitboard = Bitboard(FILE_F_U64);
pub const FILE_G: Bitboard = Bitboard(FILE_G_U64);
pub const FILE_H: Bitboard = Bitboard(FILE_H_U64);

const RANK1_U64: u64 = 0b11111111;
const RANK2_U64: u64 = RANK1_U64 << 8;
const RANK3_U64: u64 = RANK2_U64 << 8;
const RANK4_U64: u64 = RANK3_U64 << 8;
const RANK5_U64: u64 = RANK4_U64 << 8;
const RANK6_U64: u64 = RANK5_U64 << 8;
const RANK7_U64: u64 = RANK6_U64 << 8;
const RANK8_U64: u64 = RANK7_U64 << 8;

pub const RANK1: Bitboard = Bitboard(0b11111111);
pub const RANK2: Bitboard = Bitboard(RANK2_U64);
pub const RANK3: Bitboard = Bitboard(RANK3_U64);
pub const RANK4: Bitboard = Bitboard(RANK4_U64);
pub const RANK5: Bitboard = Bitboard(RANK5_U64);
pub const RANK6: Bitboard = Bitboard(RANK6_U64);
pub const RANK7: Bitboard = Bitboard(RANK7_U64);
pub const RANK8: Bitboard = Bitboard(RANK8_U64);

static mut KNIGHT_TABLE: [Bitboard; 64] = [Bitboard::EMPTY; 64];
static mut KING_TABLE: [Bitboard; 64] = [Bitboard::EMPTY; 64];
static mut PAWN_TABLE: [[Bitboard; 64]; 2] = [[Bitboard::EMPTY; 64]; 2];

pub fn knight_attacks(square: Square) -> Bitboard {
    unsafe { KNIGHT_TABLE[square.0 as usize] }
}

pub fn king_attacks(square: Square) -> Bitboard {
    unsafe { KING_TABLE[square.0 as usize] }
}

pub fn pawn_attacks(square: Square, attacker: Color) -> Bitboard {
    unsafe { PAWN_TABLE[attacker as usize][square.idx()] }
}

/// Non thread safe - this functions call's have to finish running before the program will
/// successfully run w/o undefined behavior
pub fn init_attack_boards() {
    gen_king_attack_boards();
    gen_knight_attack_boards();
    gen_pawn_attack_boards();
}

#[rustfmt::skip]
fn gen_king_attack_boards() {
    unsafe {
        KING_TABLE.iter_mut().enumerate().for_each(|(square, moves)| {
            let x = Square(square as u8).file();
            let y = Square(square as u8).rank();
            if y >= 1 {
                if x >= 1 { *moves |= Bitboard(1 << (square as u32 - 9)); }
                *moves |= Bitboard(1 << (square as u32 - 8));
                if x <= 6 { *moves |= Bitboard(1 << (square as u32 - 7)); }
            }

            if x >= 1 { *moves |= Bitboard(1 << (square as u32 - 1)); }
            if x <= 6 { *moves |= Bitboard(1 << (square as u32 + 1)); }

            if y <= 6 {
                if x >= 1 { *moves |= Bitboard(1 << (square as u32 + 7)); }
                *moves |= Bitboard(1 << (square as u32 + 8));
                if x <= 6 { *moves |= Bitboard(1 << (square as u32 + 9)); }
            }
        });
    }
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
    fn deltas(&self) -> (i8, i8) {
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
fn gen_knight_attack_boards() {
    unsafe {
        KNIGHT_TABLE.iter_mut().enumerate().for_each(|(square, moves)| {
            let current_rank = Square(square as u8).rank();
            let current_file = Square(square as u8).file();
            for mv in KnightMovement::iter() {
                let (dir_x, dir_y) = mv.deltas();
                if !(0..8).contains(&(current_file as i8 + dir_x)) {
                    continue;
                }
                if !(0..8).contains(&(current_rank as i8 + dir_y)) {
                    continue;
                }
                let new_index = (square as i32 + mv as i32) as u8;

                if (0..64).contains(&new_index) {
                    *moves |= Square(new_index).bitboard();
                } else {
                    continue;
                }
            }
        });
    }
}

fn gen_pawn_attack_boards() {
    unsafe {
        for sq in Square::iter() {
            let bb_square = sq.bitboard();
            let mut w = Bitboard::EMPTY;
            if let Some(w1) = bb_square.checked_shift(NorthEast) {
                w |= w1;
            }
            if let Some(w2) = bb_square.checked_shift(NorthWest) {
                w |= w2;
            }
            let mut b = Bitboard::EMPTY;
            if let Some(b1) = bb_square.checked_shift(SouthWest) {
                b |= b1;
            }
            if let Some(b2) = bb_square.checked_shift(SouthEast) {
                b |= b2;
            }
            PAWN_TABLE[Color::White as usize][sq.idx()] = w;
            PAWN_TABLE[Color::Black as usize][sq.idx()] = b;
        }
    }
}

#[cfg(test)]
mod test_attack_boards {
    use crate::{
        moves::attack_boards::{init_attack_boards, pawn_attacks},
        types::{pieces::Color, square::Square},
    };

    #[test]
    fn test_pawn_attacks() {
        init_attack_boards();
        let p_sq = Square(40);
        assert_eq!(pawn_attacks(p_sq, Color::Black), Square(33).bitboard());
        assert_eq!(pawn_attacks(p_sq, Color::White), Square(49).bitboard());

        let p_sq = Square(19);
        assert_eq!(
            pawn_attacks(p_sq, Color::Black),
            (Square(10).bitboard() | Square(12).bitboard())
        );
        assert_eq!(
            pawn_attacks(p_sq, Color::White),
            (Square(26).bitboard() | Square(28).bitboard())
        );
    }
}