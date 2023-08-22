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

static mut TURN_HASH: u64 = 0;
static mut PIECE_SQUARE_HASHES: [[[u64; 64]; 6]; 2] = [[[0; 64]; 6]; 2];

/// Function checks for the presence of the board in the game. If the board position will have occurred three times,
/// returns true indicating the position would be a stalemate due to the threefold repetition rule
pub fn check_for_3x_repetition(board: &Board) -> bool {
    debug_assert_eq!(board.zobrist_hash, board.generate_hash());
    let hash = board.zobrist_hash;
    let count = board.history.iter().filter(|&&x| x == hash).count();
    count >= 2
}

pub fn init_zobrist() {
    let mut rng = Rng::default();
    unsafe {
        TURN_HASH = rng.next_u64();
        PIECE_SQUARE_HASHES
            .iter_mut()
            .flatten()
            .flatten()
            .for_each(|x| *x = rng.next_u64());
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
                    unsafe {
                        hash ^= PIECE_SQUARE_HASHES[color as usize][piece as usize]
                            [occupancies.pop_lsb().idx()]
                    }
                }
            }
        }

        if self.to_move == Color::Black {
            hash ^= unsafe { TURN_HASH };
        }

        hash
    }
}

#[cfg(test)]
mod hashing_test {
    use crate::board::fen;
    use crate::init::init;

    #[test]
    fn test_hashing() {
        init();
        let board1 = fen::build_board(fen::STARTING_FEN);
        let board2 = fen::build_board("4r3/4k3/8/4K3/8/8/8/8 w - - 0 1");
        let board3 = fen::build_board("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        assert_ne!(board1.generate_hash(), board2.generate_hash());
        assert_eq!(board1.generate_hash(), board3.generate_hash());
    }
}
