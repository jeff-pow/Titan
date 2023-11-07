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

    /// Returns the index of the lowest bit of a bitboard, and modifies the bitboard to exclude
    /// that bit
    pub fn pop_lsb(&mut self) -> Square {
        let lsb = self.0 & self.0.wrapping_neg();
        self.0 ^= lsb;
        Square(lsb.trailing_zeros() as u8)
    }

    pub fn get_lsb(self) -> Square {
        let lsb = self.0 & self.0.wrapping_neg();
        Square(lsb.trailing_zeros() as u8)
    }

    pub fn occupied(self, sq: Square) -> bool {
        debug_assert!(sq.is_valid());
        // self.0 & (1 << sq.0) != 0
        self & sq.bitboard() != Bitboard::EMPTY
    }

    pub fn empty(self, sq: Square) -> bool {
        !self.occupied(sq)
    }

    pub fn count_bits(self) -> i32 {
        self.0.count_ones().try_into().expect("Valid conversion")
    }

    /// Executes a shift without checking to ensure no information is list. Only to be used when a
    /// shift has already been proven to be safe
    pub fn shift(self, dir: Direction) -> Bitboard {
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
