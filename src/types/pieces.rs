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
    White = 0,
    Black = 1,
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
    King = 0,
    Queen = 1,
    Rook = 2,
    Bishop = 3,
    Knight = 4,
    Pawn = 5,
}

#[inline(always)]
pub fn value(piece_name: Option<PieceName>) -> i32 {
    if let Some(p) = piece_name {
        return p.value();
    }
    0
}

impl PieceName {
    #[inline(always)]
    pub const fn value(&self) -> i32 {
        match self {
            PieceName::King => KING_PTS,
            PieceName::Queen => QUEEN_PTS,
            PieceName::Rook => ROOK_PTS,
            PieceName::Bishop => BISHOP_PTS,
            PieceName::Knight => KNIGHT_PTS,
            PieceName::Pawn => PAWN_PTS,
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
