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

    fn new(&mut self, bias: &[i16; LAYER_1_SIZE], color_to_update: Color) {
        self.v[color_to_update.idx()] = *bias;
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
    net: Network,
}

impl NetworkState {
    pub fn new(board: &Board) -> Box<Self> {
        let mut net: Box<Self> = unsafe {
            let layout = std::alloc::Layout::new::<Self>();
            let ptr = std::alloc::alloc_zeroed(layout);
            if ptr.is_null() {
                std::alloc::handle_alloc_error(layout);
            }
            Box::from_raw(ptr.cast())
        };

        net
    }

    fn refresh_accumulators(&mut self, board: &Board) {
        let white_king = board.king_square(Color::White);
        let black_king = board.king_square(Color::Black);

        self.accumulator.new(&self.net.feature_bias, Color::White);
        self.accumulator.new(&self.net.feature_bias, Color::Black);

        for c in Color::iter() {
            for p in PieceName::iter() {
                let bb = board.bitboard(c, p);
                for sq in bb {
                    // self.update features
                }
            }
        }
    }
}

const COLOR_OFFSET: usize = 64 * 6;
const PIECE_OFFSET: usize = 64;
fn feature_idx(color: Color, piece: PieceName, sq: Square) -> usize {
    color.idx() * COLOR_OFFSET + piece.idx() * PIECE_OFFSET + sq.idx()
}
