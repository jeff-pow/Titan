use crate::{
    attack_boards::{FILE_A, FILE_H},
    moves::Direction,
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
