use std::ops::{Shl, ShlAssign};
use crate::{
    attack_boards::*,
    moves::{self, coordinates, rank, Direction},
};

#[inline]
pub fn create_square_and_checked_shift(bitboard: u64, dir: Direction) -> Option<u64> {
    let bitboard = bitboard.max(1);
    match dir {
        Direction::North => {
            if bitboard.leading_zeros() < 8 {
                None
            } else {
                bitboard.checked_shl(8)
            }
        }
        Direction::NorthWest => {
            let shifted = (bitboard.checked_shl(7).unwrap_or(0)) & !FILE_H;
            if bitboard.leading_zeros() >= 7 && shifted.trailing_zeros() >= 7 {
                Some(shifted)
            } else {
                None
            }
        }
        Direction::West => {
            let shifted = (bitboard.checked_shr(1).unwrap_or(0)) & !FILE_H;
            if shifted.leading_zeros() >= 7 && bitboard.trailing_zeros() >= 1 {
                Some(shifted)
            } else {
                None
            }
        }
        Direction::SouthWest => {
            let shifted = (bitboard.checked_shr(9).unwrap_or(0)) & !FILE_H;
            if shifted.leading_zeros() >= 7 && bitboard.trailing_zeros() >= 9 {
                Some(shifted)
            } else {
                None
            }
        }
        Direction::South => {
            if bitboard.trailing_zeros() < 8 {
                None
            } else {
                bitboard.checked_shr(8)
            }
        }
        Direction::SouthEast => {
            let shifted = (bitboard.checked_shr(7).unwrap_or(0)) & !FILE_A;
            if bitboard.trailing_zeros() >= 7 && shifted.leading_zeros() >= 7 {
                Some(shifted)
            } else {
                None
            }
        }
        Direction::East => {
            let shifted = (bitboard.checked_shl(1).unwrap_or(0)) & !FILE_A;
            if bitboard.leading_zeros() >= 1 && shifted.trailing_zeros() >= 7 {
                Some(shifted)
            } else {
                None
            }
        }
        Direction::NorthEast => {
            let shifted = (bitboard.checked_shl(9).unwrap_or(0)) & !FILE_A;
            if bitboard.leading_zeros() >= 9 && shifted.trailing_zeros() >= 7 {
                Some(shifted)
            } else {
                None
            }
        }
    }
}

#[inline]
pub fn shift(bitboard: u64, dir: Direction) -> u64 {
    match dir {
        Direction::North => bitboard << 8,
        Direction::NorthWest => (bitboard << 7) & !FILE_H,
        Direction::West => (bitboard >> 1) & !FILE_H,
        Direction::SouthWest => (bitboard >> 9) & !FILE_H,
        Direction::South => bitboard >> 8,
        Direction::SouthEast => (bitboard >> 7) & !FILE_A,
        Direction::East => (bitboard << 1) & !FILE_A,
        Direction::NorthEast => (bitboard << 9) & !FILE_A,
    }
}

#[inline]
pub fn pop_lsb(bb: &mut u64) -> u64 {
    let lsb = *bb & bb.wrapping_neg();
    *bb ^= lsb;
    lsb.trailing_zeros() as u64
}

#[inline]
pub fn bit_is_on(bb: u64, idx: usize) -> bool {
    bb & (1 << idx) != 0
}

#[inline]
pub fn bit_is_off(bb: u64, idx: usize) -> bool {
    bb & (1 << idx) == 0
}

#[inline]
pub fn get_rank_bitboard(square: u8) -> u64 {
    let x = moves::rank(square);
    match x {
        0 => RANK1,
        1 => RANK2,
        2 => RANK3,
        3 => RANK4,
        4 => RANK5,
        5 => RANK6,
        6 => RANK7,
        7 => RANK8,
        _ => panic!(),
    }
}

