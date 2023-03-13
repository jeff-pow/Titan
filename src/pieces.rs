pub const INFINITY: i32 = 100000000;
pub const KING_PTS: i32 = INFINITY;
pub const QUEEN_PTS: i32 = 900;
pub const ROOK_PTS: i32 = 500;
pub const BISHOP_PTS: i32 = 300;
pub const KNIGHT_PTS: i32 = 300;
pub const PAWN_PTS: i32 = 100;


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
    pub fn value(&self) -> i32 {
        match self.piece_name {
            PieceName::King => KING_PTS as i32,
            PieceName::Queen => QUEEN_PTS,
            PieceName::Rook => ROOK_PTS,
            PieceName::Bishop => BISHOP_PTS,
            PieceName::Knight => KNIGHT_PTS,
            PieceName::Pawn => PAWN_PTS,
        }
    }
}
