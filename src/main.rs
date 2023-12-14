#![allow(clippy::module_inception)]
#![cfg_attr(feature = "simd", feature(stdsimd))]

mod bench;
mod board;
mod engine;
mod eval;
mod moves;
mod search;
mod spsa;
mod types;

use crate::bench::bench;
use crate::engine::uci::main_loop;
use std::env;

fn main() {
    let args = env::args().collect::<Vec<_>>();
    if args.contains(&"bench".to_string()) {
        bench();
    } else {
        main_loop();
    }
}
