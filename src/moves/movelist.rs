use std::{mem::MaybeUninit, ops::Index};

use super::moves::Move;

pub const MAX_LEN: usize = 218;
#[derive(Copy, Clone, Debug)]
/// Movelist elements contains a move and an i32 where a score can be stored later to be used in move ordering
/// for efficient search pruning
pub struct MoveList {
    pub arr: [MoveListEntry; MAX_LEN],
    len: usize,
    _current_idx: usize,
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

    pub fn append(&mut self, other: MoveList) {
        assert!(self.len + other.len <= MAX_LEN);
        self.arr[self.len..self.len + other.len].copy_from_slice(&other.arr[..other.len]);
        self.len += other.len;
    }

    fn swap(&mut self, a: usize, b: usize) {
        unsafe {
            let ptr_a: *mut MoveListEntry = &mut self.arr[a];
            let ptr_b: *mut MoveListEntry = &mut self.arr[b];
            std::ptr::swap(ptr_a, ptr_b);
        }
    }

    /// Sorts next move into position and then returns the move entry
    pub(super) fn pick_move(&mut self, idx: usize) -> MoveListEntry {
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
        let arr: MaybeUninit<[MoveListEntry; MAX_LEN]> = MaybeUninit::uninit();
        Self { arr: unsafe { arr.assume_init() }, len: 0, _current_idx: 0 }
    }
}

#[cfg(test)]
mod movelist_test {
    use super::*;
    #[test]
    fn test_append_move_lists() {
        let mut move_list1 = MoveList::default();
        move_list1.push(Move(1));
        move_list1.push(Move(2));

        let mut move_list2 = MoveList::default();
        move_list2.push(Move(3));
        move_list2.push(Move(4));

        move_list1.append(move_list2);

        // Check if the combined length is as expected
        assert_eq!(move_list1.len(), 4);

        // Check if the elements are appended correctly
        let other = [
            MoveListEntry { m: Move(1), score: 0 },
            MoveListEntry { m: Move(2), score: 0 },
            MoveListEntry { m: Move(3), score: 0 },
            MoveListEntry { m: Move(4), score: 0 },
        ];
        assert_eq!(&move_list1.arr[..move_list1.len()], &other);
    }
}
