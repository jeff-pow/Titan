use crate::search::INFINITY;

pub const KING_PTS: i32 = INFINITY;
pub const QUEEN_PTS: i32 = 1000;
pub const ROOK_PTS: i32 = 525;
pub const BISHOP_PTS: i32 = 350;
pub const KNIGHT_PTS: i32 = 350;
pub const PAWN_PTS: i32 = 100;
pub const NUM_PIECES: usize = 6;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Piece {
    pub current_square: i8,
    pub color: Color,
    pub piece_name: PieceName,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(usize)]
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

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(usize)]
pub enum PieceName {
    King = 0,
    Queen = 1,
    Rook = 2,
    Bishop = 3,
    Knight = 4,
    Pawn = 5,
}

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

impl Piece {
    #[allow(dead_code)]
    pub fn new(color: Color, piece_name: PieceName, starting_square: i8) -> Self {
        Self {
            current_square: starting_square,
            color,
            piece_name,
        }
    }
}
