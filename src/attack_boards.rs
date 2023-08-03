use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::moves::coordinates;

pub struct AttackBoards {
    pub king: [u64; 64],
    pub knight: [u64; 64],
}

impl AttackBoards {
    pub fn new() -> Self {
        AttackBoards {
            king: gen_king_attack_boards(),
            knight: gen_knight_attack_boards(),
        }
    }
}
#[rustfmt::skip]
fn gen_king_attack_boards() -> [u64; 64] {
    // let mut arr = [0; 64];
    // for square in 0..64 {
    //     let (x, y) = coordinates(square);
    //     let moves = 
    //     (square & !7 & !6 & !0) << 15
    //     | (square & !7 & !6 & !7) << 17
    //     | (square & !0 & !1 & !7) << 6
    //     | (square & !6 & !7 & !7) << 10
    //     | (square & !0 & !1 & !0) >> 17
    //     | (square & !0 & !1 & !7) >> 15
    //     | (square & !0 & !1 & !0) >> 10
    //     | (square & !6 & !7 & !0) >> 6;
    //             let moves =
    //         (square.wrapping_sub(9) & !7) | // Up left
    //         (square.wrapping_sub(8) & !6) | // Up
    //         (square.wrapping_sub(7) & !7) | // Up right
    //         (square.wrapping_sub(1) & !0) | // Left
    //         (square.wrapping_add(1) & !7) | // Right
    //         (square.wrapping_add(7) & !0) | // Down left
    //         (square.wrapping_add(8) & !6) | // Down
    //         (square.wrapping_add(9) & !7);  // Down right
    //
    //     arr[square] = moves as u64;
    // }
    let mut arr = [0; 64];
    arr.iter_mut().enumerate().for_each(|(square, moves)| {
        dbg!(square);
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
            if x >= 1 { *moves |= 1u64 << (square + 15); }
            if x <= 6 { *moves |= 1u64 << (square + 17); }
        }
        if y <= 5 {
            if x >= 2 { *moves |= 1u64 << (square + 6); }
            if x <= 5 { *moves |= 1u64 << (square + 10); }
        }
    });
    arr
}

#[rustfmt::skip]
fn gen_knight_attack_boards() -> [u64; 64] {
    let mut arr = [0; 64];
    for square in 0..64 {
        let (x, y) = coordinates(square);
        let moves = 
        (square & !7 & !6 & !0) << 15
        | (square & !7 & !6 & !7) << 17
        | (square & !0 & !1 & !7) << 6
        | (square & !6 & !7 & !7) << 10
        | (square & !0 & !1 & !0) >> 17
        | (square & !0 & !1 & !7) >> 15
        | (square & !0 & !1 & !0) >> 10
        | (square & !6 & !7 & !0) >> 6;
        arr[square] = moves as u64;
    }
    arr
}

/// Movement chords are defined by a combination of three cardinal directions - ex West West North
#[derive(EnumIter, Copy, Clone, Debug, PartialEq, Eq)]
#[allow(clippy::upper_case_acronyms)]
enum KnightMovement {
    WWN = 6,
    WNN = 15,
    ENN = 17,
    EEN = 10,
    EES = -6,
    ESS = -15,
    WSS = -17,
    WWS = -10,
}
