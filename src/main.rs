#![allow(clippy::module_inception)]
mod bench;
mod board;
mod engine;
mod eval;
mod moves;
mod search;
mod types;

use crate::engine::uci::main_loop;

fn main() {
    main_loop();
}
