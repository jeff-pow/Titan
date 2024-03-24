#![allow(clippy::module_inception)]
#![allow(clippy::cast_possible_truncation)]
#![allow(long_running_const_eval)]
#![cfg_attr(feature = "avx512", feature(stdarch_x86_avx512))]

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
    if env::args().any(|x| x == *"bench") {
        bench();
    } else {
        main_loop();
    }
}
