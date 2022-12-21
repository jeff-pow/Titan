use std::f32::INFINITY;

pub const KING: u8 = 1;
pub const KING_PTS: f32 = INFINITY;

pub const QUEEN: u8 = 2;
pub const QUEEN_PTS: f32 = 9.;

pub const ROOK: u8 = 4;
pub const ROOK_PTS: f32 = 5.;

pub const BISHOP: u8 = 8;
pub const BISHOP_PTS: f32 = 3.;

pub const KNIGHT: u8 = 16;
pub const KNIGHT_PTS: f32 = 3.;

pub const PAWN: u8 = 32;
pub const PAWN_PTS: f32 = 1.;

pub const WHITE: u8 = 64;
pub const BLACK: u8 = 128;
/** 
 * Pieces have an index based off of what piece they are. A pieces bits can be read to determine 
 * piece type. Black pieces will always be > 16, and white pieces will always be less than 8.
 * These values are stored in the constants with the name of the piece. The constants labeled 
 * <piece name>_PTS are used to store values of pieces for the engine to decide how to value pieces
 */
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Piece {
    pub current_square: u8,
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
    pub fn new(color: Color, piece_name: PieceName, starting_square: u8) -> Self {
        Self { 
            current_square: starting_square,
            color: color,
            piece_name: piece_name,
        }
    }
    pub fn can_castle(&self) -> bool {
        todo!();
    }
    pub fn change_square(&mut self, new_idx: u8) {
        self.current_square = new_idx;
    }
}
