#![allow(clippy::module_inception)]
pub mod board;
pub mod engine;
pub mod moves;
pub mod search;
pub mod types;

use crate::board::fen::{build_board, STARTING_FEN};
use crate::search::pvs::search;
use crate::search::{SearchInfo, SearchType};
use engine::uci::main_loop;

fn main() {
    main_loop();
}
