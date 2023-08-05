use crate::{
    attack_boards::{FILE_A, FILE_H},
    moves::Direction,
};

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
