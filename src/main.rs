pub mod board;
pub mod engine;
pub mod init;
pub mod moves;
pub mod search;
pub mod types;

use crate::engine::uci::main_loop;
use crate::init::init;

fn main() {
    init();
    // let board = board::fen::build_board(board::fen::STARTING_FEN);
    // let mut searcher = search::alpha_beta::Search::new();
    // println!("{}", searcher.search(&board, 7));
    main_loop();
}
