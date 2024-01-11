use crate::moves::moves::Move;
use crate::types::bitboard::Bitboard;
use crate::types::pieces::Piece;
use crate::types::square::Square;

#[derive(Clone, PartialEq)]
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
