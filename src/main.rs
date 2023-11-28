#![allow(clippy::module_inception)]
#![cfg_attr(feature = "simd", feature(stdsimd))]

mod bench;
mod board;
mod engine;
mod eval;
mod moves;
mod search;
mod types;

use crate::bench::bench;
use crate::engine::uci::main_loop;
use std::env;

fn main() {
    let args = env::args().collect::<Vec<_>>();
    if args.len() > 1 && args[1] == "bench" {
        bench();
    } else {
        main_loop();
    }
}
