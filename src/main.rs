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
    // should be 22 :(
    println!("{}", NET.evaluate(&b.accumulator, b.to_move));
    // main_loop();
}
