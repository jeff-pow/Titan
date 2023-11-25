#![allow(clippy::module_inception)]
#![feature(stdsimd)]
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
    if args[1] == "bench" {
        bench();
    } else {
        main_loop();
    }

}
