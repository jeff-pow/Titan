#![allow(clippy::module_inception)]
#![feature(stdsimd)]
mod bench;
mod board;
mod engine;
mod eval;
mod moves;
mod search;
mod types;

use board::fen::{build_board, STARTING_FEN};
use engine::uci::main_loop;

fn main() {
    // dbg!(build_board(STARTING_FEN).evaluate());
    main_loop();
}
