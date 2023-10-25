#![allow(clippy::module_inception)]
pub mod board;
pub mod engine;
pub mod eval;
pub mod moves;
pub mod search;
pub mod types;

use crate::{
    board::fen::{build_board, STARTING_FEN},
    eval::nnue::NET,
};

fn main() {
    let mut b = build_board(STARTING_FEN);
    b.refresh_accumulators();
    b.accumulator.assert_valid(&b.array_board);
    // should be 22 :(
    println!("{}", b.evaluate());
    println!("{}", NET.feature_weights.iter().filter(|x| **x == 0).count());
    println!("{}", NET.feature_weights.len());
    // main_loop();
}
