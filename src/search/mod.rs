use arr_macro::arr;
use arrayvec::ArrayVec;
use std::ops::{Index, IndexMut};
use std::sync::atomic::{AtomicI32, Ordering};

use crate::eval::accumulator::Accumulator;
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

#[derive(Clone, Copy, Default)]
pub(super) struct PlyEntry {
    pub killer_move: Move,
    pub played_move: Move,
    pub static_eval: i32,
    pub singular: Move,
    /// Double extensions
    pub dbl_extns: i32,
}

#[derive(Default)]
struct PV {
    line: ArrayVec<Move, { MAX_SEARCH_DEPTH as usize }>,
}

impl PV {
    fn update(&mut self, m: Move, other: PV) {
        self.line.clear();
        self.line.push(m);
        self.line.extend(other.line);
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
    LMR_REDUCTIONS[0][0].store(0, Ordering::Relaxed);
    LMR_REDUCTIONS[1][0].store(0, Ordering::Relaxed);
    LMR_REDUCTIONS[0][1].store(0, Ordering::Relaxed);
}

pub fn get_reduction(depth: i32, moves_played: i32) -> i32 {
    LMR_REDUCTIONS[depth.min(MAX_SEARCH_DEPTH) as usize][(moves_played as usize).min(MAX_LEN)]
        .load(Ordering::Relaxed)
}
#[derive(Clone)]
pub struct AccumulatorStack {
    pub(crate) stack: Vec<Accumulator>,
}

impl AccumulatorStack {
    pub fn increment(&mut self) {
        self.stack.push(*self.stack.last().unwrap());
    }

    pub fn top(&mut self) -> &mut Accumulator {
        self.stack.last_mut().unwrap()
    }

    pub fn pop(&mut self) -> Accumulator {
        self.stack.pop().unwrap()
    }

    pub fn push(&mut self, acc: Accumulator) {
        self.stack.push(acc)
    }

    pub fn new(base_accumulator: Accumulator) -> Self {
        let mut vec = Vec::with_capacity(MAX_SEARCH_DEPTH as usize + 50);
        vec.push(base_accumulator);
        Self { stack: vec }
    }
}
