use std::mem::MaybeUninit;
use crate::{
    board::board::Board,
    search::{killers::NUM_KILLER_MOVES, SearchInfo},
};

use super::moves::Move;

pub const MAX_LEN: usize = 218;
#[derive(Copy, Clone, Debug)]
/// Movelist elements contains a move and an i32 where a score can be stored later to be used in move ordering
/// for efficient search pruning
pub struct MoveList {
    pub arr: [(Move, u32); MAX_LEN],
    pub len: usize,
}

impl MoveList {
    #[inline(always)]
    pub fn push(&mut self, m: Move) {
        debug_assert!(self.len < MAX_LEN);
        self.arr[self.len] = (m, 0);
        self.len += 1;
    }

    #[inline(always)]
    pub fn append(&mut self, other: &MoveList) {
        for idx in 0..other.len {
            self.push(other.arr[idx].0);
        }
    }

    #[inline(always)]
    pub fn swap(&mut self, a: usize, b: usize) {
        unsafe {
            let ptr_a: *mut (Move, u32) = &mut self.arr[a];
            let ptr_b: *mut (Move, u32) = &mut self.arr[b];
            std::ptr::swap(ptr_a, ptr_b);
        }
    }

    #[inline(always)]
    pub fn iter(&self) -> MoveIter {
        MoveIter {
            movelist: self,
            curr: 0,
        }
    }

    #[inline(always)]
    pub fn get_move(&self, idx: usize) -> &Move {
        &self.arr[idx].0
    }

    #[inline(always)]
    pub fn get_score(&self, idx: usize) -> u32 {
        self.arr[idx].1
    }

    #[inline(always)]
    pub fn get_mut(&mut self, idx: usize) -> &mut (Move, u32) {
        &mut self.arr[idx]
    }

    #[inline(always)]
    pub fn into_vec(self) -> Vec<Move> {
        let mut v = Vec::new();
        for i in 0..self.len {
            v.push(*self.get_move(i));
        }
        v
    }
}

const KILLER_VAL: u32 = 10;
const MVV_LVA_OFFSET: u32 = u32::MAX - 256;
const TTMOVE_SORT_VALUE: u32 = 60;

pub const MVV_LVA: [[u32; 7]; 7] = [
    [0, 0, 0, 0, 0, 0, 0],       // victim K, attacker K, Q, R, B, N, P, None
    [50, 51, 52, 53, 54, 55, 0], // victim Q, attacker K, Q, R, B, N, P, None
    [40, 41, 42, 43, 44, 45, 0], // victim R, attacker K, Q, R, B, N, P, None
    [30, 31, 32, 33, 34, 35, 0], // victim B, attacker K, Q, R, B, N, P, None
    [20, 21, 22, 23, 24, 25, 0], // victim K, attacker K, Q, R, B, N, P, None
    [10, 11, 12, 13, 14, 15, 0], // victim P, attacker K, Q, R, B, N, P, None
    [0, 0, 0, 0, 0, 0, 0],       // victim None, attacker K, Q, R, B, N, P, None
];

pub fn score_move_list(
    ply: i8,
    board: &Board,
    moves: &mut MoveList,
    table_move: Move,
    search_info: &SearchInfo,
) {
    for i in 0..moves.len {
        let (m, m_score) = moves.get_mut(i);
        let piece_moving = board.piece_on_square(m.origin_square()).unwrap();
        let capture = board.piece_on_square(m.dest_square());
        let mut score = 0;
        if m == &table_move {
            score = MVV_LVA_OFFSET + TTMOVE_SORT_VALUE;
        } else if let Some(capture) = capture {
            score = MVV_LVA_OFFSET + MVV_LVA[capture as usize][piece_moving as usize];
        } else {
            let mut n = 0;
            while n < NUM_KILLER_MOVES && score == 0 {
                let killer_move = search_info.killer_moves[ply as usize][n];
                if *m == killer_move {
                    score = MVV_LVA_OFFSET - ((i as u32 + 1) * KILLER_VAL);
                }
                n += 1;
            }
        }
        *m_score = score;
    }
}

pub fn sort_next_move(moves: &mut MoveList, idx: usize) {
    // for i in (idx + 1)..moves.len {
    //     if moves.get_score(i) > moves.get_score(idx) {
    //         moves.swap(idx, i);
    //     }
    // }
    let mut max_idx = idx;
    for i in (idx + 1)..moves.len {
        if moves.get_score(max_idx) < moves.get_score(i) {
            max_idx = i;
        }
    }
    moves.swap(max_idx, idx);
}

pub struct MoveIter<'a> {
    movelist: &'a MoveList,
    curr: usize,
}

impl Iterator for MoveIter<'_> {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        if self.curr >= self.movelist.len {
            None
        } else {
            let m = self.movelist.arr[self.curr];
            self.curr += 1;
            Some(m.0)
        }
    }
}

impl FromIterator<Move> for MoveList {
    fn from_iter<I: IntoIterator<Item = Move>>(iter: I) -> Self {
        let mut move_list = MoveList::default();

        for m in iter {
            move_list.arr[move_list.len] = (m, 0);
            move_list.len += 1;
        }

        move_list
    }
}

impl Default for MoveList {
    fn default() -> Self {
        Self {
            // arr: [(Move::NULL, 0); MAX_LEN],
            arr: unsafe { MaybeUninit::uninit().assume_init() },
            len: 0,
        }
    }
}
