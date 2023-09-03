// use std::collections::HashMap;

use strum::IntoEnumIterator;

use crate::{
    board::board::Board,
    moves::magics::Rng,
    types::{
        bitboard::Bitboard,
        pieces::{Color, PieceName},
    },
};

pub struct Zobrist {
    pub piece_square_hashes: [[[u64; 64]; 6]; 2],
    pub turn_hash: u64,
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
        Self {
            turn_hash,
            piece_square_hashes,
        }
    }
}

impl Board {
    /// Provides a hash for the board eval to be placed into a transposition table
    #[inline(always)]
    pub(crate) fn generate_hash(&self) -> u64 {
        let mut hash = 0;

        for color in Color::iter() {
            for piece in PieceName::iter() {
                let mut occupancies = self.bitboards[color as usize][piece as usize];
                while occupancies != Bitboard::EMPTY {
                    hash ^=
                        self.zobrist.piece_square_hashes[color as usize][piece as usize][occupancies.pop_lsb().idx()]

                }
            }
        }

        if self.to_move == Color::Black {
            hash ^= self.zobrist_consts.turn_hash;
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
