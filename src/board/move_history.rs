use std::mem::MaybeUninit;

pub const MAX_LEN: usize = 500;
#[derive(Clone, Copy)]
/// u64list elements contains a move and an i32 where a score can be stored later to be used in move ordering
/// for efficient search pruning
pub struct BoardHistory {
    pub arr: [u64; MAX_LEN],
    pub len: usize,
}

impl BoardHistory {
    #[inline(always)]
    pub fn push(&mut self, hash: u64) {
        debug_assert!(self.len < MAX_LEN);
        if self.len >= MAX_LEN - 1 {
            return;
        }
        self.arr[self.len] = hash;
        self.len += 1;
    }
}

impl Default for BoardHistory {
    fn default() -> Self {
        // Uninitialized memory is much faster than initializing it when the important stuff will
        // be written over anyway ;)
        let arr: MaybeUninit<[u64; MAX_LEN]> = MaybeUninit::uninit();
        Self {
            arr: unsafe { arr.assume_init() },
            len: 0,
        }
    }
}