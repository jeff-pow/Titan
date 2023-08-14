// use std::collections::HashMap;
use rustc_hash::FxHashMap;

use crate::{
    board::board::Board,
    search::eval::eval,
    types::{bitboard::Bitboard, pieces::Color},
};

#[rustfmt::skip]
/// Randomly generated values to hash boards. Far from perfect, but *probably* good enough to avoid
/// any collisions that will majorly affect decision making
const SQUARE_HASHES: [u64; 64] = [
0x987019bc8b603aac, 0xdb4ecbbe7286bf41, 0xdd4011ba06a01ec3, 0x3bd9d3c4f88f773d, 0xc8e369bb8754b32d, 0xd8d00026f0249be1, 0x6868cf1aac89168c, 0x8653b94c8e20b9c1,
0x799f7f18f2139f18, 0x50639d1ce6d4c90e, 0xab097ce82c5c3182, 0x9e3bc31bbd6e4f34, 0x024e76b495682123, 0xaa81c5a550552336, 0xbc40ec5434582311, 0xda86ca687b0933a2,
0xeb4c3526316cf784, 0x118f834724c3d5e1, 0x2898bdd77ae76559, 0x617734a59b5bab06, 0x847a3115cf54d8b5, 0xd13aa4489f8057bb, 0x347d17ee130ede04, 0xbe89ee23a98b0760,
0xffcc43de5fe530bb, 0x569dd36892830fb8, 0x5707bc5895696d8a, 0xe1d0dd86c80bfb40, 0x8f0d5866bced25a3, 0x58de5ae1eae94b22, 0x0118a31e0007bd55, 0xc9f5da792d2adb3f,
0x2319c97970800612, 0xfe5052dacd4e8554, 0x7e856893d7358886, 0x2ee46d7051593aaa, 0x3a0c018cdace0afe, 0x205241a49b8c1759, 0x123849ccd31f433b, 0xd780832c8a0c4f6a,
0x70c1ef0df2e5919c, 0x50cec1198612153e, 0x1d5b78c31ac2380c, 0xf0809be397cd07dd, 0x25f2ab099b458ee4, 0x8c9e8cc2f49aaab7, 0x1fd731c4a166ff1d, 0x7af79fbb529e66c1,
0x053e510d8699a8ea, 0xbff7984370d780ce, 0x82dcfcd33d03e404, 0x4045f49923b1d7cc, 0xf7491819dcd5a68d, 0x93a2a8e29efe0679, 0x37dd5a91e3e83b3c, 0xc605e9f68fc5b333,
0x344057a0d3bc89af, 0x383c647f42f417b7, 0x58d163ba5c76fb69, 0xe18b6540a36fff7d, 0x3dc554a23ac08ac7, 0xc5d5edffdead807d, 0x71c6a53448e3a35d, 0xd628f30ca0a6a2e4
];

const TURN_HASH: u64 = 0x0b2727e5e37fed2d;

/// Function checks for the presence of the board in the game. If the board position will have occurred three times,
/// returns true indicating the position would be a stalemate due to the threefold repetition rule
pub fn check_for_3x_repetition(board: &Board) -> bool {
    debug_assert_eq!(board.zobrist_hash, board.generate_hash());
    let hash = board.zobrist_hash;
    let count = board.history.iter().filter(|&&x| x == hash).count();
    count >= 2
}

impl Board {
    /// Provides a hash for the board eval to be placed into a transposition table
    #[inline(always)]
    pub fn generate_hash(&self) -> u64 {
        let mut hash = 0;
        let mut occupancies = self.occupancies();
        while occupancies != Bitboard::EMPTY {
            hash ^= SQUARE_HASHES[occupancies.pop_lsb().idx()];
        }

        if self.to_move == Color::Black {
            hash ^= TURN_HASH;
        }

        if self.en_passant_square.is_valid() {
            hash ^= SQUARE_HASHES[self.en_passant_square.idx()];
        }

        // dbg!(hsh.elapsed());
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
