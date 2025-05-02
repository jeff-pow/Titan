use arrayvec::ArrayVec;
use std::{
    array,
    ops::{Index, IndexMut},
};

use self::{game_time::Clock, search::MAX_PLY};
use crate::{chess_move::Move, types::pieces::Piece};

pub mod game_time;
pub mod lmr_table;
pub mod search;

#[derive(Clone, Copy)]
pub struct PlyEntry {
    pub killer_move: Option<Move>,
    pub played_move: Option<Move>,
    pub moved_piece: Piece,
    pub static_eval: i32,
    pub cutoffs: u32,
    pub singular: Option<Move>,
    /// Double extensions
    pub multi_extns: i32,
}

impl Default for PlyEntry {
    fn default() -> Self {
        Self {
            killer_move: Default::default(),
            played_move: Default::default(),
            moved_piece: Piece::None,
            static_eval: Default::default(),
            cutoffs: Default::default(),
            singular: Default::default(),
            multi_extns: Default::default(),
        }
    }
}

pub struct PVTable {
    table: [ArrayVec<Option<Move>, { MAX_PLY + 1 }>; MAX_PLY + 1],
}

impl PVTable {
    pub fn best_move(&self) -> Move {
        self.table[0][0].unwrap()
    }

    pub fn pv(&self) -> &[Option<Move>] {
        &self.table[0]
    }

    pub fn clear(&mut self) {
        self.table.iter_mut().for_each(|pv| pv.clear());
    }

    pub fn append(&mut self, m: Option<Move>, ply: usize) {
        self.table[ply].clear();
        self.table[ply].push(m);
        let (lower, upper) = self.table.split_at_mut(ply);
        lower.last_mut().unwrap().extend(upper.first().unwrap());
    }
}

impl Default for PVTable {
    fn default() -> Self {
        Self { table: array::from_fn(|_| ArrayVec::new_const()) }
    }
}

#[derive(Clone)]
pub struct SearchStack {
    stack: [PlyEntry; MAX_PLY as usize + 5],
}

impl SearchStack {
    pub fn prev(&self, ply: i32) -> Option<(Move, Piece)> {
        self.stack.get(ply as usize).and_then(|e| e.played_move.map(|m| (m, e.moved_piece)))
    }
}

impl Default for SearchStack {
    fn default() -> Self {
        Self { stack: [PlyEntry::default(); MAX_PLY as usize + 5] }
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
