#![allow(clippy::module_inception)]
pub mod board;
pub mod engine;
pub mod eval;
pub mod moves;
pub mod search;
pub mod types;

use engine::uci::main_loop;
use crate::engine::perft::epd_perft;

fn main() {
    // main_loop();
    epd_perft("ethereal_perft.epd");
}
