use std::sync::Arc;

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
const DIVISOR: i32 = 255 * 64;
const CRELU_MIN: i16 = 0;
const CRELU_MAX: i16 = i8::MAX as i16;
const LAYER_1_SIZE: usize = 1536;
#[derive(Clone, Copy)]
pub struct Accumulator {
    pub v: [[i16; LAYER_1_SIZE]; 2],
}

impl Accumulator {
    fn get(&self, color: Color) -> [i16; LAYER_1_SIZE] {
        self.v[color.idx()]
    }

    fn init(&mut self, bias: &[i16; LAYER_1_SIZE], update: Update) {
        if update.white {
            self.v[Color::White.idx()] = *bias;
        }
        if update.black {
            self.v[Color::Black.idx()] = *bias;
        }
    }
}

#[repr(C)]
#[derive(Clone)]
pub struct Network {
    pub feature_weights: [i16; INPUT_SIZE * LAYER_1_SIZE],
    pub feature_bias: [i16; LAYER_1_SIZE],
    pub output_weights: [i8; LAYER_1_SIZE * 2],
    pub ouput_bias: i16,
}

#[repr(C)]
pub struct NetworkState {
    inputs: [[i16; INPUT_SIZE]; 2],
    accumulator: Accumulator,
    net: Network,
}

impl NetworkState {
    pub fn new(board: &Board) -> Box<Self> {
        // let mut net: Box<Self> = unsafe {
        //     let layout = std::alloc::Layout::new::<Self>();
        //     let ptr = std::alloc::alloc_zeroed(layout);
        //     if ptr.is_null() {
        //         std::alloc::handle_alloc_error(layout);
        //     }
        //     Box::from_raw(ptr.cast())
        // };
        let net = Self {
            inputs: [[0; INPUT_SIZE]; 2],
            accumulator: Accumulator {
                v: [[0; LAYER_1_SIZE]; 2],
            },
            net: Network {
                feature_weights: [0; 768  * (768 * 2)],
                feature_bias: [0; 768 * 2],
                output_weights: [0; (768 * 2) * 2],
                ouput_bias: 0,
            }
        };
        let mut net = Box::from(net);

        net.refresh_accumulators(board, Update::BOTH);

        net
    }



