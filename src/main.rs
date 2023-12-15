#![allow(clippy::module_inception)]
#![allow(long_running_const_eval)]
#![cfg_attr(feature = "simd", feature(stdsimd))]

mod bench;
mod board;
mod engine;
mod eval;
mod moves;
mod search;
mod types;

use crate::bench::bench;
use crate::engine::uci::main_loop;
use crate::moves::attack_boards::{KING_ATTACKS, KNIGHT_ATTACKS, PAWN_ATTACKS};
use crate::moves::magics::{BISHOP_TABLE, ROOK_TABLE};
use crate::moves::movegenerator::MG;
use std::env;

fn main() {
    assert_eq!(MG.pawn_table, PAWN_ATTACKS);
    assert_eq!(MG.king_table, KING_ATTACKS);
    assert_eq!(MG.knight_table, KNIGHT_ATTACKS);
    assert_eq!(MG.magics.bishop_table, BISHOP_TABLE);
    // assert_eq!(MG.magics.rook_table, ROOK_TABLE);
    for (idx, i) in MG.magics.rook_table.iter().enumerate() {
        assert_eq!(i, &ROOK_TABLE[idx], "{idx} {:?}", i);
    }
    let args = env::args().collect::<Vec<_>>();
    if args.contains(&"bench".to_string()) {
        bench();
    } else {
        main_loop();
    }
}
