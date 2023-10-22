#![allow(clippy::module_inception)]
pub mod board;
pub mod engine;
pub mod eval;
pub mod moves;
pub mod search;
pub mod types;

use engine::uci::main_loop;
use crate::board::fen::{build_board, STARTING_FEN};
use crate::eval::nnue::NET;
use crate::types::pieces::Color;

fn main() {
    // let mut b = build_board(STARTING_FEN);
    // b.refresh_accumulators();
    // assert_eq!(b.accumulator.get(Color::White), b.accumulator.get(Color::Black));
    println!("{:?}", &NET);
    main_loop();
}
