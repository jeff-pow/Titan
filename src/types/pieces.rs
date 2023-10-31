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
    #[inline(always)]
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

    #[inline(always)]
    pub fn idx(self) -> usize {
        self as usize
        // match self {
        //     PieceName::Pawn => 0,
        //     PieceName::Knight => 1,
        //     PieceName::Bishop => 2,
        //     PieceName::Rook => 3,
        //     PieceName::Queen => 4,
        //     PieceName::King => 5,
        // }
    }
}

impl From<usize> for PieceName {
    fn from(value: usize) -> Self {
        match value {
            0 => PieceName::King,
            1 => PieceName::Queen,
            2 => PieceName::Rook,
            3 => PieceName::Bishop,
            4 => PieceName::Knight,
            5 => PieceName::Pawn,
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
