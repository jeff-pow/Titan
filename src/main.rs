#![allow(clippy::module_inception)]
pub mod board;
pub mod engine;
pub mod eval;
pub mod moves;
pub mod search;
pub mod types;

use board::fen::{build_board, STARTING_FEN};
use eval::nnue::Network;

use crate::eval::nnue::NETWORK;

fn main() {
    let net = Network::new();
    let mut board = build_board(STARTING_FEN);
    board.refresh_accumulators(&net);
    println!("{}", NETWORK.evaluate(&board.accumulator, board.to_move));
    // main_loop();
}