#[inline]
pub fn get_file_bitboard(square: u8) -> u64 {
    let y = moves::file(square);
    match y {
        0 => FILE_A,
        1 => FILE_B,
        2 => FILE_C,
        3 => FILE_D,
        4 => FILE_E,
        5 => FILE_F,
        6 => FILE_G,
        7 => FILE_H,
        _ => panic!(),
    }
}

#[inline]
pub fn distance(s1: u8, s2: u8) -> u64 {
    let (x1, y1) = coordinates(s1 as usize);
    let (x2, y2) = coordinates(s2 as usize);
    let x_diff = x1.abs_diff(x2);
    let y_diff = y1.abs_diff(y2);
    x_diff.max(y_diff) as u64
}

pub fn dist(s1: u8, s2: u8) -> u64 {
    let x1 = rank(s1);
    let y1 = moves::file(s1);
    let x2 = rank(s2);
    let y2 = moves::file(s2);
    let x_diff = x1.abs_diff(x2);
    let y_diff = y1.abs_diff(y2);
    x_diff.max(y_diff) as u64
}

#[cfg(test)]
mod bitboard_tests {
    use strum::IntoEnumIterator;

    use super::*;

    #[test]
    fn test_shift_valid() {
        let bitboard = 0b1100_0000_0000_0000;
        assert_eq!(
            shift(bitboard, Direction::North),
            0b1100_0000_0000_0000 << 8
        );
        assert_eq!(
            shift(bitboard, Direction::South),
            0b1100_0000_0000_0000 >> 8
        );
        assert_eq!(
            shift(bitboard, Direction::East),
            (0b1100_0000_0000_0000 << 1) & !FILE_A
        );
        assert_eq!(
            shift(bitboard, Direction::West),
            (0b1100_0000_0000_0000 >> 1) & !FILE_H
        );
        assert_eq!(
            shift(bitboard, Direction::NorthEast),
            (0b1100_0000_0000_0000 << 9) & !FILE_A
        );
        assert_eq!(
            shift(bitboard, Direction::NorthWest),
            (0b1100_0000_0000_0000 << 7) & !FILE_H
        );
        assert_eq!(
            shift(bitboard, Direction::SouthEast),
            (0b1100_0000_0000_0000 >> 7) & !FILE_A
        );
        assert_eq!(
            shift(bitboard, Direction::SouthWest),
            (0b1100_0000_0000_0000 >> 9) & !FILE_H
        );
    }

    #[test]
    fn test_checked_shift_valid() {
        let bitboard = 0b1100_0000_0000_0000;
        assert_eq!(
            checked_shift(bitboard, Direction::North),
            Some(0b1100_0000_0000_0000 << 8)
        );
        assert_eq!(
            checked_shift(bitboard, Direction::South),
            Some(0b1100_0000_0000_0000 >> 8)
        );
        assert_eq!(
            checked_shift(bitboard, Direction::East),
            Some((0b1100_0000_0000_0000 << 1) & !FILE_A)
        );
        assert_eq!(
            checked_shift(bitboard, Direction::West),
            Some((0b1100_0000_0000_0000 >> 1) & !FILE_H)
        );
        assert_eq!(
            checked_shift(bitboard, Direction::NorthEast),
            Some((0b1100_0000_0000_0000 << 9) & !FILE_A)
        );
        assert_eq!(
            checked_shift(bitboard, Direction::NorthWest),
            Some((0b1100_0000_0000_0000 << 7) & !FILE_H)
        );
        assert_eq!(
            checked_shift(bitboard, Direction::SouthEast),
            Some((0b1100_0000_0000_0000 >> 7) & !FILE_A)
        );
        assert_eq!(
            checked_shift(bitboard, Direction::SouthWest),
            Some((0b1100_0000_0000_0000 >> 9) & !FILE_H)
        );
    }

    #[test]
    fn test_shift_checked_shift_equivalence() {
        let bitboard = 0b1100_0000_0000_0000;
        for dir in Direction::iter() {
            assert_eq!(shift(bitboard, dir), checked_shift(bitboard, dir).unwrap());
        }
    }
}
