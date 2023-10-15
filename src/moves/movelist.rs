use crate::{board::board::Board, search::killers::NUM_KILLER_MOVES};
use std::mem::MaybeUninit;

use super::moves::{Move, Promotion};

pub const MAX_LEN: usize = 218;
#[derive(Copy, Clone, Debug)]
/// Movelist elements contains a move and an i32 where a score can be stored later to be used in move ordering
/// for efficient search pruning
pub struct MoveList {
    arr: [MoveListEntry; MAX_LEN],
    len: usize,
    current_idx: usize,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct MoveListEntry {
    pub m: Move,
    pub score: u32,
}

impl MoveListEntry {
    fn new(m: Move, score: u32) -> Self {
        MoveListEntry { m, score }
    }
}

impl MoveList {
    #[inline(always)]
    pub fn push(&mut self, m: Move) {
        debug_assert!(self.len < MAX_LEN);
        debug_assert_ne!(m, Move::NULL);
        self.arr[self.len] = MoveListEntry::new(m, 0);
        self.len += 1;
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[inline(always)]
    pub fn has_next(&self) -> bool {
        self.current_idx < self.len
    }

    #[inline(always)]
    pub fn append(&mut self, other: &MoveList) {
        for idx in 0..other.len {
            self.push(other.arr[idx].m);
        }
    }

    #[inline(always)]
    pub fn swap(&mut self, a: usize, b: usize) {
        unsafe {
            let ptr_a: *mut MoveListEntry = &mut self.arr[a];
            let ptr_b: *mut MoveListEntry = &mut self.arr[b];
            std::ptr::swap(ptr_a, ptr_b);
        }
    }

    #[inline(always)]
    pub fn get_one(&mut self, idx: usize) -> Option<MoveListEntry> {
        if !self.has_next() {
            return None;
        }
        let e = self.pick_move(idx);
        Some(e)
    }

    #[inline(always)]
    /// Sorts next move into position and then returns a reference to the move
    fn pick_move(&mut self, idx: usize) -> MoveListEntry {
        self.sort_next_move(idx);
        self.get_move(idx)
    }

    #[inline(always)]
    pub fn get_move(&self, idx: usize) -> MoveListEntry {
        self.arr[idx]
    }

    #[inline(always)]
    pub fn get_score(&self, idx: usize) -> u32 {
        self.arr[idx].score
    }

    #[inline(always)]
    pub fn get_mut(&mut self, idx: usize) -> &mut MoveListEntry {
        &mut self.arr[idx]
    }

    #[inline(always)]
    pub fn into_vec(self) -> Vec<Move> {
        let mut v = Vec::new();
        self.into_iter().for_each(|x| v.push(x));
        v
    }

    pub fn score_move_list(&mut self, board: &Board, table_move: Move, killers: &[Move; NUM_KILLER_MOVES]) {
        for i in 0..self.len {
            let entry = self.get_mut(i);
            let m = &mut entry.m;
            let m_score = &mut entry.score;
            let piece_moving = board.piece_at(m.origin_square()).unwrap();
            let capture = board.piece_at(m.dest_square());
            let promotion = m.promotion();
            let mut score = 0;
            if m == &table_move {
                score = SCORED_MOVE_OFFSET + TTMOVE_SORT_VAL;
            } else if let Some(promotion) = promotion {
                if promotion == Promotion::Queen {
                    score = SCORED_MOVE_OFFSET + QUEEN_PROMOTION;
                }
            } else if let Some(capture) = capture {
                score = SCORED_MOVE_OFFSET + MVV_LVA[capture as usize][piece_moving as usize];
            } else {
                let mut n = 0;
                while n < NUM_KILLER_MOVES && score == 0 {
                    let killer_move = killers[n];
                    if *m == killer_move {
                        score = SCORED_MOVE_OFFSET - ((i as u32 + 1) * KILLER_VAL);
                    }
                    n += 1;
                }
            }
            *m_score = score;
        }
    }

    pub fn sort_next_move(&mut self, idx: usize) {
        let mut max_idx = idx;
        for i in (idx + 1)..self.len {
            if self.get_score(i) > self.get_score(max_idx) {
                max_idx = i;
            }
        }
        self.swap(max_idx, idx);
    }
}

const KILLER_VAL: u32 = 10;
const SCORED_MOVE_OFFSET: u32 = 1000;
const QUEEN_PROMOTION: u32 = 67;
const TTMOVE_SORT_VAL: u32 = 70;
// Most valuable victim, least valuable attacker
// Table is addressed from table[victim][capturer]
// Ex second row is each piece attacking a queen w/ the later columns being less valuable pieces
pub const MVV_LVA: [[u32; 6]; 6] = [
    [60, 61, 62, 63, 64, 65], // victim K
    [50, 51, 52, 53, 54, 55], // victim Q
    [40, 41, 42, 43, 44, 45], // victim R
    [30, 31, 32, 33, 34, 35], // victim B
    [20, 21, 22, 23, 24, 25], // victim K
    [10, 11, 12, 13, 14, 15], // victim P
];

impl ExactSizeIterator for MoveList {
    fn len(&self) -> usize {
        self.len
    }
}

impl Iterator for MoveList {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_idx >= self.len {
            None
        } else {
            let m = self.pick_move(self.current_idx);
            self.current_idx += 1;
            Some(m.m)
        }
    }
}

impl FromIterator<Move> for MoveList {
    fn from_iter<I: IntoIterator<Item = Move>>(iter: I) -> Self {
        let mut move_list = MoveList::default();
        for m in iter {
            move_list.push(m);
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
