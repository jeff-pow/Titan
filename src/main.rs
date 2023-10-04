#![allow(clippy::module_inception)]
pub mod board;
pub mod engine;
pub mod eval;
pub mod moves;
pub mod search;
pub mod types;

use engine::uci::main_loop;
use moves::movelist::MAX_LEN;
use search::{pvs::MAX_SEARCH_DEPTH, SearchInfo};

use crate::search::{get_reduction, reduction};

fn main() {
    let s = SearchInfo::default();
    (0..MAX_SEARCH_DEPTH + 1).for_each(|i| {
        (0..MAX_LEN + 1).for_each(|j| assert_eq!(get_reduction(&s, i, j as i32), reduction(i, j as i32)));
    });
    // main_loop();
}
