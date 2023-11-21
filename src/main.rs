#![allow(clippy::module_inception)]
mod bench;
pub mod board;
pub mod engine;
pub mod eval;
pub mod moves;
pub mod search;
pub mod types;

use std::time::Instant;

use crate::board::fen::{build_board, STARTING_FEN};
use engine::perft::non_bulk_perft;

fn main() {
    let s = Instant::now();
    let a = non_bulk_perft(build_board(STARTING_FEN), 6);
    println!("{} nps", a as f64 / s.elapsed().as_secs_f64());
    // main_loop();
}
