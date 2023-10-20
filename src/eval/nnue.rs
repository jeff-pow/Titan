use crate::types::{
    pieces::{Color, PieceName},
    square::Square,
};

pub const INPUT_SIZE: usize = 768;
const HIDDEN_SIZE: usize = 1536;
pub static NETWORK: Network = Network::new();
#[derive(Clone, Copy)]
pub struct Accumulator([[i16; HIDDEN_SIZE]; 2]);
impl Default for Accumulator {
    fn default() -> Self {
        Self([[0; HIDDEN_SIZE]; 2])
    }
}

impl Accumulator {
    pub fn get(&self, color: Color) -> [i16; HIDDEN_SIZE] {
        self.0[color.idx()]
    }

    pub fn add_feature(&mut self, piece: PieceName, color: Color, sq: Square) {
        let white_idx = feature_idx(color, piece, sq);
        let black_idx = feature_idx(!color, piece, sq.flip_vertical());

        activate(&mut self.get(Color::White), &NETWORK.feature_weights, white_idx * HIDDEN_SIZE);
        activate(&mut self.get(Color::Black), &NETWORK.feature_weights, black_idx * HIDDEN_SIZE);
    }

    pub fn remove_feature(&mut self, net: &Network, piece: PieceName, color: Color, sq: Square) {
        let white_idx = feature_idx(color, piece, sq);
        let black_idx = feature_idx(!color, piece, sq.flip_vertical());

        deactivate(&mut self.get(Color::White), &net.feature_weights, white_idx * HIDDEN_SIZE);
        deactivate(&mut self.get(Color::Black), &net.feature_weights, black_idx * HIDDEN_SIZE);
    }

    pub fn move_feature(&self, net: &Network, piece: PieceName, color: Color, from: Square, to: Square) {
        let white_to = feature_idx(color, piece, to);
        let black_to = feature_idx(!color, piece, to.flip_vertical());
        let white_from = feature_idx(color, piece, from);
        let black_from = feature_idx(!color, piece, from.flip_vertical());

        activate(&mut self.get(Color::White), &net.feature_weights, white_to * HIDDEN_SIZE);
        deactivate(&mut self.get(Color::White), &net.feature_weights, white_from * HIDDEN_SIZE);

        activate(&mut self.get(Color::Black), &net.feature_weights, black_to * HIDDEN_SIZE);
        deactivate(&mut self.get(Color::Black), &net.feature_weights, black_from * HIDDEN_SIZE);
    }

    pub(crate) fn reset(&mut self) {
        for (idx, x) in self.0[Color::White.idx()].iter_mut().enumerate() {
            *x = NETWORK.feature_bias[idx];
        }
        for (idx, x) in self.0[Color::Black.idx()].iter_mut().enumerate() {
            *x = NETWORK.feature_bias[idx];
        }
    }
}

#[repr(C)]
#[derive(Clone)]
pub struct Network {
    pub feature_weights: [i16; INPUT_SIZE * HIDDEN_SIZE],
    pub feature_bias: [i16; HIDDEN_SIZE],
    pub output_weights: [i16; HIDDEN_SIZE * 2],
    pub ouput_bias: i16,
}

impl Network {
    pub const fn new() -> Self {
        Self {
            feature_weights: [0; INPUT_SIZE * HIDDEN_SIZE],
            feature_bias: [0; HIDDEN_SIZE],
            output_weights: [0; HIDDEN_SIZE * 2],
            ouput_bias: 0,
        }
    }

    pub fn evaluate(&self, acc: &Accumulator, to_move: Color) -> i32 {
        let (us, them) = (acc.get(to_move), acc.get(!to_move));

        let weights = &self.output_weights;
        let mut output = 0;
        us.iter()
            .zip(&weights[..HIDDEN_SIZE])
            .for_each(|(i, w)| output += crelu(*i) * i32::from(*w));
        them.iter()
            .zip(&weights[HIDDEN_SIZE..])
            .for_each(|(i, w)| output += crelu(*i) * i32::from(*w));

        (output / 255 + i32::from(self.ouput_bias)) * 400 / (64 * 255)
        // (output + i32::from(self.net.ouput_bias)) * SCALE / DIVISOR
    }
}

#[inline(always)]
fn crelu(i: i16) -> i32 {
    const CRELU_MIN: i16 = 0;
    const CRELU_MAX: i16 = i8::MAX as i16;
    i.clamp(CRELU_MIN, CRELU_MAX).into()
}

fn deactivate(input: &mut [i16], weights: &[i16], offset: usize) {
    for (i, d) in input.iter_mut().zip(&weights[offset..offset + HIDDEN_SIZE]) {
        *i -= *d;
    }
}

fn activate(input: &mut [i16], weights: &[i16], offset: usize) {
    for (i, d) in input.iter_mut().zip(&weights[offset..offset + HIDDEN_SIZE]) {
        *i += *d;
    }
}

const COLOR_OFFSET: usize = 64 * 6;
const PIECE_OFFSET: usize = 64;
fn feature_idx(color: Color, piece: PieceName, sq: Square) -> usize {
    color.idx() * COLOR_OFFSET + piece.idx() * PIECE_OFFSET + sq.idx()
}
