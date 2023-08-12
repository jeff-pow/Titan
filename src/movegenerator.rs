use crate::bitboard::Bitboard;
use crate::magics::{SMagic, BISHOP_M_SIZE, ROOK_M_SIZE};
use crate::moves::Move;

pub struct MoveGenerator {
    moves: Vec<Move>,
}
