use arrayvec::ArrayVec;
use std::ops::{Index, IndexMut};

use crate::eval::accumulator::Accumulator;
use crate::moves::moves::Move;

use self::game_time::Clock;
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
    /// User has requested a search until a particular depth
    Depth(i32),
    /// Search determines how much time to allow itself
    Time(Clock),
    /// Only search for N nodes
    Nodes(u64),
    #[default]
    /// Search forever
    Infinite,
}

#[derive(Clone)]
pub struct AccumulatorStack {
    pub(crate) stack: Vec<Accumulator>,
}

impl AccumulatorStack {
    pub fn increment(&mut self) {
        self.stack.extend_from_within(self.stack.len() - 1..self.stack.len());
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

    pub fn clear(&mut self, base_accumulator: Accumulator) {
        assert!(self.stack.len() == 1);
        self.stack[0] = base_accumulator;
    }

    pub fn new(base_accumulator: Accumulator) -> Self {
        let mut vec = Vec::with_capacity(MAX_SEARCH_DEPTH as usize + 50);
        vec.push(base_accumulator);
        Self { stack: vec }
    }
}
