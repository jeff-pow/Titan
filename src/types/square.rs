use std::ops;

use crate::moves::{
    attack_boards::{
        FILE_A, FILE_B, FILE_C, FILE_D, FILE_E, FILE_F, FILE_G, FILE_H, RANK1, RANK2, RANK3, RANK4, RANK5, RANK6,
        RANK7, RANK8,
    },
    moves::Direction,
};

use super::bitboard::Bitboard;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Square(pub u32);

pub const NUM_SQUARES: usize = 64;

impl Square {
    /// Function checks whether a shift is valid before executing it
    pub fn checked_shift(self, dir: Direction) -> Option<Square> {
        let current_file = self.file();
        let current_rank = self.rank();
        let (dir_x, dir_y) = dir.to_xy();
        if !(0..8).contains(&(current_file as i32 + dir_x)) {
            return None;
        }
        if !(0..8).contains(&(current_rank as i32 + dir_y)) {
            return None;
        }
        let new_index = (self.0 as i32 + dir as i32) as u32;

        if (0..64).contains(&new_index) {
            Some(Square(new_index))
        } else {
            None
        }
    }

    /// Function does not check a shift's validity before returning it. Only to be used when the
    /// shifts validity has already been proven valid elsewhere
    pub fn shift(self, dir: Direction) -> Square {
        let new_square = self.0 as i32 + dir as i32;
        Square(new_square as u32)
    }

    /// Calculates the distance between two squares
    #[rustfmt::skip]
    pub fn dist(self, other: Square) -> u32 {
        self.file().abs_diff(other.file())
            .max(self.rank().abs_diff(other.rank()))
    }

    /// Rank is the horizontal row of the piece (y-coord)
    pub fn rank(self) -> u32 {
        self.0 >> 3
    }

    pub fn flip_vertical(self) -> Square {
        Square(self.0 ^ 56)
    }

    /// File is the vertical column of the piece (x-coord)
    pub fn file(self) -> u32 {
        self.0 & 0b111
    }

    pub fn idx(self) -> usize {
        self.0 as usize
    }

    pub fn get_rank_bitboard(self) -> Bitboard {
        let x = self.rank();
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

    pub fn get_file_bitboard(self) -> Bitboard {
        let y = self.file();
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

    pub fn is_valid(self) -> bool {
        self.0 < 64
    }

    pub fn bitboard(self) -> Bitboard {
        Bitboard(1 << self.0)
    }

    pub fn iter() -> impl Iterator<Item = Self> {
        (0..64).map(Self)
    }
}

impl ops::BitAnd for Square {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Square(self.0 & rhs.0)
    }
}

impl ops::BitOr for Square {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Square(self.0 | rhs.0)
    }
}

impl ops::BitXor for Square {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self::Output {
        Square(self.0 ^ rhs.0)
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
