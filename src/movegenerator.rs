use crate::bitboard::Bitboard;
use crate::pleco_magics::{SMagic, BISHOP_M_SIZE, ROOK_M_SIZE};

pub struct MoveGenerator {
    pub knight_table: [Bitboard; 64],
    pub king_table: [Bitboard; 64],
    pub pawn_table: [[Bitboard; 64]; 2],
    rook_magics: [SMagic; 64],
    pub rook_table: [u64; ROOK_M_SIZE],
    bishop_magics: [SMagic; 64],
    pub bishop_table: [u64; BISHOP_M_SIZE],
}
