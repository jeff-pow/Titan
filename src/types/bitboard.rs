use std::{fmt, ops};

use crate::moves::{
    attack_boards::{FILE_A, FILE_H},
    moves::Direction,
};

use super::square::Square;

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Bitboard(pub u64);

impl Bitboard {
    pub const EMPTY: Bitboard = Bitboard(0);

    #[inline(always)]
    /// Returns the index of the lowest bit of a bitboard, and modifies the bitboard to exclude
    /// that bit
    pub fn pop_lsb(&mut self) -> Square {
        let lsb = self.0 & self.0.wrapping_neg();
        self.0 ^= lsb;
        Square(lsb.trailing_zeros() as u8)
    }

    #[inline(always)]
    pub fn get_lsb(&self) -> Square {
        let lsb = self.0 & self.0.wrapping_neg();
        Square(lsb.trailing_zeros() as u8)
    }

    #[inline(always)]
    pub fn square_occupied(&self, sq: Square) -> bool {
        debug_assert!(sq.is_valid());
        self.0 & (1 << sq.0) != 0
    }

    #[inline(always)]
    pub fn square_is_empty(&self, sq: Square) -> bool {
        !self.square_occupied(sq)
    }

    #[inline(always)]
    /// Checks a bitboard shift to ensure no information is lost and then executes the shift
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
        };
        result.map(Bitboard)
    }

    /// Executes a shift without checking to ensure no information is list. Only to be used when a
    /// shift has already been proven to be safe
    #[inline(always)]
    pub fn shift(&self, dir: Direction) -> Bitboard {
        match dir {
            Direction::North => Bitboard(self.0 << 8),
            Direction::NorthWest => Bitboard((self.0 << 7) & !FILE_H.0),
            Direction::West => Bitboard((self.0 >> 1) & !FILE_H.0),
            Direction::SouthWest => Bitboard((self.0 >> 9) & !FILE_H.0),
            Direction::South => Bitboard(self.0 >> 8),
            Direction::SouthEast => Bitboard((self.0 >> 7) & !FILE_A.0),
            Direction::East => Bitboard((self.0 << 1) & !FILE_A.0),
            Direction::NorthEast => Bitboard((self.0 << 9) & !FILE_A.0),
        }
    }
}

impl Iterator for Bitboard {
    type Item = Square;

    fn next(&mut self) -> Option<Self::Item> {
        if *self == Bitboard::EMPTY {
            None
        } else {
            Some(self.pop_lsb())
        }
    }
}

impl fmt::Debug for Bitboard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for row in (0..8).rev() {
            for col in 0..8 {
                let index = row * 8 + col;
                let bit_is_set = self.0 & (1 << index) != 0;

                if bit_is_set {
                    write!(f, "1")?;
                } else {
                    write!(f, "0")?;
                }

                if col < 7 {
                    write!(f, " ")?;
                }
            }

            if row > 0 {
                writeln!(f)?;
            }
        }

        Ok(())
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
impl ops::BitAndAssign for Bitboard {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0
    }
}
impl ops::BitOr for Bitboard {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Bitboard(self.0 | rhs.0)
    }
}
impl ops::BitOrAssign for Bitboard {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}
impl ops::BitXorAssign for Bitboard {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0
    }
}
impl ops::BitXor for Bitboard {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Bitboard(self.0 ^ rhs.0)
    }
}
impl std::cmp::PartialOrd for Bitboard {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}
impl std::ops::Shl for Bitboard {
    type Output = Self;

    fn shl(self, rhs: Self) -> Self::Output {
        Bitboard(self.0.shl(rhs.0))
    }
}
impl std::ops::Shr for Bitboard {
    type Output = Self;

    fn shr(self, rhs: Self) -> Self::Output {
        Bitboard(self.0.shr(rhs.0))
    }
}
