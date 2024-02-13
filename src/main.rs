#![allow(clippy::module_inception)]
#![allow(long_running_const_eval)]
#![cfg_attr(feature = "avx512", feature(stdarch_x86_avx512))]
#[cfg(all(feature = "avx2", feature = "avx512"))]
compile_error!("Cannot enable both avx2 and avx512 simultaneously.");

mod bench;
mod board;
mod consts;
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
