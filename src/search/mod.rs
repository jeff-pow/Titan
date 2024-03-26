use arrayvec::ArrayVec;
use std::ops::{Index, IndexMut};

use crate::eval::accumulator::{Accumulator, Delta};
use crate::moves::moves::Move;

use self::game_time::Clock;
use self::search::MAX_SEARCH_DEPTH;

pub mod game_time;
pub mod history_table;
pub mod lmr_table;
pub mod quiescence;
pub mod search;
pub mod see;
pub mod thread;

#[derive(Clone, Copy, Default)]
pub struct PlyEntry {
    pub killer_move: Move,
    pub played_move: Move,
    pub static_eval: i32,
    pub cutoffs: u32,
    pub singular: Move,
    /// Double extensions
    pub dbl_extns: i32,
}

#[derive(Default)]
struct PV {
    line: ArrayVec<Move, { MAX_SEARCH_DEPTH as usize }>,
}

impl PV {
    fn update(&mut self, m: Move, other: Self) {
        self.line.clear();
        self.line.push(m);
        self.line.extend(other.line);
    }
}

#[derive(Clone)]
pub struct SearchStack {
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

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub enum SearchType {
    /// User has requested a search until a particular depth
    Depth(i32),
    /// Search determines how much time to allow itself
    Time(Clock),
    /// Only search for N nodes
    Nodes(u64),
    /// Search for a mate at the provided depth
    Mate(i32),
    #[default]
    /// Search forever
    Infinite,
}

#[derive(Clone)]
pub struct AccumulatorStack {
    pub(crate) stack: Vec<Accumulator>,
    pub top: usize,
}

impl AccumulatorStack {
    pub fn apply_update(&mut self, delta: &mut Delta) {
        let (bottom, top) = self.stack.split_at_mut(self.top + 1);
        top[0].lazy_ref_update(delta, bottom.last().unwrap());
        self.top += 1;
    }

    pub fn top(&mut self) -> &mut Accumulator {
        &mut self.stack[self.top]
    }

    pub fn pop(&mut self) -> Accumulator {
        self.top -= 1;
        self.stack[self.top + 1]
    }

    pub fn push(&mut self, acc: Accumulator) {
        self.top += 1;
        self.stack[self.top] = acc;
    }

    pub fn clear(&mut self, base_accumulator: &Accumulator) {
        self.stack[0] = *base_accumulator;
        self.top = 0;
    }

    pub fn new(base_accumulator: &Accumulator) -> Self {
        let mut vec = vec![Accumulator::default(); MAX_SEARCH_DEPTH as usize + 50];
        vec[0] = *base_accumulator;
        Self { stack: vec, top: 0 }
    }
}
