use crate::{
    board::board::Board,
    types::{
        pieces::{Color, PieceName, NUM_PIECES},
        square::{Square, NUM_SQUARES},
    },
};

pub const INPUT_SIZE: usize = 768;
const HIDDEN_SIZE: usize = 768;
const Q: i32 = 255 * 64;
const SCALE: i32 = 400;
const NET: Network = unsafe { std::mem::transmute(*include_bytes!("../../net.nnue")) };

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(align(64))]
#[repr(C)]
struct Chunk([i16; HIDDEN_SIZE]);

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(align(64))]
#[repr(C)]
pub struct Accumulator([Chunk; 2]);
impl Default for Accumulator {
    fn default() -> Self {
        Self([NET.feature_bias; 2])
    }
}

impl Accumulator {
    pub fn add_feature(&mut self, piece: PieceName, color: Color, sq: Square) {
        let white_idx = feature_idx(color, piece, sq);
        let black_idx = feature_idx(!color, piece, sq.flip_vertical());
        self.activate(&NET.feature_weights[white_idx], Color::White);
        self.activate(&NET.feature_weights[black_idx], Color::Black);
    }

    pub fn remove_feature(&mut self, piece: PieceName, color: Color, sq: Square) {
        let white_idx = feature_idx(color, piece, sq);
        let black_idx = feature_idx(!color, piece, sq.flip_vertical());
        self.deactivate(&NET.feature_weights[white_idx], Color::White);
        self.deactivate(&NET.feature_weights[black_idx], Color::Black);
    }

    fn deactivate(&mut self, weights: &Chunk, color: Color) {
        self.0[color.idx()].0.iter_mut().zip(&weights.0).for_each(|(i, &d)| {
            *i -= d;
        });
    }

    fn activate(&mut self, weights: &Chunk, color: Color) {
        self.0[color.idx()].0.iter_mut().zip(&weights.0).for_each(|(i, &d)| {
            *i += d;
        });
    }
}

#[repr(align(64))]
#[repr(C)]
#[derive(Clone, Debug)]
struct Network {
    feature_weights: [Chunk; INPUT_SIZE],
    feature_bias: Chunk,
    output_weights: [Chunk; 2],
    output_bias: i16,
}

impl Board {
    #[allow(clippy::deref_addrof)]
    pub fn evaluate(&self) -> i32 {
        let (us, them) = (self.accumulator.0[self.to_move.idx()], self.accumulator.0[(!self.to_move).idx()]);
        let weights = &NET.output_weights;

        // So this is odd... It crashes if I don't take the address and deref
        // I don't know enough rust to fix it
        // ¯\_(ツ)_/¯
        let mut output = i32::from(*&NET.output_bias);

        for (&i, &w) in us.0.iter().zip(&weights[0].0) {
            output += crelu(i) * i32::from(w);
        }

        for (&i, &w) in them.0.iter().zip(&weights[1].0) {
            output += crelu(i) * i32::from(w);
        }

        (output) * SCALE / Q
    }
}

const RELU_MIN: i16 = 0;
const RELU_MAX: i16 = 255;
fn crelu(i: i16) -> i32 {
    i32::from(i.clamp(RELU_MIN, RELU_MAX))
}

const COLOR_OFFSET: usize = NUM_SQUARES * NUM_PIECES;
const PIECE_OFFSET: usize = NUM_SQUARES;
fn feature_idx(color: Color, piece: PieceName, sq: Square) -> usize {
    color.idx() * COLOR_OFFSET + piece.idx() * PIECE_OFFSET + sq.idx()
}
