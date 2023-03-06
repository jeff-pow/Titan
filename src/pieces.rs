use std::f64::INFINITY;

pub const KING_PTS: f64 = INFINITY;
pub const QUEEN_PTS: f64 = 9.;
pub const ROOK_PTS: f64 = 5.;
pub const BISHOP_PTS: f64 = 3.;
pub const KNIGHT_PTS: f64 = 3.;
pub const PAWN_PTS: f64 = 1.;


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Piece {
    pub current_square: i8,
    pub color: Color,
    pub piece_name: PieceName,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    White,
    Black,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum PieceName {
    King,
    Queen,
    Rook,
    Bishop,
    Knight,
    Pawn,
}

impl Piece {
    pub fn new(color: Color, piece_name: PieceName, starting_square: i8) -> Self {
        Self {
            current_square: starting_square,
            color,
            piece_name,
        }
    }
}

pub fn get_piece_value(piece: &Piece) -> f64 {
    match piece.piece_name {
        PieceName::King => KING_PTS,
        PieceName::Queen => QUEEN_PTS,
        PieceName::Rook => ROOK_PTS,
        PieceName::Bishop => BISHOP_PTS,
        PieceName::Knight => KNIGHT_PTS,
        PieceName::Pawn => PAWN_PTS,
    }
}
