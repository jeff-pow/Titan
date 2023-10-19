use crate::{
    board::board::Board,
    search::{killers::NUM_KILLER_MOVES, see::see, SearchInfo},
    types::pieces::PieceName,
};
use std::mem::MaybeUninit;

use super::moves::{Move, Promotion};

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
    #[inline(always)]
    pub fn push(&mut self, m: Move) {
        debug_assert!(self.len < MAX_LEN);
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
        if idx >= self.len {
            return None;
        }
        Some(self.pick_move(idx))
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
    pub fn get_score(&self, idx: usize) -> i32 {
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

    pub fn sort_next_move(&mut self, idx: usize) {
        let mut max_idx = idx;
        for i in (idx + 1)..self.len {
            if self.get_score(i) > self.get_score(max_idx) {
                max_idx = i;
            }
        }
        self.swap(max_idx, idx);
    }

    pub fn score_moves(
        &mut self,
        board: &Board,
        table_move: Move,
        killers: &[Move; NUM_KILLER_MOVES],
        info: &SearchInfo,
    ) {
        for i in 0..self.len {
            let entry = self.get_mut(i);
            let m = &mut entry.m;
            let score = &mut entry.score;
            let piece_moving = board.piece_at(m.origin_square()).unwrap();
            let capture = board.piece_at(m.dest_square());
            let promotion = m.promotion();
            if *m == table_move {
                *score = TTMOVE;
            } else if let Some(promotion) = promotion {
                match promotion {
                    Promotion::Queen => *score = QUEEN_PROMOTION,
                    _ => *score = BAD_PROMOTION,
                }
            } else if capture.is_some() || m.is_en_passant() {
                let captured_piece = if m.is_en_passant() {
                    PieceName::Pawn
                } else {
                    board.piece_at(m.dest_square()).expect("There is a piece here")
                };
                if see(board, *m, -PieceName::Pawn.value()) {
                    *score = GOOD_CAPTURE + MVV_LVA[piece_moving as usize][captured_piece as usize];
                } else {
                    *score = BAD_CAPTURE + MVV_LVA[piece_moving as usize][captured_piece as usize];
                }
            } else if killers[0] == *m {
                *score = KILLER_ONE;
            } else if killers[1] == *m {
                *score = KILLER_TWO;
            } else {
                *score = info.history.get_history(*m, board.to_move);
            }
        }
    }
}

const QUEEN_PROMOTION: i32 = 20000001;
const GOOD_CAPTURE: i32 = 3000000;
const KILLER_ONE: i32 = 2000000;
const KILLER_TWO: i32 = 1000000;
const BAD_CAPTURE: i32 = -10000;
const BAD_PROMOTION: i32 = -20000001;
const TTMOVE: i32 = i32::MAX - 1000;
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
