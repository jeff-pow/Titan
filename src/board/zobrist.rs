// use std::collections::HashMap;

use lazy_static::lazy_static;
use strum::IntoEnumIterator;

use crate::{
    board::board::Board,
    moves::{magics::Rng, moves::Castle},
    types::pieces::{Color, PieceName},
};

pub struct Zobrist {
    pub piece_square_hashes: [[[u64; 64]; 6]; 2],
    pub turn_hash: u64,
    pub castling: [u64; 4],
    // 64 squares plus an invalid square
    // Don't bother figuring out invalid enpassant squares, literally not worth the squeeze
    pub en_passant: [u64; 64],
}

lazy_static! {
    pub static ref ZOBRIST: Zobrist = Zobrist::default();
}

impl Default for Zobrist {
    fn default() -> Self {
        let mut rng = Rng::default();
        let turn_hash = rng.next_u64();
        let mut piece_square_hashes = [[[0; 64]; 6]; 2];
        piece_square_hashes
            .iter_mut()
            .flatten()
            .flatten()
            .for_each(|x| *x = rng.next_u64());
        let mut castling = [0; 4];
        castling.iter_mut().for_each(|x| *x = rng.next_u64());
        let mut en_passant = [0; 64];
        en_passant.iter_mut().for_each(|x| *x = rng.next_u64());
        Self {
            turn_hash,
            piece_square_hashes,
            castling,
            en_passant,
        }
    }
}

impl Board {
    /// Provides a hash for the board eval to be placed into a transposition table
    pub(crate) fn generate_hash(&self) -> u64 {
        let mut hash = 0;

        for color in Color::iter() {
            for piece in PieceName::iter() {
                let occupancies = self.bitboard(color, piece);
                for sq in occupancies {
                    hash ^= ZOBRIST.piece_square_hashes[color as usize][piece as usize][sq.idx()]
                }
            }
        }

        if let Some(x) = self.en_passant_square { hash ^= ZOBRIST.en_passant[x.idx()] }

        if self.can_castle(Castle::WhiteKing) {
            hash ^= ZOBRIST.castling[0];
        }
        if self.can_castle(Castle::WhiteQueen) {
            hash ^= ZOBRIST.castling[1];
        }
        if self.can_castle(Castle::BlackKing) {
            hash ^= ZOBRIST.castling[2];
        }
        if self.can_castle(Castle::BlackQueen) {
            hash ^= ZOBRIST.castling[3];
        }

        if self.to_move == Color::Black {
            hash ^= ZOBRIST.turn_hash;
        }

        hash
    }
}

#[cfg(test)]
mod hashing_test {
    use crate::board::fen;

    #[test]
    fn test_hashing() {
        let board1 = fen::build_board(fen::STARTING_FEN);
        let board2 = fen::build_board("4r3/4k3/8/4K3/8/8/8/8 w - - 0 1");
        let board3 = fen::build_board("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        assert_ne!(board1.generate_hash(), board2.generate_hash());
        assert_eq!(board1.generate_hash(), board3.generate_hash());
    }
}
