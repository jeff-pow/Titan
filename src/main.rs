pub mod board;
pub mod engine;
pub mod init;
pub mod moves;
pub mod search;
pub mod types;

use board::fen::{build_board, STARTING_FEN};
use engine::perft::perft;

use crate::engine::uci::main_loop;
use crate::init::init;

fn main() {
    init();
    let start = std::time::Instant::now();
    perft(build_board(STARTING_FEN), 6);
    dbg!(start.elapsed());
    main_loop();
}
