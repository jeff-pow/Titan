use std::mem::MaybeUninit;

pub const MAX_LEN: usize = 500;
#[derive(Clone, Copy, Debug, PartialEq)]
/// u64list elements contains a move and an i32 where a score can be stored later to be used in move ordering
/// for efficient search pruning
pub struct BoardHistory {
    pub arr: [u64; MAX_LEN],
    pub len: usize,
}

impl BoardHistory {
    pub fn push(&mut self, hash: u64) {
        debug_assert!(self.len < MAX_LEN);
        if self.len >= MAX_LEN - 1 {
            return;
        }
        self.arr[self.len] = hash;
        self.len += 1;
    }

    /// Function checks for the presence of the board in the game. If the board position will have occurred three times,
    /// returns true indicating the position would be a stalemate due to the threefold repetition rule
    pub fn check_for_3x_repetition(&self, hash: u64) -> bool {
        let len = self.len;
        let mut count = 0;
        for i in (0..len).rev() {
            if self.arr[i] == hash {
                count += 1;
            }
        }
        count > 1
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
