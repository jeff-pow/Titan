#![allow(clippy::module_inception)]
mod bench;
pub mod board;
pub mod engine;
pub mod eval;
pub mod moves;
pub mod search;
pub mod types;

use crate::engine::uci::main_loop;

fn main() {
    main_loop();
}
