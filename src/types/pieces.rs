use crate::impl_index;
use std::ops::{self, Index, IndexMut};

use strum_macros::EnumIter;

pub const KING_PTS: i32 = 0;
pub const QUEEN_PTS: i32 = 1000;
pub const ROOK_PTS: i32 = 525;
pub const BISHOP_PTS: i32 = 350;
pub const KNIGHT_PTS: i32 = 350;
pub const PAWN_PTS: i32 = 100;
pub const NUM_PIECES: usize = 6;

impl_index!(Color);
#[derive(EnumIter, Debug, Copy, Clone, PartialEq, Eq)]
pub enum Color {
    White,
    Black,
}

#[macro_export]
macro_rules! impl_index {
    ($enum_name:ident) => {
        impl<T, const N: usize> Index<$enum_name> for [T; N] {
            type Output = T;

            fn index(&self, index: $enum_name) -> &Self::Output {
                &self[index as usize]
            }
        }

        impl<T, const N: usize> IndexMut<$enum_name> for [T; N] {
            fn index_mut(&mut self, index: $enum_name) -> &mut Self::Output {
                &mut self[index as usize]
            }
        }
    };
}

impl Color {
    pub fn idx(self) -> usize {
        match self {
            Color::White => 0,
            Color::Black => 1,
        }
    }
}

impl ops::Not for Color {
    type Output = Color;
    fn not(self) -> Self::Output {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}

impl From<usize> for Color {
    fn from(value: usize) -> Self {
        match value {
            0 => Color::White,
            1 => Color::Black,
            _ => panic!("Invalid color index"),
        }
    }
}

impl_index!(PieceName);
#[derive(Debug, EnumIter, Copy, Clone, PartialEq, Eq)]
pub enum PieceName {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

impl PieceName {
    pub fn value(self) -> i32 {
        match self {
            PieceName::King => KING_PTS,
            PieceName::Queen => QUEEN_PTS,
            PieceName::Rook => ROOK_PTS,
            PieceName::Bishop => BISHOP_PTS,
            PieceName::Knight => KNIGHT_PTS,
            PieceName::Pawn => PAWN_PTS,
        }
    }

    pub fn idx(self) -> usize {
        self as usize
    }
}

impl From<usize> for PieceName {
    fn from(value: usize) -> Self {
        match value {
            5 => PieceName::King,
            4 => PieceName::Queen,
            3 => PieceName::Rook,
            2 => PieceName::Bishop,
            1 => PieceName::Knight,
            0 => PieceName::Pawn,
            _ => panic!("Invalid piece index"),
        }
    }
}

#[derive(Eq, Copy, Clone, PartialEq, Debug)]
pub struct Piece {
    pub name: PieceName,
    pub color: Color,
}

impl Piece {
    pub fn new(name: PieceName, color: Color) -> Self {
        Self { name, color }
    }
}
