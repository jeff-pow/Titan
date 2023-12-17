#![allow(clippy::module_inception)]
#![allow(long_running_const_eval)]
#![cfg_attr(feature = "simd", feature(stdsimd))]

mod bench;
mod board;
mod engine;
mod eval;
mod moves;
mod search;
mod spsa;
mod types;

use search::lmr_reductions;
use spsa::print_ob_tunable_params;

use crate::bench::bench;
use crate::engine::uci::main_loop;
use std::env;

fn main() {
    lmr_reductions();
    print_ob_tunable_params();
    let args = env::args().collect::<Vec<_>>();
    if args.contains(&"bench".to_string()) {
        bench();
    } else {
        main_loop();
    }
}
