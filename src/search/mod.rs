use std::sync::atomic::AtomicBool;

use lazy_static::lazy_static;

use crate::board::board::Board;
use crate::moves::movelist::MAX_LEN;
use crate::moves::moves::Move;

use self::search::MAX_SEARCH_DEPTH;
use self::{game_time::GameTime, search_stats::SearchStats};

pub mod game_time;
pub mod history_heuristics;
pub mod quiescence;
pub mod search;
pub mod search_stats;
pub mod see;
pub mod thread;

// Tunable Constants
/// Initial aspiration window value
pub const INIT_ASP: i32 = 10;
pub const NUM_KILLER_MOVES: usize = 2;
/// Begin LMR if more than this many moves have been searched
pub const LMR_THRESHOLD: i32 = 2;
pub const MIN_LMR_DEPTH: i32 = 2;
pub const MAX_LMP_DEPTH: i32 = 6;
pub const LMP_CONST: i32 = 3;
pub const RFP_MULTIPLIER: i32 = 70;
pub const MAX_RFP_DEPTH: i32 = 9;
pub const MIN_NMP_DEPTH: i32 = 3;
pub const MIN_IIR_DEPTH: i32 = 4;

#[derive(Clone)]
pub struct SearchInfo<'a> {
    pub board: Board,
    pub search_stats: SearchStats,
    pub game_time: GameTime,
    pub search_type: SearchType,
    pub max_depth: i32,
    pub halt: &'a AtomicBool,
    pub searching: &'a AtomicBool,
}

#[derive(Clone, Copy, Default)]
pub struct PlyEntry {
    pub killers: [Move; 2],
    pub played_move: Move,
    pub static_eval: i32,
}

#[derive(Clone, Copy, Default, PartialEq)]
pub enum SearchType {
    Depth, // User has requested a search until a particular depth
    Time,  // Search determines how much time to allow itself
    #[default]
    Infinite, // Search forever
}

lazy_static! {
    pub static ref LMR_REDUCTIONS: LmrReductions = lmr_reductions();
}

type LmrReductions = [[i32; MAX_LEN + 1]; (MAX_SEARCH_DEPTH + 1) as usize];
fn lmr_reductions() -> LmrReductions {
    let mut arr = [[0; MAX_LEN + 1]; (MAX_SEARCH_DEPTH + 1) as usize];
    for depth in 0..MAX_SEARCH_DEPTH + 1 {
        for moves_played in 0..MAX_LEN + 1 {
            arr[depth as usize][moves_played] = reduction(depth, moves_played as i32);
        }
    }
    arr
}

pub fn get_reduction(depth: i32, moves_played: i32) -> i32 {
    LMR_REDUCTIONS[depth as usize][moves_played as usize]
}

pub fn reduction(depth: i32, moves_played: i32) -> i32 {
    if depth <= MIN_LMR_DEPTH || moves_played < LMR_THRESHOLD {
        return 1;
    }
    let depth = depth as f32;
    let ply = moves_played as f32;
    let ret = 1. + depth.ln() * ply.ln() / 2.;
    ret as i32
}

pub fn store_pv(pvs: &mut Vec<Move>, node_pvs: &mut Vec<Move>, m: Move) {
    pvs.clear();
    pvs.push(m);
    pvs.append(node_pvs);
}
