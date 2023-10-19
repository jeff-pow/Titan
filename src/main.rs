#![allow(clippy::module_inception)]
pub mod board;
pub mod engine;
pub mod eval;
pub mod moves;
pub mod search;
pub mod types;

use board::fen::{build_board, STARTING_FEN};
use engine::uci::main_loop;
use eval::nnue::NetworkState;

fn main() {
    let board = build_board(STARTING_FEN);
    let n = NetworkState::new(&board);
    println!("{}", n.evaluate(board.to_move));
    // main_loop();
}
