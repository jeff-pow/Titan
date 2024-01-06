use arrayvec::ArrayVec;

use crate::{
    board::board::Board,
    search::{thread::ThreadData, NUM_KILLER_MOVES},
    types::pieces::{Piece, PieceName},
};
use std::ops::Index;

use super::moves::Move;

pub const MAX_LEN: usize = 218;
#[derive(Clone, Debug)]
/// Movelist elements contains a move and an i32 where a score can be stored later to be used in move ordering
/// for efficient search pruning
pub struct MoveList {
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

    pub fn len(&self) -> usize {
        self.arr.len()
    }

    /// Sorts next move into position and then returns a reference to the move
    pub(super) fn pick_move(&mut self, idx: usize) -> MoveListEntry {
        self.sort_next_move(idx);
        self.arr[idx]
    }

    fn sort_next_move(&mut self, idx: usize) {
        let mut max_idx = idx;
        for i in (idx + 1)..self.len() {
            if self.arr[i].score > self.arr[max_idx].score {
                max_idx = i;
            }
        }
        self.arr.swap(max_idx, idx);
    }

    pub(crate) fn score_moves(
        &mut self,
        board: &Board,
        table_move: Move,
        killers: [Move; NUM_KILLER_MOVES],
        td: &ThreadData,
    ) {
        for i in 0..self.len() {
            let entry = &mut self.arr[i];
            let prev = td.stack.prev_move(td.ply - 1);
            let counter = td.history.get_counter(prev);
            entry.score = if entry.m == table_move {
                TTMOVE
            } else if let Some(promotion) = entry.m.promotion() {
                match promotion.name() {
                    PieceName::Queen => QUEEN_PROMOTION + td.history.capt_hist(entry.m, board),
                    _ => BAD_PROMOTION,
                }
            } else if board.capture(entry.m) != Piece::None {
                let c = board.capture(entry.m);
                // TODO: Try a threshold of 0 or 1 here
                assert_ne!(c.name(), PieceName::King);
                (if board.see(entry.m, -PieceName::Pawn.value()) {
                    GOOD_CAPTURE
                } else {
                    BAD_CAPTURE
                }) + MVV[c.name()]
                    + td.history.capt_hist(entry.m, board)
            } else if killers[0] == entry.m {
                KILLER_ONE
            } else if killers[1] == entry.m {
                KILLER_TWO
            } else if counter == entry.m {
                COUNTER_MOVE
            } else {
                td.history.quiet_history(entry.m, &td.stack, td.ply)
            };
        }
    }

    #[allow(dead_code)]
    pub(super) fn assert_no_duplicates(&self) {
        for m in self.arr.iter().take(self.len()) {
            assert!(
                self.arr.iter().take(self.len()).filter(|&x| x == m).count() == 1,
                "{}",
                m.m.to_san()
            );
        }
    }
}

const MVV: [i32; 6] = [0, 2400, 2400, 4800, 9600, 0];
const TTMOVE: i32 = i32::MAX - 1000;
const QUEEN_PROMOTION: i32 = 20_000_001;
pub const GOOD_CAPTURE: i32 = 10_000_000;
const KILLER_ONE: i32 = 1_000_000;
const KILLER_TWO: i32 = 900_000;
const COUNTER_MOVE: i32 = 800_000;
pub const BAD_CAPTURE: i32 = -10000;
const BAD_PROMOTION: i32 = -QUEEN_PROMOTION;

impl Index<usize> for MoveList {
    type Output = Move;

    fn index(&self, index: usize) -> &Self::Output {
        &self.arr[index].m
    }
}

impl Iterator for MoveList {
    type Item = MoveListEntry;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_idx >= self.len() {
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
        Self { arr: ArrayVec::new(), current_idx: 0 }
    }
}
