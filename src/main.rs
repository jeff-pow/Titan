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
use crate::engine::uci::main_loop;
use crate::moves::movepicker::MovePicker;
use moves::movelist::MoveListEntry;
use search::lmr_table::LmrTable;
use search::thread::ThreadData;
use std::env;
use std::sync::atomic::{AtomicBool, AtomicU64};

fn main() {
    dbg!(Move(656132));
    // let mut b = Board::default();
    // b.make_move::<false>(Move::from_san("b1c3", &b));
    // b.make_move::<false>(Move::from_san("e7e5", &b));
    // b.make_move::<false>(Move::from_san("e2e4", &b));
    // b.make_move::<false>(Move::from_san("b8c6", &b));
    // b.make_move::<false>(Move::from_san("g1f3", &b));
    // b.make_move::<false>(Move::from_san("f8c5", &b));
    // b.make_move::<false>(Move::from_san("f1e2", &b));
    // b.make_move::<false>(Move::from_san("g8f6", &b));
    // b.make_move::<false>(Move::from_san("d2d3", &b));
    // b.make_move::<false>(Move::from_san("e8g8", &b));
    // b.make_move::<false>(Move::from_san("e1g1", &b));
    // let mut picker = MovePicker {
    //     phase: moves::movepicker::MovePickerPhase::TTMove,
    //     skip_quiets: false,
    //     margin: -100,
    //     moves: moves::movelist::MoveList::default(),
    //     index: 0,
    //     tt_move: Move(0),
    //     killer_move: Move(749500),
    //     counter_move: Move(68339),
    // };
    // let lmr = LmrTable::new();
    // let global_nodes = AtomicU64::new(0);
    // let binding = AtomicBool::new(false);
    // let td = ThreadData::new(&binding, Vec::new(), 0, &lmr, &global_nodes);
    // while let Some(MoveListEntry { m, score: _hist_score }) = picker.next(&b, &td) {
    //     let mut new_b = b;
    //     let s = m.to_san();
    //     dbg!(s);
    //     let q = b.is_pseudo_legal(m);
    //     if !b.is_legal(m) {
    //         continue;
    //     }
    //     new_b.make_move::<true>(m);
    // }
    if env::args().any(|x| x == *"bench") {
        bench();
    } else {
        main_loop();
    }
}
