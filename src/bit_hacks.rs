use crate::{
    attack_boards::*,
    moves::{coordinates, Direction},
};

#[inline]
pub fn shift(bitboard: u64, dir: Direction) -> u64 {
    match dir {
        Direction::North => bitboard << 8,
        Direction::NorthWest => (bitboard << 7) & !FILE_H,
        Direction::West => (bitboard >> 1) & !FILE_H,
        Direction::SouthWest => (bitboard >> 9) & !FILE_H,
        Direction::South => bitboard >> 8,
        Direction::SouthEast => (bitboard >> 7) & !FILE_A,
        Direction::East => (bitboard << 1) & !FILE_A,
        Direction::NorthEast => (bitboard << 9) & !FILE_A,
    }
}

#[inline]
pub fn pop_lsb(bb: &mut u64) -> u64 {
    let lsb = *bb & bb.wrapping_neg();
    *bb ^= lsb;
    lsb.trailing_zeros() as u64
}

#[inline]
pub fn bit_is_on(bb: u64, idx: usize) -> bool {
    bb & (1 << idx) != 0
}

#[inline]
pub fn bit_is_off(bb: u64, idx: usize) -> bool {
    bb & (1 << idx) == 0
}

#[inline]
pub fn get_rank_bitboard(square: u8) -> u64 {
    let (x, _) = coordinates(square as usize);
    match x {
        0 => RANK1,
        1 => RANK2,
        2 => RANK3,
        3 => RANK4,
        4 => RANK5,
        5 => RANK6,
        6 => RANK7,
        7 => RANK8,
        _ => panic!(),
    }
}

#[inline]
pub fn get_file_bitboard(square: u8) -> u64 {
    let (_, y) = coordinates(square as usize);
    match y {
        0 => FILE_A,
        1 => FILE_B,
        2 => FILE_C,
        3 => FILE_D,
        4 => FILE_E,
        5 => FILE_F,
        6 => FILE_G,
        7 => FILE_H,
        _ => panic!(),
    }
}

#[inline]
pub fn distance(s1: u8, s2: u8) -> u64 {
    let (x1, y1) = coordinates(s1 as usize);
    let (x2, y2) = coordinates(s2 as usize);
    let x_diff = x1.abs_diff(x2);
    let y_diff = y1.abs_diff(y2);
    x_diff.max(y_diff) as u64
}
