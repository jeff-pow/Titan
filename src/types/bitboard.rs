use std::{fmt, ops};

use crate::moves::{attack_boards::FILES, moves::Direction};

use super::square::Square;

#[derive(Clone, Copy, Default, Eq, PartialEq)]
pub struct Bitboard(pub u64);

impl Bitboard {
    pub const EMPTY: Bitboard = Bitboard(0);

    /// Returns the index of the lowest bit of a bitboard, and modifies the bitboard to exclude
    /// that bit
    pub fn pop_lsb(&mut self) -> Square {
        let lsb = self.get_lsb();
        self.0 &= self.0 - 1;
        lsb
    }

    pub const fn get_lsb(self) -> Square {
        unsafe { std::mem::transmute(self.0.trailing_zeros()) }
    }

    pub fn occupied(self, sq: Square) -> bool {
        self & sq.bitboard() != Bitboard::EMPTY
    }

    pub fn empty(self, sq: Square) -> bool {
        !self.occupied(sq)
    }

    pub fn count_bits(self) -> u32 {
        self.0.count_ones()
    }

    /// Executes a shift without checking to ensure no information is lost. Only to be used when a
    /// shift has already been proven to be safe
    pub const fn shift(self, dir: Direction) -> Bitboard {
        match dir {
            Direction::North => Bitboard(self.0 << 8),
            Direction::NorthWest => Bitboard((self.0 << 7) & !FILES[7].0),
            Direction::West => Bitboard((self.0 >> 1) & !FILES[7].0),
            Direction::SouthWest => Bitboard((self.0 >> 9) & !FILES[7].0),
            Direction::South => Bitboard(self.0 >> 8),
            Direction::SouthEast => Bitboard((self.0 >> 7) & !FILES[0].0),
            Direction::East => Bitboard((self.0 << 1) & !FILES[0].0),
            Direction::NorthEast => Bitboard((self.0 << 9) & !FILES[0].0),
        }
    }
}

impl Iterator for Bitboard {
    type Item = Square;

    fn next(&mut self) -> Option<Self::Item> {
        if *self == Bitboard::EMPTY {
            None
        } else {
            Some({
                let lsb = self.get_lsb();
                *self ^= lsb.bitboard();
                lsb
            })
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
        Bitboard(!self.0)
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
