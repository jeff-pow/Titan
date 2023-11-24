#![allow(clippy::module_inception)]
#![feature(stdsimd)]
mod bench;
mod board;
mod engine;
mod eval;
mod moves;
mod search;
mod types;

use engine::uci::main_loop;

use crate::board::fen::{build_board, STARTING_FEN};

fn main() {
    // dbg!(build_board(STARTING_FEN).evaluate());
    main_loop();
}
