#![allow(clippy::module_inception)]
#![deny(unused_must_use)]
#![cfg_attr(feature = "avx512", feature(stdarch_x86_avx512))]

mod attack_boards;
mod bench;
mod board;
mod chess_move;
mod eval;
mod history_table;
mod magics;
mod movegen;
mod movelist;
mod movepicker;
mod perft;
mod search;
mod see;
mod thread;
mod transposition;
mod types;
mod uci;
mod utils;

use crate::bench::bench;
use board::Board;
use search::{search::start_search, SearchType};
use std::{
    env,
    process::exit,
    sync::atomic::{AtomicBool, AtomicU64},
};
use thread::ThreadData;
use transposition::{TranspositionTable, TARGET_TABLE_SIZE_MB};
use uci::main_loop;

fn main() {
    let board = Board::from_fen("4k1K1/3n4/2N5/4N3/8/8/8/8 w - - 0 1");
    let tt = TranspositionTable::new(TARGET_TABLE_SIZE_MB);
    let halt = AtomicBool::new(false);
    let global_nodes = AtomicU64::new(0);

    let mut thread = ThreadData::new(&halt, Vec::new(), 0, &global_nodes);

    thread.search_types = vec![SearchType::Mate(2), SearchType::Nodes(20000)];

    //start_search(&mut thread, true, board, &tt);
    //exit(0);

    if env::args().any(|x| x == *"bench") {
        bench();
    } else {
        main_loop();
    }
}
