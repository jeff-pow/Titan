use strum_macros::EnumIter;

use crate::search::alpha_beta::INFINITY;

pub const KING_PTS: i32 = INFINITY;
pub const QUEEN_PTS: i32 = 1000;
pub const ROOK_PTS: i32 = 525;
pub const BISHOP_PTS: i32 = 350;
pub const KNIGHT_PTS: i32 = 350;
pub const PAWN_PTS: i32 = 100;
pub const NUM_PIECES: usize = 6;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    White = 0,
    Black = 1,
}

impl Color {
    pub fn opposite(&self) -> Color {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}

pub fn opposite_color(color: Color) -> Color {
    match color {
        Color::White => Color::Black,
        Color::Black => Color::White,
    }
}

#[derive(Debug, EnumIter, Copy, Clone, PartialEq, Eq)]
#[repr(usize)]
pub enum PieceName {
    King = 0,
    Queen = 1,
    Rook = 2,
    Bishop = 3,
    Knight = 4,
    Pawn = 5,
}

#[inline(always)]
pub fn piece_value(piece_name: PieceName) -> i32 {
    match piece_name {
        PieceName::King => KING_PTS,
        PieceName::Queen => QUEEN_PTS,
        PieceName::Rook => ROOK_PTS,
        PieceName::Bishop => BISHOP_PTS,
        PieceName::Knight => KNIGHT_PTS,
        PieceName::Pawn => PAWN_PTS,
    }
}
