#![allow(clippy::module_inception)]
#![allow(long_running_const_eval)]
#![cfg_attr(feature = "simd", feature(stdsimd))]

mod bench;
mod board;
mod engine;
mod eval;
mod moves;
mod search;
mod types;

use crate::bench::bench;
use crate::board::zobrist::{Z, ZOBRIST};
use crate::engine::uci::main_loop;
use std::env;

fn main() {
    assert_eq!(Z.turn_hash, ZOBRIST.turn_hash);
    let args = env::args().collect::<Vec<_>>();
    if args.contains(&"bench".to_string()) {
        bench();
    } else {
        main_loop();
    }
}
