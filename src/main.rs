#![allow(clippy::module_inception)]
pub mod board;
pub mod engine;
pub mod eval;
pub mod moves;
pub mod search;
pub mod types;

use std::time::Instant;

use engine::uci::main_loop;

use crate::{board::fen::build_board, engine::perft::perft};

fn main() {
    // main_loop();
    let board = build_board("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -");
    let start = Instant::now();
    let p = perft::<false>(board, 5);
    assert_eq!(193_690_690, p);
    println!("{} nps", p as f64 / start.elapsed().as_secs_f64());
}
