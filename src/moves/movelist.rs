use crate::{
    board::board::Board,
    search::{killers::NUM_KILLER_MOVES, ThreadData},
    types::pieces::PieceName,
};
use std::{mem::MaybeUninit, ops::Index};

use super::moves::Move;

pub const MAX_LEN: usize = 218;
#[derive(Copy, Clone, Debug)]
/// Movelist elements contains a move and an i32 where a score can be stored later to be used in move ordering
/// for efficient search pruning
pub struct MoveList {
    pub arr: [MoveListEntry; MAX_LEN],
    len: usize,
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
        debug_assert!(self.len < MAX_LEN);
        self.arr[self.len] = MoveListEntry::new(m, 0);
        self.len += 1;
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    fn swap(&mut self, a: usize, b: usize) {
        unsafe {
            let ptr_a: *mut MoveListEntry = &mut self.arr[a];
            let ptr_b: *mut MoveListEntry = &mut self.arr[b];
            std::ptr::swap(ptr_a, ptr_b);
        }
    }

    /// Sorts next move into position and then returns a reference to the move
    fn pick_move(&mut self, idx: usize) -> MoveListEntry {
        self.sort_next_move(idx);
        self.arr[idx]
    }

    fn sort_next_move(&mut self, idx: usize) {
        let mut max_idx = idx;
        for i in (idx + 1)..self.len {
            if self.arr[i].score > self.arr[max_idx].score {
                max_idx = i;
            }
        }
        self.swap(max_idx, idx);
    }

    pub fn score_moves(&mut self, board: &Board, table_move: Move, killers: [Move; NUM_KILLER_MOVES], td: &ThreadData) {
        for i in 0..self.len {
            let entry = &mut self.arr[i];
            let prev = td.current_line.last().unwrap_or(&Move::NULL);
            let counter = td.history.get_counter(board.to_move, *prev);
            entry.score = if entry.m == table_move {
                TTMOVE
            } else if let Some(promotion) = entry.m.promotion() {
                match promotion {
                    PieceName::Queen => QUEEN_PROMOTION,
                    _ => BAD_PROMOTION,
                }
            } else if let Some(c) = board.capture(entry.m) {
                (if board.see(entry.m, -PieceName::Pawn.value()) {
                    GOOD_CAPTURE
                } else {
                    BAD_CAPTURE
                }) + MVV_LVA[board.piece_at(entry.m.origin_square()).unwrap()][c]
                    + td.history.capt_hist(entry.m, board.to_move, c)
            } else if killers[0] == entry.m {
                KILLER_ONE
            } else if killers[1] == entry.m {
                KILLER_TWO
            } else if counter == entry.m {
                COUNTER_MOVE
            } else {
                td.history.quiet_history(entry.m, board.to_move)
            };
        }
    }
}

const TTMOVE: i32 = i32::MAX - 1000;
pub const GOOD_CAPTURE: i32 = 3000000;
pub const BAD_CAPTURE: i32 = -10000;
const KILLER_ONE: i32 = 2000000;
const KILLER_TWO: i32 = 1000000;
const COUNTER_MOVE: i32 = 900000;
const QUEEN_PROMOTION: i32 = 20000001;
const BAD_PROMOTION: i32 = -20000001;
/// [Attacker][Victim]
#[rustfmt::skip]
const MVV_LVA: [[i32; 6]; 6] = [
// Victims
//   K   Q   R   B   N   P       Attacker
    [60, 50, 40, 30, 20, 10], // K
    [61, 51, 41, 31, 21, 11], // Q
    [62, 52, 42, 32, 22, 12], // R
    [63, 53, 43, 33, 23, 13], // B
    [64, 54, 44, 34, 24, 14], // N
    [65, 55, 45, 35, 25, 15], // P
];

impl Index<usize> for MoveList {
    type Output = Move;

    fn index(&self, index: usize) -> &Self::Output {
        &self.arr[index].m
    }
}

impl Iterator for MoveList {
    type Item = MoveListEntry;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_idx >= self.len {
            None
        } else {
            let m = self.pick_move(self.current_idx);
            self.current_idx += 1;
            Some(m)
        }
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
        let arr: MaybeUninit<[MoveListEntry; MAX_LEN]> = MaybeUninit::uninit();
        Self {
            arr: unsafe { arr.assume_init() },
            len: 0,
            current_idx: 0,
        }
    }
}