    pub fn refresh_accumulators(&mut self, board: &Board, update: Update) {
        if update.white {
            self.inputs[Color::White.idx()].fill(0);
        }
        if update.black {
            self.inputs[Color::Black.idx()].fill(0);
        }

        self.accumulator.init(&self.net.feature_bias, update);

        for c in Color::iter() {
            for p in PieceName::iter() {
                let bb = board.bitboard(c, p);
                for sq in bb {
                    self.update_feature(p, c, sq, Activation::Activate, update)
                }
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn update_feature(&mut self, piece: PieceName, color: Color, sq: Square, act: Activation, update: Update) {
        let white_idx = feature_idx(color, piece, sq);
        let black_idx = feature_idx(!color, piece, sq.flip_vertical());
        let acc = &mut self.accumulator;

        match act {
            Activation::Activate => {
                if update.white {
                    activate(&mut acc.get(Color::White), &self.net.feature_weights, white_idx * LAYER_1_SIZE);
                    self.inputs[Color::White.idx()][white_idx] = 1;
                }
                if update.black {
                    activate(&mut acc.get(Color::Black), &self.net.feature_weights, black_idx * LAYER_1_SIZE);
                    self.inputs[Color::Black.idx()][black_idx] = 1;
                }
            }
            Activation::Deactivate => {
                if update.white {
                    deactivate(&mut acc.get(Color::White), &self.net.feature_weights, white_idx * LAYER_1_SIZE);
                    self.inputs[Color::White.idx()][white_idx] = 0;
                }
                if update.black {
                    deactivate(&mut acc.get(Color::Black), &self.net.feature_weights, black_idx * LAYER_1_SIZE);
                    self.inputs[Color::Black.idx()][black_idx] = 0;
                }
            }
        }

        self.assert_valid(white_idx, black_idx, (color, piece, sq), update, act);
    }

    pub fn update_pov_move(&mut self, piece: PieceName, color: Color, from: Square, to: Square) {
        let white_to = feature_idx(color, piece, to);
        let black_to = feature_idx(!color, piece, to.flip_vertical());
        let white_from = feature_idx(color, piece, from);
        let black_from = feature_idx(!color, piece, from.flip_vertical());

        self.inputs[Color::White.idx()][white_from] = 0;
        self.inputs[Color::White.idx()][white_to] = 1;
        self.inputs[Color::Black.idx()][black_from] = 0;
        self.inputs[Color::Black.idx()][black_to] = 1;
    }

    #[allow(clippy::too_many_arguments)]
    pub fn update_pov_manual(&mut self, piece: PieceName, color: Color, sq: Square, act: Activation) {
        let white_idx = feature_idx(color, piece, sq);
        let black_idx = feature_idx(!color, piece, sq.flip_vertical());

        match act {
            Activation::Activate => {
                self.inputs[Color::White.idx()][white_idx] = 1;
                self.inputs[Color::Black.idx()][black_idx] = 1;
            }
            Activation::Deactivate => {
                self.inputs[Color::White.idx()][white_idx] = 0;
                self.inputs[Color::Black.idx()][black_idx] = 0;
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn move_feature(
        &mut self,
        piece: PieceName,
        color: Color,
        from: Square,
        to: Square,
        act: Activation,
        update: Update,
    ) {
        let white_to = feature_idx(color, piece, to);
        let black_to = feature_idx(!color, piece, to.flip_vertical());
        let white_from = feature_idx(color, piece, from);
        let black_from = feature_idx(!color, piece, from.flip_vertical());

        let acc = &mut self.accumulator;

        if update.white {
            activate(&mut acc.get(Color::White), &self.net.feature_weights, white_to * LAYER_1_SIZE);
            deactivate(&mut acc.get(Color::White), &self.net.feature_weights, white_from * LAYER_1_SIZE);
        }
        if update.black {
            activate(&mut acc.get(Color::Black), &self.net.feature_weights, black_to * LAYER_1_SIZE);
            deactivate(&mut acc.get(Color::Black), &self.net.feature_weights, black_from * LAYER_1_SIZE);
        }

        if update.white {
            self.inputs[Color::White.idx()][white_from] = 0;
        }
        if update.black {
            self.inputs[Color::Black.idx()][black_from] = 0;
        }
        if update.white {
            self.inputs[Color::White.idx()][white_to] = 1;
        }
        if update.black {
            self.inputs[Color::Black.idx()][black_to] = 1;
        }
        self.assert_valid(white_from, black_from, (color, piece, from), update, act);
        self.assert_valid(white_to, black_to, (color, piece, to), update, act);
    }

    pub fn evaluate(&self, to_move: Color) -> i32 {
        let acc = &self.accumulator;

        let (us, them) = if to_move == Color::White {
            (acc.get(Color::White), acc.get(Color::Black))
        } else {
            (acc.get(Color::Black), acc.get(Color::White))
        };

        let output = flatten(&us, &them, &self.net.output_weights);

        (output + i32::from(self.net.ouput_bias)) * SCALE / DIVISOR
    }

    fn assert_valid(
        &self,
        white: usize,
        black: usize,
        feature: (Color, PieceName, Square),
        update: Update,
        act: Activation,
    ) {
        let (color, piece, sq) = feature;
        let val = if act == Activation::Activate { 1 } else { 0 };
        if update.white {
            assert_eq!(
                self.inputs[Color::White.idx()][white],
                val,
                "piece: {:?}, color: {:?} sq: {:?}",
                piece,
                color,
                sq
            );
        }
        if update.black {
            assert_eq!(
                self.inputs[Color::Black.idx()][black],
                val,
                "piece: {:?}, color: {:?} sq: {:?}",
                piece,
                color,
                sq
            );
        }
    }
}

fn flatten(us: &[i16; LAYER_1_SIZE], them: &[i16; LAYER_1_SIZE], weights: &[i8; LAYER_1_SIZE * 2]) -> i32 {
    let mut sum = 0;
    us.iter()
        .zip(&weights[..LAYER_1_SIZE])
        .for_each(|(i, w)| sum += crelu(*i) * i32::from(*w));
    them.iter()
        .zip(&weights[LAYER_1_SIZE..])
        .for_each(|(i, w)| sum += crelu(*i) * i32::from(*w));
    sum
}

#[inline(always)]
fn crelu(i: i16) -> i32 {
    i.clamp(CRELU_MIN, CRELU_MAX).into()
}

fn deactivate<const SIZE: usize, const WEIGHTS: usize>(
    input: &mut [i16; SIZE],
    weights: &[i16; WEIGHTS],
    offset: usize,
) {
    for (i, d) in input.iter_mut().zip(&weights[offset..offset + SIZE]) {
        *i -= *d;
    }
}

fn activate<const SIZE: usize, const WEIGHTS: usize>(input: &mut [i16; SIZE], weights: &[i16; WEIGHTS], offset: usize) {
    for (i, d) in input.iter_mut().zip(&weights[offset..offset + SIZE]) {
        *i += *d;
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum Activation {
    Activate,
    Deactivate,
}

#[derive(Copy, Clone, PartialEq)]
pub struct Update {
    pub white: bool,
    pub black: bool,
}

impl Update {
    const BOTH: Self = Self {
        white: true,
        black: true,
    };
    pub fn flip(self) -> Self {
        Self {
            white: self.black,
            black: self.white,
        }
    }

    pub fn color(c: Color) -> Self {
        match c {
            Color::White => Self {
                white: true,
                black: false,
            },
            Color::Black => Self {
                white: false,
                black: true,
            },
        }
    }
}

const COLOR_OFFSET: usize = 64 * 6;
const PIECE_OFFSET: usize = 64;
fn feature_idx(color: Color, piece: PieceName, sq: Square) -> usize {
    color.idx() * COLOR_OFFSET + piece.idx() * PIECE_OFFSET + sq.idx()
}
