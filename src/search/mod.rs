use std::ops::{Index, IndexMut};

use lazy_static::lazy_static;

use crate::moves::movelist::MAX_LEN;
use crate::moves::moves::Move;

use self::search::MAX_SEARCH_DEPTH;

pub mod game_time;
pub mod history_table;
pub mod quiescence;
pub mod search;
pub mod see;
pub mod thread;

// Tunable Constants
/// Initial aspiration window value
pub const INIT_ASP: i32 = 10;
const MIN_ASP_DEPTH: i32 = 4;
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

#[derive(Clone, Copy, Default)]
pub(super) struct PlyEntry {
    pub killers: [Move; 2],
    pub played_move: Move,
    pub static_eval: i32,
}

#[derive(Clone, Copy)]
struct PV {
    line: [Move; MAX_SEARCH_DEPTH as usize],
    len: usize,
}

impl PV {
    fn update(&mut self, m: Move, other: PV) {
        self.line[0] = m;
        self.line[1..=other.len].copy_from_slice(&other.line[..other.len]);
        self.len = other.len + 1;
    }
}

impl Default for PV {
    fn default() -> Self {
        Self { line: [Move::NULL; MAX_SEARCH_DEPTH as usize], len: Default::default() }
    }
}

#[derive(Clone)]
pub(crate) struct SearchStack {
    stack: [PlyEntry; MAX_SEARCH_DEPTH as usize],
}

impl SearchStack {
    pub fn prev_move(&self, ply: i32) -> Move {
        self.stack.get(ply as usize).map_or(Move::NULL, |e| e.played_move)
    }
}

impl Default for SearchStack {
    fn default() -> Self {
        Self { stack: [PlyEntry::default(); MAX_SEARCH_DEPTH as usize] }
    }
}

impl Index<i32> for SearchStack {
    type Output = PlyEntry;

    fn index(&self, index: i32) -> &Self::Output {
        &self.stack[index as usize]
    }
}

impl IndexMut<i32> for SearchStack {
    fn index_mut(&mut self, index: i32) -> &mut Self::Output {
        &mut self.stack[index as usize]
    }
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
