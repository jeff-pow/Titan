#![allow(clippy::module_inception)]
pub mod board;
pub mod engine;
pub mod eval;
pub mod moves;
pub mod search;
pub mod types;

use engine::uci::main_loop;

use crate::eval::nnue::NET;

fn main() {
    // let mut b = build_board(STARTING_FEN);
    // b.refresh_accumulators();
    // assert_eq!(b.accumulator.get(Color::White), b.accumulator.get(Color::Black));
    println!("{:?}", &NET);
    // main_loop();
}
