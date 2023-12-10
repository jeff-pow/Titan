pub const MAX_LEN: usize = 500;
#[derive(Clone, Copy, Debug, PartialEq)]
/// u64list elements contains a move and an i32 where a score can be stored later to be used in move ordering
/// for efficient search pruning
pub struct BoardHistory {
    pub arr: [u64; MAX_LEN],
}
