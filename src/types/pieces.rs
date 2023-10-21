use std::ops;

use strum_macros::EnumIter;

pub const KING_PTS: i32 = 0;
pub const QUEEN_PTS: i32 = 1000;
pub const ROOK_PTS: i32 = 525;
pub const BISHOP_PTS: i32 = 350;
pub const KNIGHT_PTS: i32 = 350;
pub const PAWN_PTS: i32 = 100;
pub const NUM_PIECES: usize = 6;

#[derive(EnumIter, Debug, Copy, Clone, PartialEq, Eq)]
pub enum Color {
    White,
    Black,
}

impl Color {
    #[inline(always)]
    pub fn idx(&self) -> usize {
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

#[derive(Debug, EnumIter, Copy, Clone, PartialEq, Eq)]
pub enum PieceName {
    King,
    Queen,
    Rook,
    Bishop,
    Knight,
    Pawn,
}

impl PieceName {
    #[inline(always)]
    pub fn value(&self) -> i32 {
        match self {
            PieceName::King => KING_PTS,
            PieceName::Queen => QUEEN_PTS,
            PieceName::Rook => ROOK_PTS,
            PieceName::Bishop => BISHOP_PTS,
            PieceName::Knight => KNIGHT_PTS,
            PieceName::Pawn => PAWN_PTS,
        }
    }

    #[inline(always)]
    pub fn idx(&self) -> usize {
        match self {
            PieceName::King => 0,
            PieceName::Queen => 1,
            PieceName::Rook => 2,
            PieceName::Bishop => 3,
            PieceName::Knight => 4,
            PieceName::Pawn => 5,
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

    #[inline(always)]
    pub fn value(&self) -> i32 {
        self.name.value()
    }
}
