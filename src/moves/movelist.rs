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
            if move_list.len >= MAX_LEN {
                break;
            }
            move_list.arr[move_list.len] = (m, 0);
            move_list.len += 1;
        }

        move_list
    }
}

impl Default for MoveList {
    fn default() -> Self {
        Self {
            arr: [(Move::NULL, 0); MAX_LEN],
            len: 0,
        }
    }
}
