
use arrayvec::ArrayVec;

use crate::{
    board::board::Board,
    search::{thread::ThreadData, NUM_KILLER_MOVES},
    types::pieces::PieceName,
};
use std::ops::Index;


use super::moves::Move;

pub const MAX_LEN: usize = 218;
#[derive(Clone, Debug)]
/// Movelist elements contains a move and an i32 where a score can be stored later to be used in move ordering
/// for efficient search pruning
pub struct MoveList {
    // pub arr: [MoveListEntry; MAX_LEN],
    pub arr: ArrayVec<MoveListEntry, MAX_LEN>,
    current_idx: usize,

}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct MoveListEntry {
    pub m: Move,
    pub score: i32,
}

impl MoveListEntry {
    fn new(m: Move, score: i32) -> Self {
        MoveListEntry { m, score }
    }
}

impl MoveList {
    pub fn push(&mut self, m: Move) {
        self.arr.push(MoveListEntry::new(m, 0));
    }


    fn swap(&mut self, a: usize, b: usize) {
        self.arr.swap(a, b);
    }

    /// Sorts next move into position and then returns the move entry
    pub(super) fn pick_move(&mut self, idx: usize) -> MoveListEntry {
        self.sort_next_move(idx);
        self.arr[idx]
    }

    fn sort_next_move(&mut self, idx: usize) {
        let mut max_idx = idx;
        for i in (idx + 1)..self.arr.len() {
            if self.arr[i].score > self.arr[max_idx].score {
                max_idx = i;
            }
        }

        self.swap(max_idx, idx);
    }
}

impl Index<usize> for MoveList {
    type Output = Move;

    fn index(&self, index: usize) -> &Self::Output {
        &self.arr[index].m
    }
}

impl FromIterator<MoveListEntry> for MoveList {
    fn from_iter<I: IntoIterator<Item = MoveListEntry>>(iter: I) -> Self {
        let mut move_list = MoveList::default();
        for m in iter {
            move_list.push(m.m);
        }
        move_list
    }
}

impl Default for MoveList {
    fn default() -> Self {
        // Uninitialized memory is much faster than initializing it when the important stuff will
        // be written over anyway ;)
        Self { arr: ArrayVec::new(), current_idx: 0 }
    }
}
