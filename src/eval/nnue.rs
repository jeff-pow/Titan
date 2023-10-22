use std::process::exit;
use crate::types::{
    pieces::{Color, PieceName},
    square::Square,
};

pub const INPUT_SIZE: usize = 768;
const HIDDEN_SIZE: usize = 1024;
pub const NET: Network = unsafe { std::mem::transmute(*include_bytes!("../../net.nnue")) };
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub struct Accumulator([[i16; HIDDEN_SIZE]; 2]);
impl Default for Accumulator {
    fn default() -> Self {
        let mut a = Self([[0; HIDDEN_SIZE]; 2]);
        a.reset();
        a
    }
}

impl Accumulator {
    pub fn get(&self, color: Color) -> [i16; HIDDEN_SIZE] {
        self.0[color.idx()]
    }

    pub fn add_feature(&mut self, piece: PieceName, color: Color, sq: Square) {
        let white_idx = feature_idx(color, piece, sq);
        let black_idx = feature_idx(!color, piece, sq.flip_vertical());
        // activate(&mut self.get(Color::White), &NET.feature_weights, white_idx * HIDDEN_SIZE);
        // activate(&mut self.get(Color::Black), &NET.feature_weights, black_idx * HIDDEN_SIZE);
        let a = self.0[Color::White.idx()].clone();
        dbg!(&NET.feature_weights[white_idx..white_idx + HIDDEN_SIZE]);
        exit(0);
        for (i, d) in self.0[Color::White.idx()].iter_mut().zip(&NET.feature_weights[white_idx..white_idx + HIDDEN_SIZE]) {
            *i += *d;
        }
        assert_ne!(a, self.0[Color::White.idx()]);
        for (i, d) in self.0[Color::Black.idx()].iter_mut().zip(&NET.feature_weights[black_idx..black_idx + HIDDEN_SIZE]) {
            *i += *d;
        }
        dbg!(piece);
        dbg!(color);
        dbg!(sq);
        dbg!(self.get(Color::White));
        dbg!(self.get(Color::Black));
        assert_ne!(self.get(Color::White), self.get(Color::Black));
    }

    pub fn remove_feature(&mut self, net: &Network, piece: PieceName, color: Color, sq: Square) {
        let white_idx = feature_idx(color, piece, sq);
        let black_idx = feature_idx(!color, piece, sq.flip_vertical());
        deactivate(&mut self.get(Color::White), &net.feature_weights, white_idx * HIDDEN_SIZE);
        deactivate(&mut self.get(Color::Black), &net.feature_weights, black_idx * HIDDEN_SIZE);
    }

    pub(crate) fn reset(&mut self) {
        self.0[Color::White.idx()] = NET.feature_bias.clone();
        self.0[Color::Black.idx()] = NET.feature_bias.clone();
    }
}

// #[repr(align(64))]
#[repr(C)]
#[derive(Clone, Debug)]
pub struct Network {
    pub feature_weights: [i16; INPUT_SIZE * HIDDEN_SIZE],
    pub feature_bias: [i16; HIDDEN_SIZE],
    pub output_weights: [i16; HIDDEN_SIZE * 2],
    pub output_bias: i16,
}

impl Network {
    pub fn evaluate(&self, acc: &Accumulator, to_move: Color) -> i32 {
        let (us, them) = (acc.get(to_move), acc.get(!to_move));

        let weights = &self.output_weights;
        let mut output = 0;

        for (&i, &w) in us.iter().zip(&weights[..HIDDEN_SIZE]) {
            output += screlu(i) * i32::from(w);
        }
        for (&i, &w) in them.iter().zip(&weights[HIDDEN_SIZE..]) {
            output += screlu(i) * i32::from(w);
        }

        (output / 255 + i32::from(self.output_bias)) * 400 / (64 * 255)
    }
}

const RELU_MIN: i16 = 0;
const RELU_MAX: i16 = 255;
#[inline(always)]
fn screlu(i: i16) -> i32 {
    let i = i32::from(i.clamp(RELU_MIN, RELU_MAX));
    i * i
}

#[allow(dead_code)]
fn crelu(i: i16) -> i32 {
    i32::from(i.clamp(RELU_MIN, RELU_MAX))
}

fn deactivate(input: &mut [i16], weights: &[i16], offset: usize) {
    for (i, &d) in input.iter_mut().zip(&weights[offset..offset + HIDDEN_SIZE]) {
        *i -= d;
    }
}

fn activate(input: &mut [i16], weights: &[i16], offset: usize) {
    for (i, &d) in input.iter_mut().zip(&weights[offset..offset + HIDDEN_SIZE]) {
        *i += d;
    }
}

const COLOR_OFFSET: usize = 64 * 6;
const PIECE_OFFSET: usize = 64;
fn feature_idx(color: Color, piece: PieceName, sq: Square) -> usize {
    color.idx() * COLOR_OFFSET + piece.idx() * PIECE_OFFSET + sq.idx()
}
