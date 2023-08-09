use crate::{
    bitboard::Bitboard,
    board::Board,
    moves::coordinates,
    pieces::{Color, PieceName},
    pleco_magics::{self, init_magics},
    square::Square,
};

const FILE_A_U64: u64 = 0x101010101010101;
const FILE_B_U64: u64 = FILE_A_U64 << 1;
const FILE_C_U64: u64 = FILE_A_U64 << 2;
const FILE_D_U64: u64 = FILE_A_U64 << 3;
const FILE_E_U64: u64 = FILE_A_U64 << 4;
const FILE_F_U64: u64 = FILE_A_U64 << 5;
const FILE_G_U64: u64 = FILE_A_U64 << 6;
const FILE_H_U64: u64 = FILE_A_U64 << 7;

pub const FILE_A: Bitboard = Bitboard(FILE_A_U64);
pub const FILE_B: Bitboard = Bitboard(FILE_B_U64);
pub const FILE_C: Bitboard = Bitboard(FILE_C_U64);
pub const FILE_D: Bitboard = Bitboard(FILE_D_U64);
pub const FILE_E: Bitboard = Bitboard(FILE_E_U64);
pub const FILE_F: Bitboard = Bitboard(FILE_F_U64);
pub const FILE_G: Bitboard = Bitboard(FILE_G_U64);
pub const FILE_H: Bitboard = Bitboard(FILE_H_U64);

const RANK1_U64: u64 = 0b11111111;
const RANK2_U64: u64 = RANK1_U64 << 8;
const RANK3_U64: u64 = RANK2_U64 << 8;
const RANK4_U64: u64 = RANK3_U64 << 8;
const RANK5_U64: u64 = RANK4_U64 << 8;
const RANK6_U64: u64 = RANK5_U64 << 8;
const RANK7_U64: u64 = RANK6_U64 << 8;
const RANK8_U64: u64 = RANK7_U64 << 8;

pub const RANK1: Bitboard = Bitboard(0b11111111);
pub const RANK2: Bitboard = Bitboard(RANK2_U64);
pub const RANK3: Bitboard = Bitboard(RANK3_U64);
pub const RANK4: Bitboard = Bitboard(RANK4_U64);
pub const RANK5: Bitboard = Bitboard(RANK5_U64);
pub const RANK6: Bitboard = Bitboard(RANK6_U64);
pub const RANK7: Bitboard = Bitboard(RANK7_U64);
pub const RANK8: Bitboard = Bitboard(RANK8_U64);

static mut KNIGHT_TABLE: [Bitboard; 64] = [Bitboard::empty(); 64];
static mut KING_TABLE: [Bitboard; 64] = [Bitboard::empty(); 64];

pub fn knight_attacks(square: Square) -> Bitboard {
    unsafe { KNIGHT_TABLE[square.0 as usize] }
}

pub fn king_attacks(square: Square) -> Bitboard {
    unsafe { KING_TABLE[square.0 as usize] }
}

/// Non thread safe - this functions callee's have to finish running before the program will
/// successfully run w/o race conditions
pub fn init_attack_boards() {
    gen_king_attack_boards();
    gen_knight_attack_boards();
    init_magics();
}

#[rustfmt::skip]
fn gen_king_attack_boards() {
    unsafe {
        KING_TABLE.iter_mut().enumerate().for_each(|(square, moves)| {
            let (x, y) = coordinates(square);
            if y >= 1 {
                if x >= 1 { *moves |= Bitboard(1 << (square as u32 - 9)); }
                *moves |= Bitboard(1 << (square as u32 - 8));
                if x <= 6 { *moves |= Bitboard(1<< (square as u32 - 7)); }
            }

            if x >= 1 { *moves |= Bitboard(1 << (square as u32 - 1)); }
            if x <= 6 { *moves |= Bitboard(1 << (square as u32 + 1)); }

            if y <= 6 {
                if x >= 1 { *moves |= Bitboard(1 << (square as u32 + 7)); }
                *moves |= Bitboard(1 << (square as u32 + 8));
                if x <= 6 { *moves |= Bitboard(1 << (square as u32 + 9)); }
            }
        });
    }
}

#[rustfmt::skip]
fn gen_knight_attack_boards() {
    unsafe {
        KNIGHT_TABLE.iter_mut().enumerate().for_each(|(square, moves)| {
            let (x, y) = coordinates(square);
            let x = Square(square as u8).rank();
            let y = Square(square as u8).file();
            if x >= 2 {
                if y >= 1 { *moves |= Bitboard(1 << (square - 17)); }
                if y <= 6 { *moves |= Bitboard(1 << (square - 15)); }
            }
            if x >= 1 {
                if y >= 2 { *moves |= Bitboard(1 << (square - 10)); }
                if y <= 5 { *moves |= Bitboard(1 << (square - 6)); }
            }
            if x <= 6 {
                if y >= 1 && square + 15 < 64 { *moves |= Bitboard(1 << (square + 15)); }
                if y <= 6 && square + 17 < 64 { *moves |= Bitboard(1 << (square + 17)); }
            }
            if x <= 5 {
                if y >= 2 && square + 6 < 64 { *moves |= Bitboard(1 << (square + 6)); }
                if y <= 5 && square + 10 < 64 { *moves |= Bitboard(1 << (square + 10)); }
            }
        });
    }
}

pub fn gen_pawn_attack_board(board: &Board) -> Bitboard {
    let pawns = board.board[board.to_move as usize][PieceName::Pawn as usize];

    if board.to_move == Color::White {
        ((pawns << Bitboard(9)) & !FILE_A) | ((pawns << Bitboard(7)) & !FILE_H)
    } else {
        ((pawns >> Bitboard(9)) & !FILE_H) | ((pawns >> Bitboard(9)) & !FILE_H)
    }
}
