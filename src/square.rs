use std::ops;

use crate::{attack_boards::*, bitboard::Bitboard, moves::Direction};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Square(pub u8);

impl Square {
    /// Declaration of an invalid square used as the equivalent of null
    pub const INVALID: Square = Square(64);

    #[inline]
    pub fn shift(&self, dir: Direction) -> Option<Square> {
        let new_square = self.0 as i8 + dir as i8;
        if Square(new_square as u8).is_valid() {
            return Some(Square(new_square as u8));
        }
        None
    }

    pub fn dist(&self, sq: Square) -> u64 {
        let x1 = self.rank();
        let y1 = self.file();
        let x2 = sq.rank();
        let y2 = sq.file();
        let x_diff = x1.abs_diff(x2);
        let y_diff = y1.abs_diff(y2);
        x_diff.max(y_diff) as u64
    }

    #[inline]
    pub fn rank(&self) -> u8 {
        self.0 >> 3
    }

    #[inline]
    pub fn file(&self) -> u8 {
        self.0 & 0b111
    }

    #[inline]
    pub fn get_rank_bitboard(square: Square) -> Bitboard {
        let x = square.rank();
        match x {
            0 => Bitboard(RANK1),
            1 => Bitboard(RANK2),
            2 => Bitboard(RANK3),
            3 => Bitboard(RANK4),
            4 => Bitboard(RANK5),
            5 => Bitboard(RANK6),
            6 => Bitboard(RANK7),
            7 => Bitboard(RANK8),
            _ => panic!(),
        }
    }

    #[inline]
    pub fn get_file_bitboard(square: Square) -> Bitboard {
        let y = square.file();
        match y {
            0 => Bitboard(FILE_A),
            1 => Bitboard(FILE_B),
            2 => Bitboard(FILE_C),
            3 => Bitboard(FILE_D),
            4 => Bitboard(FILE_E),
            5 => Bitboard(FILE_F),
            6 => Bitboard(FILE_G),
            7 => Bitboard(FILE_H),
            _ => panic!(),
        }
    }

    #[inline]
    pub fn is_valid(&self) -> bool {
        self.0 < 64
    }

    pub fn new(sq: u8) -> Self {
        Square(sq)
    }

    #[inline]
    pub fn bitboard(&self) -> Bitboard {
        Bitboard(1 << self.0 as u8)
    }

    #[inline]
    pub fn pop_lsb(&mut self) -> u8 {
        let lsb = self.0 & self.0.wrapping_neg();
        self.0 ^= lsb;
        lsb.trailing_zeros() as u8
    }
}

pub struct SquareIter {
    current: u8,
    end: u8,
}

impl SquareIter {
    pub fn new() -> Self {
        SquareIter {
            current: 0,
            end: 64,
        }
    }
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
