use std::{fmt, ops};

use super::square::Square;
use crate::{attack_boards::FILES, chess_move::Direction};

#[derive(Clone, Copy, Default, Eq, PartialEq)]
pub struct Bitboard(pub u64);

impl Bitboard {
    pub const EMPTY: Self = Self(0);

    pub fn is_empty(self) -> bool {
        self == Bitboard::EMPTY
    }

    /// Returns the index of the lowest bit of a bitboard, and modifies the bitboard to exclude
    /// that bit
    pub fn pop_lsb(&mut self) -> Square {
        let lsb = self.lsb();
        self.0 &= self.0 - 1;
        lsb
    }

    pub const fn lsb(self) -> Square {
        unsafe { std::mem::transmute(self.0.trailing_zeros()) }
    }

    pub fn occupied(self, sq: Square) -> bool {
        self & sq.bitboard() != Self::EMPTY
    }

    pub fn empty(self, sq: Square) -> bool {
        !self.occupied(sq)
    }

    pub const fn count_bits(self) -> i32 {
        self.0.count_ones() as i32
    }

    /// Executes a shift without checking to ensure no information is lost. Only to be used when a
    /// shift has already been proven to be safe
    pub const fn shift(self, dir: Direction) -> Self {
        match dir {
            Direction::North => Self(self.0 << 8),
            Direction::NorthWest => Self((self.0 << 7) & !FILES[7].0),
            Direction::West => Self((self.0 >> 1) & !FILES[7].0),
            Direction::SouthWest => Self((self.0 >> 9) & !FILES[7].0),
            Direction::South => Self(self.0 >> 8),
            Direction::SouthEast => Self((self.0 >> 7) & !FILES[0].0),
            Direction::East => Self((self.0 << 1) & !FILES[0].0),
            Direction::NorthEast => Self((self.0 << 9) & !FILES[0].0),
        }
    }

    pub const fn contains(self, sq: Square) -> bool {
        self.and(sq.bitboard()).0 != 0
    }

    pub const fn and(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }
}

impl Iterator for Bitboard {
    type Item = Square;

    fn next(&mut self) -> Option<Self::Item> {
        if *self == Self::EMPTY {
            None
        } else {
            Some(self.pop_lsb())
        }
    }
}

impl fmt::Debug for Bitboard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f)?;
        for row in (0..8).rev() {
            for col in 0..8 {
                let index = row * 8 + col;
                let bit_is_set = self.0 & (1 << index) != 0;

                if bit_is_set {
                    write!(f, "X")?;
                } else {
                    write!(f, ".")?;
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
        Self(!self.0)
    }
}

// Macros from carp
macro_rules! impl_math_ops {
    ($($trait:ident::$fn:ident),*) => {
        $(impl std::ops::$trait for Bitboard {
            type Output = Self;

            fn $fn(self, other: Self) -> Self::Output {
                Self(std::ops::$trait::$fn(self.0, other.0))
            }
        })*
    };
}

impl_math_ops! {
    BitAnd::bitand,
    BitOr::bitor,
    BitXor::bitxor,
    Shl::shl,
    Shr::shr
}

macro_rules! impl_math_assign_ops {
    ($($trait:ident::$fn:ident),*) => {
        $(impl std::ops::$trait for Bitboard {

            fn $fn(&mut self, other: Self) {
                std::ops::$trait::$fn(&mut self.0, other.0)
            }
        })*
    };
}

impl_math_assign_ops! {
    BitAndAssign::bitand_assign,
    BitOrAssign::bitor_assign,
    BitXorAssign::bitxor_assign
}
