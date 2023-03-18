use std::{collections::HashMap, hash::Hasher};

use crate::{board::Board, pieces::Color, eval::eval};
use crate::search::IN_CHECK_MATE;

#[rustfmt::skip]
const PIECE_HASHES: [u64; 64] = [
0x987019bc8b603aac, 0xdb4ecbbe7286bf41, 0xdd4011ba06a01ec3, 0x3bd9d3c4f88f773d, 0xc8e369bb8754b32d, 0xd8d00026f0249be1, 0x6868cf1aac89168c, 0x8653b94c8e20b9c1,
0x799f7f18f2139f18, 0x50639d1ce6d4c90e, 0xab097ce82c5c3182, 0x9e3bc31bbd6e4f34, 0x024e76b495682123, 0xaa81c5a550552336, 0xbc40ec5434582311, 0xda86ca687b0933a2,
0xeb4c3526316cf784, 0x118f834724c3d5e1, 0x2898bdd77ae76559, 0x617734a59b5bab06, 0x847a3115cf54d8b5, 0xd13aa4489f8057bb, 0x347d17ee130ede04, 0xbe89ee23a98b0760,
0xffcc43de5fe530bb, 0x569dd36892830fb8, 0x5707bc5895696d8a, 0xe1d0dd86c80bfb40, 0x8f0d5866bced25a3, 0x58de5ae1eae94b22, 0x0118a31e0007bd55, 0xc9f5da792d2adb3f,
0x2319c97970800612, 0xfe5052dacd4e8554, 0x7e856893d7358886, 0x2ee46d7051593aaa, 0x3a0c018cdace0afe, 0x205241a49b8c1759, 0x123849ccd31f433b, 0xd780832c8a0c4f6a,
0x70c1ef0df2e5919c, 0x50cec1198612153e, 0x1d5b78c31ac2380c, 0xf0809be397cd07dd, 0x25f2ab099b458ee4, 0x8c9e8cc2f49aaab7, 0x1fd731c4a166ff1d, 0x7af79fbb529e66c1,
0x053e510d8699a8ea, 0xbff7984370d780ce, 0x82dcfcd33d03e404, 0x4045f49923b1d7cc, 0xf7491819dcd5a68d, 0x93a2a8e29efe0679, 0x37dd5a91e3e83b3c, 0xc605e9f68fc5b333,
0x344057a0d3bc89af, 0x383c647f42f417b7, 0x58d163ba5c76fb69, 0xe18b6540a36fff7d, 0x3dc554a23ac08ac7, 0xc5d5edffdead807d, 0x71c6a53448e3a35d, 0xd628f30ca0a6a2e4
];

const DEPTH_HASHES: [u64; 12] = [
    0xe1cdf95730e6d147, 0xaf8178c6a9aac919, 0x806514c8c42cae1d, 0x574e46b99dd612c2, 0xb42131d22f80671c, 0x34adc1c0a1eedaa2, 0xd1bda26897e9c301, 0x4de223d667b2859a, 0x8aa9718c3054950c, 0x93685e3eb0e1e3dc, 0xab7d6f6ec8b75186, 0xfdb16b38b7bfe671
];

pub const EXACT: i32 = 0;
pub const LOWER_BOUND: i32 = 1;
pub const UPPER_BOUND: i32 = 2;

pub struct TableEntry {
    pub depth: i32,
    pub eval: i32,
    pub node_type: i32,
}

pub fn hash_entry(board: &Board, depth: i32) -> u64 {
    let mut hash = hash_board(board);
    hash ^= DEPTH_HASHES[depth as usize];
    hash
}

pub fn get_node_eval(hash: &u64, ply: i32, depth: i32, map: &mut HashMap<u64, TableEntry>, alpha: i32, beta: i32) -> Option<i32> {
    if let Some(entry) = map.get(hash) {
        if entry.depth >= depth {
            let corrected_score = fix_score(entry.eval, ply);
            if entry.node_type == EXACT {
                return Some(corrected_score);
            }
            if entry.node_type == UPPER_BOUND && corrected_score <= alpha {
                return Some(corrected_score);
            }
            if entry.node_type == LOWER_BOUND && corrected_score >= beta {
                return Some(corrected_score);
            }
        }
    }
    None
}

fn fix_score(score: i32, ply: i32) -> i32 {
    if score.abs() == IN_CHECK_MATE {
        let sign = score.signum();
        return (score * sign - ply) * sign;
    }
    score
}

pub fn place_entry(hash: &u64, depth: i32, eval: i32, map: &mut HashMap<u64, TableEntry>, node_type: i32) {
    map.insert(*hash, TableEntry{
        depth,
        eval,
        node_type,
    });
}


/// Function checks for the presence of the board in the game. If the board position will have occured three times,
/// returns true indicating the position would be a stalemate due to the threefold repetition rule
pub fn check_for_3x_repetition(board: &Board, triple_repetitions: &mut HashMap<u64, u8>) -> bool {
    let hash = hash_board(board);
    if let Some(num) = triple_repetitions.get(&hash) {
        if num >= &2 {
            return true;
        }
    }
    false
}

pub fn hash_board(board: &Board) -> u64 {
    let mut hash = 0;
    for square in 0..64 {
        if let Some(piece) = board.board[square] {
            let idx = piece.current_square;
            hash ^= PIECE_HASHES[idx as usize];
        }
    }

    if board.to_move == Color::Black {
        hash ^= 0x0b2727e5e37fed2d;
    }

    hash
}

pub fn get_transposition(board: &Board, transpos_table: &mut HashMap<u64, i32>) -> i32 {
    let hash = hash_board(board);
    let found = transpos_table.get(&hash);
    match found {
        Some(val) => {
            *val
        }
        None => {
            let val = eval(board);
            transpos_table.insert(hash, val);
            val
        }
    }
}

pub fn add_to_map(board: &Board, triple_repetitions: &mut HashMap<u64, u8>) {
    let hash = hash_board(board);
    let found = triple_repetitions.get(&hash);
    match found {
        // If the board state has already been found, add one to the number of times that location has been found
        Some(count) => {
            triple_repetitions.insert(hash, *count + 1);
        }
        // Otherwise this is the first occurrence to the map
        None => {
            triple_repetitions.insert(hash, 1);
        }
    }
}

pub fn remove_from_map(board: &Board, triple_repetitions: &mut HashMap<u64, u8>) {
    let hash = hash_board(board);
    let found = triple_repetitions.get(&hash);
    if let Some(count) = found {
        triple_repetitions.insert(hash, *count - 1);
    }
}

#[cfg(test)]
mod hashing_test {
    use crate::{fen, zobrist::hash_board};

    #[test]
    fn test_different_boards() {
        let board1 = fen::build_board(fen::STARTING_FEN);
        let board2 = fen::build_board("4r3/4k3/8/4K3/8/8/8/8 w - - 0 1");
        let board3 = fen::build_board("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        assert_ne!(hash_board(&board1), hash_board(&board2));
        assert_eq!(hash_board(&board1), hash_board(&board3));
    }
}
