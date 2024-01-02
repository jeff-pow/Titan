use self::nnue::Network;

pub(crate) mod accumulator;
pub mod nnue;

type Block = [i16; HIDDEN_SIZE];

#[cfg(feature = "simd")]
const CHUNK_SIZE: usize = 32;
#[cfg(feature = "simd")]
/// Number of SIMD vectors contained within one hidden layer
const REQUIRED_ITERS: usize = HIDDEN_SIZE / CHUNK_SIZE;

pub const INPUT_SIZE: usize = 768;
const HIDDEN_SIZE: usize = 1536;

static NET: Network = unsafe { std::mem::transmute(*include_bytes!("../../bins/181_screlu.bin")) };
