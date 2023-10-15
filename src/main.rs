#![allow(clippy::module_inception)]
pub mod board;
pub mod engine;
pub mod eval;
pub mod moves;
pub mod search;
pub mod types;

use engine::uci::main_loop;

use crate::{board::fen::build_board, moves::movepicker::perft};

fn main() {
    // main_loop();
    let board = build_board("n1n5/PPPk4/8/8/8/8/4Kppp/5N1N b - - 0 1");
    assert_eq!(71_179_139, perft(board, 6));
}
