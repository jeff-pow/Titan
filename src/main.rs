#![allow(clippy::module_inception)]
#[warn(clippy::perf)]
pub mod board;
pub mod engine;
pub mod eval;
pub mod moves;
pub mod search;
pub mod types;

use engine::uci::main_loop;

fn main() {
    main_loop();
}
