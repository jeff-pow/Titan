#![allow(clippy::module_inception)]
pub mod board;
pub mod engine;
pub mod eval;
pub mod moves;
pub mod search;
pub mod types;

use engine::uci::main_loop;

use crate::board::zobrist::ZOBRIST;
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
}
