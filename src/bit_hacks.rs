use crate::{
    attack_boards::{FILE_A, FILE_H},
    moves::Direction,
};

pub fn shift_north(bb: u64) -> u64 {
    bb << 8
}
pub fn shift_south(bb: u64) -> u64 {
    bb >> 8
}
pub fn shift_east(bb: u64) -> u64 {
    (bb << 1) & !FILE_A
}
pub fn shift_west(bb: u64) -> u64 {
    (bb >> 1) & !FILE_H
}
pub fn shift_northeast(bb: u64) -> u64 {
    (bb << 9) & !FILE_A
}
pub fn shift_southeast(bb: u64) -> u64 {
    (bb >> 7) & !FILE_A
}
pub fn shift_northwest(bb: u64) -> u64 {
    (bb << 7) & !FILE_H
}
pub fn shift_southwest(bb: u64) -> u64 {
    (bb >> 9) & !FILE_H
}
