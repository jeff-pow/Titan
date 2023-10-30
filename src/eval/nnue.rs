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
pub const NET: Network = unsafe { std::mem::transmute(*include_bytes!("../../net.nnue")) };

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub struct Accumulator([[i16; HIDDEN_SIZE]; 2]);
impl Default for Accumulator {
    fn default() -> Self {
        Self([NET.feature_bias; 2])
    }
}

impl Accumulator {
    pub fn get(&self, color: Color) -> &[i16; HIDDEN_SIZE] {
        &self.0[color.idx()]
    }

    pub fn add_feature(&mut self, piece: PieceName, color: Color, sq: Square) {
        let white_idx = feature_idx(color, piece, sq);
        let black_idx = feature_idx(!color, piece, sq.flip_vertical());

        self.activate(&NET.feature_weights, white_idx * HIDDEN_SIZE, Color::White);
        self.activate(&NET.feature_weights, black_idx * HIDDEN_SIZE, Color::Black);
    }

    pub fn remove_feature(&mut self, net: &Network, piece: PieceName, color: Color, sq: Square) {
        let white_idx = feature_idx(color, piece, sq);
        let black_idx = feature_idx(!color, piece, sq.flip_vertical());
        self.deactivate(&net.feature_weights, white_idx * HIDDEN_SIZE, Color::White);
        self.deactivate(&net.feature_weights, black_idx * HIDDEN_SIZE, Color::Black);
    }

    pub(crate) fn reset(&mut self) {
        self.0 = [NET.feature_bias; 2];
    }

    fn deactivate(&mut self, weights: &[i16; HIDDEN_SIZE * INPUT_SIZE], offset: usize, color: Color) {
        self.0[color.idx()]
            .iter_mut()
            .zip(&weights[offset..offset + HIDDEN_SIZE])
            .for_each(|(i, &d)| {
                *i -= d;
            })
    }

    fn activate(&mut self, weights: &[i16; HIDDEN_SIZE * INPUT_SIZE], offset: usize, color: Color) {
        self.0[color.idx()]
            .iter_mut()
            .zip(&weights[offset..offset + HIDDEN_SIZE])
            .for_each(|(i, &d)| {
                *i += d;
            })
    }
}

#[repr(align(64))]
#[repr(C)]
#[derive(Clone, Debug)]
pub struct Network {
    pub feature_weights: [i16; INPUT_SIZE * HIDDEN_SIZE],
    pub feature_bias: [i16; HIDDEN_SIZE],
    pub output_weights: [i16; HIDDEN_SIZE * 2],
    pub output_bias: i16,
}

impl Board {
    #[allow(clippy::deref_addrof)]
    pub fn evaluate(&self) -> i32 {
        let (us, them) = (self.accumulator.get(self.to_move), self.accumulator.get(!self.to_move));

        let weights = &NET.output_weights;
        let mut output = i32::from(*&NET.output_bias);

        for (&i, &w) in us.iter().zip(&weights[..HIDDEN_SIZE]) {
            output += crelu(i) * i32::from(w);
        }

        for (&i, &w) in them.iter().zip(&weights[HIDDEN_SIZE..]) {
            output += crelu(i) * i32::from(w);
        }
        // So this is odd... It crashes if I don't take the address and deref
        // I don't know enough rust to fix it
        // ¯\_(ツ)_/¯
        // output += *&NET.output_bias as i32;

        (output) * SCALE / Q
    }
}

const RELU_MIN: i16 = 0;
const RELU_MAX: i16 = 255;
#[inline(always)]
fn crelu(i: i16) -> i32 {
    i32::from(i.clamp(RELU_MIN, RELU_MAX))
}

const COLOR_OFFSET: usize = NUM_SQUARES * NUM_PIECES;
const PIECE_OFFSET: usize = NUM_SQUARES;
fn feature_idx(color: Color, piece: PieceName, sq: Square) -> usize {
    color.idx() * COLOR_OFFSET + piece.idx() * PIECE_OFFSET + sq.idx()
}
