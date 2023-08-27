#![allow(clippy::module_inception)]
pub mod board;
pub mod engine;
pub mod init;
pub mod moves;
pub mod search;
pub mod types;

use engine::uci::main_loop;

use crate::init::init;

fn main() {
    init();
    // let mut search_info = crate::search::SearchInfo::default();
    // search_info.transpos_table = crate::engine::transposition::get_table();
    // search_info.board = crate::board::fen::build_board(crate::board::fen::STARTING_FEN);
    // search_info.iter_max_depth = 9;
    // search_info.search_type = crate::search::SearchType::Depth;
    // crate::search::pvs::search(&mut search_info);
    main_loop();
}
