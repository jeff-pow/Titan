#![allow(clippy::module_inception)]
pub mod board;
pub mod engine;
pub mod eval;
pub mod moves;
pub mod search;
pub mod types;

use engine::uci::main_loop;

use crate::board::zobrist::ZOBRIST;
use crate::engine::transposition::{bad_overwrites, collisions, overwrites, probes, successes, writes};
use crate::moves::movegenerator::MG;
use crate::search::search::search;
use crate::search::{SearchInfo, SearchType};
use crate::types::square::Square;
fn main() {
    // main_loop();
    let mut search_info = SearchInfo::default();
    let _ = ZOBRIST.turn_hash;
    let _ = MG.king_attacks(Square(0));
    let depth = 25;
    search_info.max_depth = depth;
    search_info.search_type = SearchType::Depth;
    let mut s = search_info.clone();
    println!("bestmove {}", search(&mut s, depth).to_san());
    unsafe {
        dbg!(successes);
        dbg!(collisions);
        dbg!(probes);
        dbg!(writes);
        dbg!(overwrites);
        dbg!(bad_overwrites);
    }

    dbg!(search_info
        .transpos_table
        .read()
        .unwrap()
        .vec
        .clone()
        .into_vec()
        .iter()
        .filter(|x| x.key() == 0)
        .count());
}
