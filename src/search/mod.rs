use arr_macro::arr;
use arrayvec::ArrayVec;
use std::ops::{Index, IndexMut};
use std::sync::atomic::{AtomicI32, Ordering};

use crate::moves::movelist::MAX_LEN;
use crate::moves::moves::Move;
use crate::spsa::{LMR_BASE, LMR_DIVISOR};

use self::search::MAX_SEARCH_DEPTH;

pub mod game_time;
pub mod history_table;
pub mod quiescence;
pub mod search;
pub mod see;
pub mod thread;

pub const NUM_KILLER_MOVES: usize = 2;

#[derive(Clone, Copy, Default)]
pub(super) struct PlyEntry {
    pub killers: [Move; 2],
    pub played_move: Move,
    pub static_eval: i32,
    pub singular: Move,
    /// Double extensions
    pub dbl_extns: i32,
}

#[derive(Clone, Default)]
struct PV {
    line: ArrayVec<Move, { MAX_SEARCH_DEPTH as usize }>,
}

impl PV {
    fn update(&mut self, m: Move, other: PV) {
        self.line.clear();
        self.line.push(m);
        assert!(self.line.try_extend_from_slice(&other.line).is_ok());
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

type LmrReductions = [[AtomicI32; MAX_LEN + 1]; (MAX_SEARCH_DEPTH + 1) as usize];

pub static LMR_REDUCTIONS: LmrReductions = arr![arr![AtomicI32::new(0); 219]; 101];

pub fn lmr_reductions() {
    for depth in 0..MAX_SEARCH_DEPTH + 1 {
        for moves_played in 0..MAX_LEN + 1 {
            let reduction = (LMR_BASE.val() as f32 / 100.
                + (depth as f32).ln() * (moves_played as f32).ln()
                    / (LMR_DIVISOR.val() as f32 / 100.)) as i32;
            LMR_REDUCTIONS[depth as usize][moves_played].store(reduction, Ordering::Relaxed);
        }
    }
}

pub fn get_reduction(depth: i32, moves_played: i32) -> i32 {
    LMR_REDUCTIONS[depth as usize][moves_played as usize].load(Ordering::Relaxed)
}
