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
pub struct Square(pub u8);

pub const NUM_SQUARES: usize = 64;

impl Square {
    /// Function checks whether a shift is valid before executing it
    #[inline(always)]
    pub fn checked_shift(&self, dir: Direction) -> Option<Square> {
        let current_file = self.file();
        let current_rank = self.rank();
        let (dir_x, dir_y) = dir.to_xy();
        if !(0..8).contains(&(current_file as i8 + dir_x)) {
            return None;
        }
        if !(0..8).contains(&(current_rank as i8 + dir_y)) {
            return None;
        }
        let new_index = (self.0 as i32 + dir as i32) as u8;

        if (0..64).contains(&new_index) {
            Some(Square(new_index))
        } else {
            None
        }
    }

    #[inline(always)]
    /// Function does not check a shift's validity before returning it. Only to be used when the
    /// shifts validity has already been proven valid elsewhere
    pub fn shift(&self, dir: Direction) -> Square {
        let new_square = self.0 as i8 + dir as i8;
        debug_assert!(Square(new_square as u8).is_valid());
        Square(new_square as u8)
    }

    #[inline(always)]
    /// Calculates the distance between two square
    pub fn dist(&self, sq: Square) -> u64 {
        let y1 = self.rank();
        let x1 = self.file();
        let y2 = sq.rank();
        let x2 = sq.file();
        let x_diff = x1.abs_diff(x2);
        let y_diff = y1.abs_diff(y2);
        x_diff.max(y_diff) as u64
    }

    /// Rank is the horizontal row of the piece (y-coord)
    #[inline(always)]
    pub fn rank(&self) -> u8 {
        self.0 >> 3
    }

    #[inline(always)]
    pub fn flip_vertical(&self) -> Square {
        Square(self.0 ^ 56)
    }

    /// File is the vertical column of the piece (x-coord)
    #[inline(always)]
    pub fn file(&self) -> u8 {
        self.0 & 0b111
    }

    #[inline(always)]
    pub fn idx(&self) -> usize {
        self.0 as usize
    }

    #[inline(always)]
    pub fn get_rank_bitboard(&self) -> Bitboard {
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

    #[inline(always)]
    pub fn get_file_bitboard(&self) -> Bitboard {
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

    #[inline(always)]
    pub fn is_valid(&self) -> bool {
        self.0 < 64
    }

    #[inline(always)]
    pub fn bitboard(&self) -> Bitboard {
        Bitboard(1 << self.0)
    }

    pub fn iter() -> SquareIter {
        SquareIter { current: 0, end: 63 }
    }
}

pub struct SquareIter {
    current: u8,
    end: u8,
}

impl Iterator for SquareIter {
    type Item = Square;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current <= self.end {
            let square = Square(self.current);
            self.current += 1;
            Some(square)
        } else {
            None
        }
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
