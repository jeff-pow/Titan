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

fn main() {
    main_loop();
}
