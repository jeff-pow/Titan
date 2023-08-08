use crate::{
    attack_boards::*,
    moves::{self, coordinates, rank, Direction},
    square::Square,
};
use std::ops::{Shl, ShlAssign};

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
            let shifted = (bitboard.checked_shl(7).unwrap_or(0)) & !FILE_H.0;
            if bitboard.leading_zeros() >= 7 && shifted.trailing_zeros() >= 7 {
                Some(shifted)
            } else {
                None
            }
        }
        Direction::West => {
            let shifted = (bitboard.checked_shr(1).unwrap_or(0)) & !FILE_H.0;
            if shifted.leading_zeros() >= 7 && bitboard.trailing_zeros() >= 1 {
                Some(shifted)
            } else {
                None
            }
        }
        Direction::SouthWest => {
            let shifted = (bitboard.checked_shr(9).unwrap_or(0)) & !FILE_H.0;
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
            let shifted = (bitboard.checked_shr(7).unwrap_or(0)) & !FILE_A.0;
            if bitboard.trailing_zeros() >= 7 && shifted.leading_zeros() >= 7 {
                Some(shifted)
            } else {
                None
            }
        }
        Direction::East => {
            let shifted = (bitboard.checked_shl(1).unwrap_or(0)) & !FILE_A.0;
            if bitboard.leading_zeros() >= 1 && shifted.trailing_zeros() >= 7 {
                Some(shifted)
            } else {
                None
            }
        }
        Direction::NorthEast => {
            let shifted = (bitboard.checked_shl(9).unwrap_or(0)) & !FILE_A.0;
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
        Direction::NorthWest => (bitboard << 7) & !FILE_H.0,
        Direction::West => (bitboard >> 1) & !FILE_H.0,
        Direction::SouthWest => (bitboard >> 9) & !FILE_H.0,
        Direction::South => bitboard >> 8,
        Direction::SouthEast => (bitboard >> 7) & !FILE_A.0,
        Direction::East => (bitboard << 1) & !FILE_A.0,
        Direction::NorthEast => (bitboard << 9) & !FILE_A.0,
    }
}

#[inline]
pub fn pop_lsb(bb: &mut u64) -> u64 {
    let lsb = *bb & bb.wrapping_neg();
    *bb ^= lsb;
    lsb.trailing_zeros() as u64
}

#[inline]
pub fn get_rank_bitboard(square: u8) -> u64 {
    let x = moves::rank(square);
    match x {
        0 => RANK1.0,
        1 => RANK2.0,
        2 => RANK3.0,
        3 => RANK4.0,
        4 => RANK5.0,
        5 => RANK6.0,
        6 => RANK7.0,
        7 => RANK8.0,
        _ => panic!(),
    }
}

#[inline]
pub fn get_file_bitboard(square: u8) -> u64 {
    let y = moves::file(square);
    match y {
        0 => FILE_A.0,
        1 => FILE_B.0,
        2 => FILE_C.0,
        3 => FILE_D.0,
        4 => FILE_E.0,
        5 => FILE_F.0,
        6 => FILE_G.0,
        7 => FILE_H.0,
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
            (0b1100_0000_0000_0000 << 1) & !FILE_A.0
        );
        assert_eq!(
            shift(bitboard, Direction::West),
            (0b1100_0000_0000_0000 >> 1) & !FILE_H.0
        );
        assert_eq!(
            shift(bitboard, Direction::NorthEast),
            (0b1100_0000_0000_0000 << 9) & !FILE_A.0
        );
        assert_eq!(
            shift(bitboard, Direction::NorthWest),
            (0b1100_0000_0000_0000 << 7) & !FILE_H.0
        );
        assert_eq!(
            shift(bitboard, Direction::SouthEast),
            (0b1100_0000_0000_0000 >> 7) & !FILE_A.0
        );
        assert_eq!(
            shift(bitboard, Direction::SouthWest),
            (0b1100_0000_0000_0000 >> 9) & !FILE_H.0
        );
    }

    #[test]
    fn test_checked_shift_valid() {
        let bitboard = 0b1100_0000_0000_0000;
        assert_eq!(
            create_square_and_checked_shift(bitboard, Direction::North),
            Some(0b1100_0000_0000_0000 << 8)
        );
        assert_eq!(
            create_square_and_checked_shift(bitboard, Direction::South),
            Some(0b1100_0000_0000_0000 >> 8)
        );
        assert_eq!(
            create_square_and_checked_shift(bitboard, Direction::East),
            Some((0b1100_0000_0000_0000 << 1) & !FILE_A.0)
        );
        assert_eq!(
            create_square_and_checked_shift(bitboard, Direction::West),
            Some((0b1100_0000_0000_0000 >> 1) & !FILE_H.0)
        );
        assert_eq!(
            create_square_and_checked_shift(bitboard, Direction::NorthEast),
            Some((0b1100_0000_0000_0000 << 9) & !FILE_A.0)
        );
        assert_eq!(
            create_square_and_checked_shift(bitboard, Direction::NorthWest),
            Some((0b1100_0000_0000_0000 << 7) & !FILE_H.0)
        );
        assert_eq!(
            create_square_and_checked_shift(bitboard, Direction::SouthEast),
            Some((0b1100_0000_0000_0000 >> 7) & !FILE_A.0)
        );
        assert_eq!(
            create_square_and_checked_shift(bitboard, Direction::SouthWest),
            Some((0b1100_0000_0000_0000 >> 9) & !FILE_H.0)
        );
    }

    #[test]
    fn test_shift_create_square_and_checked_shift_equivalence() {
        let bitboard = 0b1100_0000_0000_0000;
        for dir in Direction::iter() {
            assert_eq!(
                shift(bitboard, dir),
                create_square_and_checked_shift(bitboard, dir).unwrap()
            );
        }
    }
}
