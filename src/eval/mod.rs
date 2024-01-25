use self::nnue::Network;

pub(crate) mod accumulator;
pub mod nnue;
mod simd;

// TODO: perf list and align 64
type Block = [i16; HIDDEN_SIZE];

pub const INPUT_SIZE: usize = 768;
const HIDDEN_SIZE: usize = 1536;

static NET: Network = unsafe { std::mem::transmute(*include_bytes!("../../bins/181_screlu.bin")) };
