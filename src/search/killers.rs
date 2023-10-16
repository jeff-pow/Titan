use crate::moves::moves::Move;

use super::{pvs::MAX_SEARCH_DEPTH, SearchInfo};

pub type KillerMoves = [[Move; NUM_KILLER_MOVES]; MAX_SEARCH_DEPTH as usize];

pub const NUM_KILLER_MOVES: usize = 2;

pub fn empty_killers() -> KillerMoves {
    [[Move::NULL; NUM_KILLER_MOVES]; MAX_SEARCH_DEPTH as usize]
}

pub fn store_killer_move(ply: i32, m: Move, search_info: &mut SearchInfo) {
    let first_killer = search_info.killer_moves[ply as usize][0];

    if first_killer != m {
        for i in (1..NUM_KILLER_MOVES).rev() {
            let n = i;
            let previous = search_info.killer_moves[ply as usize][n - 1];
            search_info.killer_moves[ply as usize][n] = previous;
        }
        search_info.killer_moves[ply as usize][0] = m;
    }
}
