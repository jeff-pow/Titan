use core::ops::{Index, IndexMut};

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
    pub fn checked_shift(self, dir: Direction) -> Option<Self> {
        let s = self.bitboard().shift(dir);
        if s != Bitboard::EMPTY {
            Some(s.get_lsb())
        } else {
            None
        }
        // (s != Bitboard::EMPTY).then_some(s.get_lsb())
    }

    /// Function does not check a shift's validity before returning it. Only to be used when the
    /// shifts validity has already been proven valid elsewhere
    pub const fn shift(self, dir: Direction) -> Self {
        self.bitboard().shift(dir).get_lsb()
    }

    /// Returns the max dist of file or rank between two squares
    #[rustfmt::skip]
    pub const fn dist(self, other: Square) -> u32 {
        let this = self.file().abs_diff(other.file());
        let other = self.rank().abs_diff(other.rank());
        if this > other { this } else { other }
    }

    /// Rank is the horizontal row of the piece (y-coord)
    pub const fn rank(self) -> u32 {
        self.0 >> 3
    }

    pub fn flip_vertical(self) -> Self {
        Self(self.0 ^ 56)
    }

    /// File is the vertical column of the piece (x-coord)
    pub const fn file(self) -> u32 {
        self.0 & 0b111
    }

    pub const fn idx(self) -> usize {
        self.0 as usize
    }

    pub fn get_rank_bitboard(self) -> Bitboard {
        let x = self.rank();
        RANKS[x as usize]
    }

    pub fn get_file_bitboard(self) -> Bitboard {
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
}

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

impl ToString for Square {
    fn to_string(&self) -> String {
        self.0.to_string()
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
