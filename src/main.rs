#![allow(clippy::module_inception)]
pub mod board;
pub mod engine;
pub mod init;
pub mod moves;
pub mod search;
pub mod types;

use crate::board::fen::{build_board, STARTING_FEN};
use crate::engine::transposition::get_table;
use crate::engine::uci::main_loop;
use crate::init::init;
use crate::search::{pvs, SearchInfo};

fn main() {
    init();
    // let mut search_info = SearchInfo::default();
    // search_info.transpos_table = get_table();
    // search_info.board = build_board(STARTING_FEN);
    // search_info.iter_max_depth = 9;
    // pvs::search(&mut search_info);
    main_loop();
}
