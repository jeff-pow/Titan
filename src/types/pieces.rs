use crate::impl_index;
use std::{
    mem::transmute,
    ops::{self, Index, IndexMut},
};

impl_index!(Color);
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
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
    pub const fn idx(self) -> usize {
        match self {
            Color::White => 0,
            Color::Black => 1,
        }
    }

    fn from_u32(val: u32) -> Self {
        match val {
            0 => Color::White,
            1 => Color::Black,
            _ => panic!("Unexpected value"),
        }
    }

    pub fn iter() -> impl Iterator<Item = Self> {
        [Color::White, Color::Black].into_iter()
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

const PIECE_VALUES: [i32; 6] = [100, 350, 350, 525, 1000, 0];
pub const NUM_PIECES: usize = 6;

impl_index!(PieceName);
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum PieceName {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
    None = 7,
}

impl PieceName {
    pub fn value(self) -> i32 {
        PIECE_VALUES[self]
    }

    pub(crate) fn idx(self) -> usize {
        self as usize
    }

    fn from_u32(val: u32) -> Self {
        assert!((0..6).contains(&val) || val == 7);
        unsafe { transmute(val as u8) }
    }

    pub fn iter() -> impl Iterator<Item = Self> {
        [
            PieceName::Pawn,
            PieceName::Knight,
            PieceName::Bishop,
            PieceName::Rook,
            PieceName::Queen,
            PieceName::King,
        ]
        .into_iter()
    }
}

impl_index!(Piece);
#[derive(Eq, Copy, Clone, PartialEq, Debug)]
#[repr(u8)]
pub enum Piece {
    WhitePawn,
    BlackPawn,

    WhiteKnight,
    BlackKnight,

    WhiteBishop,
    BlackBishop,

    WhiteRook,
    BlackRook,

    WhiteQueen,
    BlackQueen,

    WhiteKing,
    BlackKing,

    None = 15,
}

impl Piece {
    pub(crate) fn new(name: PieceName, color: Color) -> Self {
        unsafe { transmute(((name as u32) << 1 | color as u32) as u8) }
    }

    pub(crate) fn name(self) -> PieceName {
        // PieceName::from_u32(self as u32 >> 1)
        PieceName::from_u32(self as u32 >> 1)
    }

    pub(crate) fn value(self) -> i32 {
        self.name().value()
    }

    pub(crate) fn color(self) -> Color {
        Color::from_u32(self as u32 & 0b1)
    }

    pub(crate) fn from_u32(x: u32) -> Self {
        assert!((0..12).contains(&x) || x == 15);
        unsafe { transmute(x as u8) }
    }

    pub(crate) fn char(self) -> String {
        let c = match self.name() {
            PieceName::Pawn => "P",
            PieceName::Knight => "N",
            PieceName::Bishop => "B",
            PieceName::Rook => "R",
            PieceName::Queen => "Q",
            PieceName::King => "K",
            PieceName::None => "_",
        };
        if self.color() == Color::Black {
            c.to_ascii_lowercase()
        } else {
            c.to_string()
        }
    }
}

#[cfg(test)]
mod piece_tests {
    use super::*;

    #[test]
    fn test_new_piece() {
        assert_eq!(Piece::new(PieceName::Pawn, Color::White), Piece::WhitePawn);
        assert_eq!(Piece::new(PieceName::Knight, Color::White), Piece::WhiteKnight);
        assert_eq!(Piece::new(PieceName::Bishop, Color::White), Piece::WhiteBishop);
        assert_eq!(Piece::new(PieceName::Rook, Color::White), Piece::WhiteRook);
        assert_eq!(Piece::new(PieceName::Queen, Color::White), Piece::WhiteQueen);
        assert_eq!(Piece::new(PieceName::King, Color::White), Piece::WhiteKing);

        assert_eq!(Piece::new(PieceName::Pawn, Color::Black), Piece::BlackPawn);
        assert_eq!(Piece::new(PieceName::Knight, Color::Black), Piece::BlackKnight);
        assert_eq!(Piece::new(PieceName::Bishop, Color::Black), Piece::BlackBishop);
        assert_eq!(Piece::new(PieceName::Rook, Color::Black), Piece::BlackRook);
        assert_eq!(Piece::new(PieceName::Queen, Color::Black), Piece::BlackQueen);
        assert_eq!(Piece::new(PieceName::King, Color::Black), Piece::BlackKing);

        for color in [Color::White, Color::Black] {
            for name in [
                PieceName::Pawn,
                PieceName::Knight,
                PieceName::Bishop,
                PieceName::Rook,
                PieceName::Queen,
                PieceName::King,
            ] {
                let piece = Piece::new(name, color);
                assert_eq!(piece.name(), name);
                assert_eq!(piece.color(), color);
            }
        }
    }

    #[test]
    fn test_piece_name_color() {
        let piece = Piece::new(PieceName::Rook, Color::Black);
        assert_eq!(piece.name(), PieceName::Rook);
        assert_eq!(piece.color(), Color::Black);
    }

    #[test]
    fn test_piece_values() {
        assert_eq!(PieceName::Pawn.value(), 100);
        assert_eq!(PieceName::Knight.value(), 350);
        assert_eq!(PieceName::Bishop.value(), 350);
        assert_eq!(PieceName::Rook.value(), 525);
        assert_eq!(PieceName::Queen.value(), 1000);
        assert_eq!(PieceName::King.value(), 0);
    }

    #[test]
    fn test_piece_index_conversion() {
        assert_eq!(PieceName::from_u32(0), PieceName::Pawn);
        assert_eq!(PieceName::from_u32(1), PieceName::Knight);
        assert_eq!(PieceName::from_u32(2), PieceName::Bishop);
        assert_eq!(PieceName::from_u32(3), PieceName::Rook);
        assert_eq!(PieceName::from_u32(4), PieceName::Queen);
        assert_eq!(PieceName::from_u32(5), PieceName::King);
        assert_eq!(PieceName::from_u32(7), PieceName::None);
    }

    #[test]
    fn test_piece_color_conversion() {
        assert_eq!(Color::from_u32(0), Color::White);
        assert_eq!(Color::from_u32(1), Color::Black);
    }
}
