use core::ops::{Index, IndexMut};
use std::fmt::Display;

use crate::moves::{
    attack_boards::{FILES, RANKS},
    moves::Direction,
};

use super::bitboard::Bitboard;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Square(pub u32);

pub const NUM_SQUARES: usize = 64;

impl Square {
    /// Function checks whether a shift is valid before executing it
    pub const fn checked_shift(self, dir: Direction) -> Option<Self> {
        let s = self.bitboard().shift(dir);
        if s.0 != 0 {
            Some(s.lsb())
        } else {
            None
        }
    }

    /// Function does not check a shift's validity before returning it. Only to be used when the
    /// shifts validity has already been proven valid elsewhere
    pub const fn shift(self, dir: Direction) -> Self {
        self.bitboard().shift(dir).lsb()
    }

    /// Returns the max dist of file or rank between two squares
    #[rustfmt::skip]
    pub const fn dist(self, other: Self) -> u32 {
        let this = self.file().abs_diff(other.file());
        let other = self.rank().abs_diff(other.rank());
        if this > other { this } else { other }
    }

    /// Rank is the horizontal component of the piece (y-coord)
    pub const fn rank(self) -> u32 {
        self.0 >> 3
    }

    pub const fn flip_vertical(self) -> Self {
        Self(self.0 ^ 56)
    }

    /// File is the vertical component of the piece (x-coord)
    pub const fn file(self) -> u32 {
        self.0 & 0b111
    }

    pub const fn idx(self) -> usize {
        self.0 as usize
    }

    pub const fn rank_bitboard(self) -> Bitboard {
        let x = self.rank();
        RANKS[x as usize]
    }

    pub const fn file_bitboard(self) -> Bitboard {
        let y = self.file();
        FILES[y as usize]
    }

    pub const fn is_valid(self) -> bool {
        self.0 < 64
    }

    pub const fn bitboard(self) -> Bitboard {
        Bitboard(1 << self.0)
    }

    pub fn iter() -> impl Iterator<Item = Self> {
        (0..64).map(Self)
    }

    pub const fn dir_to(self, other: Self) -> Option<Direction> {
        let rank_diff = other.rank() as i32 - self.rank() as i32;
        let file_diff = other.file() as i32 - self.file() as i32;

        match (rank_diff, file_diff) {
            (0, _) if file_diff > 0 => Some(Direction::East),
            (0, _) if file_diff < 0 => Some(Direction::West),
            (_, 0) if rank_diff > 0 => Some(Direction::North),
            (_, 0) if rank_diff < 0 => Some(Direction::South),
            (r, f) if r == f && r > 0 => Some(Direction::NorthEast),
            (r, f) if r == -f && r > 0 => Some(Direction::NorthWest),
            (r, f) if r == f && r < 0 => Some(Direction::SouthWest),
            (r, f) if r == -f && r < 0 => Some(Direction::SouthEast),
            _ => None,
        }
    }

    pub const A1: Self = Self(0);
    pub const B1: Self = Self(1);
    pub const C1: Self = Self(2);
    pub const D1: Self = Self(3);
    pub const E1: Self = Self(4);
    pub const F1: Self = Self(5);
    pub const G1: Self = Self(6);
    pub const H1: Self = Self(7);
    pub const A2: Self = Self(8);
    pub const B2: Self = Self(9);
    pub const C2: Self = Self(10);
    pub const D2: Self = Self(11);
    pub const E2: Self = Self(12);
    pub const F2: Self = Self(13);
    pub const G2: Self = Self(14);
    pub const H2: Self = Self(15);
    pub const A3: Self = Self(16);
    pub const B3: Self = Self(17);
    pub const C3: Self = Self(18);
    pub const D3: Self = Self(19);
    pub const E3: Self = Self(20);
    pub const F3: Self = Self(21);
    pub const G3: Self = Self(22);
    pub const H3: Self = Self(23);
    pub const A4: Self = Self(24);
    pub const B4: Self = Self(25);
    pub const C4: Self = Self(26);
    pub const D4: Self = Self(27);
    pub const E4: Self = Self(28);
    pub const F4: Self = Self(29);
    pub const G4: Self = Self(30);
    pub const H4: Self = Self(31);
    pub const A5: Self = Self(32);
    pub const B5: Self = Self(33);
    pub const C5: Self = Self(34);
    pub const D5: Self = Self(35);
    pub const E5: Self = Self(36);
    pub const F5: Self = Self(37);
    pub const G5: Self = Self(38);
    pub const H5: Self = Self(39);
    pub const A6: Self = Self(40);
    pub const B6: Self = Self(41);
    pub const C6: Self = Self(42);
    pub const D6: Self = Self(43);
    pub const E6: Self = Self(44);
    pub const F6: Self = Self(45);
    pub const G6: Self = Self(46);
    pub const H6: Self = Self(47);
    pub const A7: Self = Self(48);
    pub const B7: Self = Self(49);
    pub const C7: Self = Self(50);
    pub const D7: Self = Self(51);
    pub const E7: Self = Self(52);
    pub const F7: Self = Self(53);
    pub const G7: Self = Self(54);
    pub const H7: Self = Self(55);
    pub const A8: Self = Self(56);
    pub const B8: Self = Self(57);
    pub const C8: Self = Self(58);
    pub const D8: Self = Self(59);
    pub const E8: Self = Self(60);
    pub const F8: Self = Self(61);
    pub const G8: Self = Self(62);
    pub const H8: Self = Self(63);
}

#[rustfmt::skip]
pub static SQUARE_NAMES: [&str; 64] = [
    "a1", "b1", "c1", "d1", "e1", "f1", "g1", "h1", 
    "a2", "b2", "c2", "d2", "e2", "f2", "g2", "h2",
    "a3", "b3", "c3", "d3", "e3", "f3", "g3", "h3", 
    "a4", "b4", "c4", "d4", "e4", "f4", "g4", "h4",
    "a5", "b5", "c5", "d5", "e5", "f5", "g5", "h5", 
    "a6", "b6", "c6", "d6", "e6", "f6", "g6", "h6",
    "a7", "b7", "c7", "d7", "e7", "f7", "g7", "h7", 
    "a8", "b8", "c8", "d8", "e8", "f8", "g8", "h8",
];

impl<T, const N: usize> Index<Square> for [T; N] {
    type Output = T;

    fn index(&self, index: Square) -> &Self::Output {
        &self[index.0 as usize]
    }
}

impl<T, const N: usize> IndexMut<Square> for [T; N] {
    fn index_mut(&mut self, index: Square) -> &mut Self::Output {
        &mut self[index.0 as usize]
    }
}

impl Display for Square {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod square_test {
    use super::*;

    #[test]
    fn test_valid_shift() {
        let square = Square(35);
        let new_square = square.checked_shift(Direction::North);
        assert_eq!(new_square, Some(Square(43)));
    }

    #[test]
    fn test_invalid_shift() {
        let square = Square(63);
        let new_square = square.checked_shift(Direction::North);
        assert_eq!(new_square, None);
        let square = Square(47);
        let new_square = square.checked_shift(Direction::East);
        assert!(new_square.is_none());
    }
}
