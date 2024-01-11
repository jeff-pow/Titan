use crate::{
    moves::moves::Move,
    types::{bitboard::Bitboard, pieces::Piece, square::Square},
};

#[derive(Clone, Copy, PartialEq)]
pub(super) struct Undo {
    pub capture: Piece,
    pub castling_rights: u32,
    pub en_passant_square: Option<Square>,
    pub half_moves: usize,
    pub in_check: bool,
    pub zobrist_hash: u64,
    pub m: Move,
    pub threats: Bitboard,
}
