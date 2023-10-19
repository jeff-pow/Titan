use strum::IntoEnumIterator;

use crate::{
    board::board::Board,
    types::{
        pieces::{Color, PieceName},
        square::Square,
    },
};

pub const INPUT_SIZE: usize = 768;
const SCALE: i32 = 400;
const RELU_MIN: i16 = 0;
const RELU_MAX: i16 = i8::MAX as i16;
const LAYER_1_SIZE: usize = 1536;
#[derive(Clone, Copy)]
pub struct Accumulator {
    pub v: [[i16; LAYER_1_SIZE]; 2],
}

impl Accumulator {
    fn get(&self, color: Color) -> [i16; LAYER_1_SIZE] {
        self.v[color.idx()]
    }
}

#[repr(C)]
pub struct Network {
    pub feature_weights: [i16; INPUT_SIZE * LAYER_1_SIZE],
    pub feature_bias: [i16; LAYER_1_SIZE],
    pub output_weights: [i8; LAYER_1_SIZE * 2],
    pub ouput_bias: i16,
}

pub struct NetworkState {
    inputs: [[i16; INPUT_SIZE]; 2],
    king_squares: [Square; 2],
    accumulator: Accumulator,
}

const COLOR_OFFSET: usize = 64 * 6;
const PIECE_OFFSET: usize = 64;
fn feature_idx(color: Color, piece: PieceName, sq: Square) -> usize {
    color.idx() * COLOR_OFFSET + piece.idx() * PIECE_OFFSET + sq.idx()
}
