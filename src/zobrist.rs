use crate::{
    board::Board,
    const_array,
    magics::{rand_u64, Rng},
    types::pieces::{Color, PieceName},
};

#[derive(Debug, PartialEq, Eq)]
pub struct Zobrist {
    pub piece_square_hashes: [[[u64; 64]; 6]; 2],
    pub turn_hash: u64,
    pub castling: [u64; 16],
    // 64 squares plus an invalid square
    // Don't bother figuring out invalid enpassant squares, literally not worth the squeeze
    pub en_passant: [u64; 64],
}

pub const ZOBRIST: Zobrist = {
    let mut prev = 0xE926_E621_0D9E_3487u64;

    let turn_hash = rand_u64(prev);

    let piece_square_hashes = const_array!(|c, 2| const_array!(|p, 6| const_array!(|sq, 64| {
        prev = rand_u64(prev);
        prev
    })));
    let castling = const_array!(|c, 16| {
        prev = rand_u64(prev);
        prev
    });
    let en_passant = const_array!(|sq, 64| {
        prev = rand_u64(prev);
        prev
    });
    Zobrist { piece_square_hashes, turn_hash, castling, en_passant }
};

impl Default for Zobrist {
    fn default() -> Self {
        let mut rng = Rng::default();
        let turn_hash = rng.next_u64();
        let mut piece_square_hashes = [[[0; 64]; 6]; 2];
        piece_square_hashes.iter_mut().flatten().flatten().for_each(|x| *x = rng.next_u64());
        let mut castling = [0; 16];
        castling.iter_mut().for_each(|x| *x = rng.next_u64());
        let mut en_passant = [0; 64];
        en_passant.iter_mut().for_each(|x| *x = rng.next_u64());
        Self { piece_square_hashes, turn_hash, castling, en_passant }
    }
}

impl Board {
    /// Provides a hash for the board eval to be placed into a transposition table
    pub(crate) fn generate_hash(&self) -> u64 {
        let mut hash = 0;

        for color in Color::iter() {
            for piece in PieceName::iter() {
                let occupancies = self.piece_color(color, piece);
                for sq in occupancies {
                    hash ^= ZOBRIST.piece_square_hashes[color][piece][sq];
                }
            }
        }

        if let Some(x) = self.en_passant_square {
            hash ^= ZOBRIST.en_passant[x];
        }

        hash ^= ZOBRIST.castling[self.castling_rights as usize];

        if self.stm == Color::Black {
            hash ^= ZOBRIST.turn_hash;
        }

        hash
    }
}

#[cfg(test)]
mod hashing_test {
    use crate::{board::Board, fen};

    #[test]
    fn test_hashing() {
        let board1 = Board::from_fen(fen::STARTING_FEN);
        let board2 = Board::from_fen("4r3/4k3/8/4K3/8/8/8/8 w - - 0 1");
        let board3 = Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        assert_ne!(board1.generate_hash(), board2.generate_hash());
        assert_eq!(board1.generate_hash(), board3.generate_hash());
    }
}
