use crate::{
    attack_boards::*,
    moves::{self, coordinates, rank, Direction},
};


#[inline]
pub fn shift(bitboard: u64, dir: Direction) -> u64 {
    match dir {
        Direction::North => bitboard << 8,
        Direction::NorthWest => (bitboard << 7) & !FILE_H.0,
        Direction::West => (bitboard >> 1) & !FILE_H.0,
        Direction::SouthWest => (bitboard >> 9) & !FILE_H.0,
        Direction::South => bitboard >> 8,
        Direction::SouthEast => (bitboard >> 7) & !FILE_A.0,
        Direction::East => (bitboard << 1) & !FILE_A.0,
        Direction::NorthEast => (bitboard << 9) & !FILE_A.0,
    }
}

#[inline]
pub fn pop_lsb(bb: &mut u64) -> u64 {
    let lsb = *bb & bb.wrapping_neg();
    *bb ^= lsb;
    lsb.trailing_zeros() as u64
}

#[inline]
pub fn get_rank_bitboard(square: u8) -> u64 {
    let x = moves::rank(square);
    match x {
        0 => RANK1.0,
        1 => RANK2.0,
        2 => RANK3.0,
        3 => RANK4.0,
        4 => RANK5.0,
        5 => RANK6.0,
        6 => RANK7.0,
        7 => RANK8.0,
        _ => panic!(),
    }
}

#[inline]
pub fn get_file_bitboard(square: u8) -> u64 {
    let y = moves::file(square);
    match y {
        0 => FILE_A.0,
        1 => FILE_B.0,
        2 => FILE_C.0,
        3 => FILE_D.0,
        4 => FILE_E.0,
        5 => FILE_F.0,
        6 => FILE_G.0,
        7 => FILE_H.0,
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

pub fn dist(s1: u8, s2: u8) -> u64 {
    let x1 = rank(s1);
    let y1 = moves::file(s1);
    let x2 = rank(s2);
    let y2 = moves::file(s2);
    let x_diff = x1.abs_diff(x2);
    let y_diff = y1.abs_diff(y2);
    x_diff.max(y_diff) as u64
}

