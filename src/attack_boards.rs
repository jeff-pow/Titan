use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::{
    board::Board,
    moves::coordinates,
    pieces::{Color, PieceName},
};

pub const FILE_A: u64 = 0x101010101010101;
pub const FILE_H: u64 = FILE_A << 7;

pub const RANK1: u64 = 0b11111111;
pub const RANK2: u64 = RANK1 << 8;
pub const RANK3: u64 = RANK2 << 8;
pub const RANK4: u64 = RANK3 << 8;
pub const RANK5: u64 = RANK4 << 8;
pub const RANK6: u64 = RANK5 << 8;
pub const RANK7: u64 = RANK6 << 8;
pub const RANK8: u64 = RANK7 << 8;

pub struct AttackBoards {
    pub king: [u64; 64],
    pub knight: [u64; 64],
}

impl AttackBoards {
    pub fn new() -> Self {
        AttackBoards {
            knight: gen_knight_attack_boards(),
            king: gen_king_attack_boards(),
        }
    }
}

#[rustfmt::skip]
fn gen_king_attack_boards() -> [u64; 64] {
    let mut arr = [0; 64];
    arr.iter_mut().enumerate().for_each(|(square, moves)| {
        let (x, y) = coordinates(square);  
        if y >= 1 {
            if x >= 1 { *moves |= 1u64 << (square as u32 - 9); }
            *moves |= 1u64 << (square as u32 - 8);
            if x <= 6 { *moves |= 1u64 << (square as u32 - 7); }
        }

        if x >= 1 { *moves |= 1u64 << (square as u32 - 1); }
        if x <= 6 { *moves |= 1u64 << (square as u32 + 1); }

        if y <= 6 {
            if x >= 1 { *moves |= 1u64 << (square as u32 + 7); }
            *moves |= 1u64 << (square as u32 + 8);
            if x <= 6 { *moves |= 1u64 << (square as u32 + 9); }
        }
    });
    arr
}

#[rustfmt::skip]
fn gen_knight_attack_boards() -> [u64; 64] {
    let mut arr = [0; 64];
    arr.iter_mut().enumerate().for_each(|(square, moves)| {
        let (x, y) = coordinates(square);
        if y >= 2 {
            if x >= 1 { *moves |= 1u64 << (square - 17); }
            if x <= 6 { *moves |= 1u64 << (square - 15); }
        }
        if y >= 1 {
            if x >= 2 { *moves |= 1u64 << (square - 10); }
            if x <= 5 { *moves |= 1u64 << (square - 6); }
        }
        if y <= 6 {
            if x >= 1 && square + 15 < 64 { *moves |= 1u64 << (square + 15); }
            if x <= 6 && square + 17 < 64 { *moves |= 1u64 << (square + 17); }
        }
        if y <= 5 {
            if x >= 2 && square + 6 < 64 { *moves |= 1u64 << (square + 6); }
            if x <= 5 && square + 10 < 64 { *moves |= 1u64 << (square + 10); }
        }
    });
    arr
}

pub fn gen_pawn_attack_board(board: &Board) -> u64 {
    let pawns = board.board[board.to_move as usize][PieceName::Pawn as usize];

    if board.to_move == Color::White {
        ((pawns << 9) & !FILE_A) | ((pawns << 7) & !FILE_H)
    } else {
        ((pawns >> 7) & !FILE_H) | ((pawns >> 9) & !FILE_H)
    }
}
