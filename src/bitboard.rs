use std::ops;

use crate::{
    attack_boards::{FILE_A, FILE_H},
    moves::Direction,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Bitboard(pub u64);

impl Bitboard {
    pub fn new(bitboard: u64) -> Self {
        Bitboard(bitboard)
    }

    pub fn empty() -> Self {
        Self::new(0)
    }

    #[inline]
    pub fn pop_lsb(&mut self) -> u64 {
        let lsb = self.0 & self.0.wrapping_neg();
        self.0 ^= lsb;
        lsb.trailing_zeros() as u64
    }

    #[inline]
    pub fn checked_shift(&self, dir: Direction) -> Option<Bitboard> {
        let bitboard = self.0.max(1);
        let result = match dir {
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
        };
        result.map(Bitboard)
    }

    #[inline]
    pub fn shift(&self, dir: Direction) -> Bitboard {
        match dir {
            Direction::North => Bitboard(self.0 << 8),
            Direction::NorthWest => Bitboard((self.0 << 7) & !FILE_H),
            Direction::West => Bitboard((self.0 >> 1) & !FILE_H),
            Direction::SouthWest => Bitboard((self.0 >> 9) & !FILE_H),
            Direction::South => Bitboard(self.0 >> 8),
            Direction::SouthEast => Bitboard((self.0 >> 7) & !FILE_A),
            Direction::East => Bitboard((self.0 << 1) & !FILE_A),
            Direction::NorthEast => Bitboard((self.0 << 9) & !FILE_A),
        }
    }
}

impl ops::Not for Bitboard {
    type Output = Self;

    fn not(self) -> Self::Output {
        Bitboard(!self.0)
    }
}

impl ops::BitAnd for Bitboard {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Bitboard(self.0 & rhs.0)
    }
}
impl ops::BitOr for Bitboard {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Bitboard(self.0 | rhs.0)
    }
}
impl ops::BitXor for Bitboard {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Bitboard(self.0 ^ rhs.0)
    }
}
