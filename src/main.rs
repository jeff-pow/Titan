#![allow(clippy::module_inception)]
#![allow(clippy::cast_possible_truncation)]
#![allow(long_running_const_eval)]
#![cfg_attr(feature = "avx512", feature(stdarch_x86_avx512))]
#[cfg(all(feature = "avx2", feature = "avx512"))]
compile_error!("Cannot enable both avx2 and avx512 simultaneously.");

mod bench;
mod board;
mod engine;
mod eval;
mod moves;
mod search;
mod types;

use moves::moves::Move;

use crate::bench::bench;
use crate::board::board::Board;
use crate::engine::perft::perft;
use crate::engine::uci::main_loop;
use std::env;

fn main() {
    let mut b = Board::default();
    b.make_move::<false>(Move::from_san("d2d4", &b));
    perft(&b, 1);
    if env::args().any(|x| x == *"bench") {
        bench();
    } else {
        main_loop();
    }
}
