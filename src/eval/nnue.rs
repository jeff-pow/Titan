use strum::IntoEnumIterator;

use crate::{
    board::board::Board,
    types::{
        pieces::{Color, PieceName},
        square::Square,
    },
};

pub const INPUT_SIZE: usize = 768;
#[derive(Clone, Copy)]
pub struct NnueAccumulator {
    pub v: [[i16; INPUT_SIZE]; 2],
}

impl NnueAccumulator {
    fn get(&self, color: Color) -> [i16; INPUT_SIZE] {
        self.v[color as usize]
    }
}

struct LinearLayer {
    pub input_size: usize,
    pub output_size: usize,
    pub weights: Vec<Vec<i16>>,
    pub bias: Vec<i16>,
}

impl LinearLayer {
    fn new(input_size: usize, output_size: usize) -> LinearLayer {
        let bias = vec![0; output_size];
        let weights = vec![vec![0; input_size]; output_size];
        Self {
            input_size,
            output_size,
            weights,
            bias,
        }
    }
}

fn create_accumulator(perspective: Color, board: Board) -> [i32; INPUT_SIZE] {
    let mut idx = 0;
    let mut arr = [0; INPUT_SIZE];

    for piece in PieceName::iter() {
        let bb = board.bitboard(perspective, piece);
        for sq in bb {
            arr[calc_accumulator_idx(perspective, piece, sq)] = 1;
        }
    }
    for piece in PieceName::iter() {
        let bb = board.bitboard(perspective.opp(), piece);
        for sq in bb {
            arr[calc_accumulator_idx(perspective.opp(), piece, sq)] = 1;
        }
    }
    arr
}

fn calc_accumulator_idx(color: Color, piece: PieceName, sq: Square) -> usize {
    let c = color as usize * INPUT_SIZE / 2;
    let p = piece as usize * INPUT_SIZE / 2 / 6;
    let s = sq.idx();
    c + p + s
}

fn refresh_accumulator(perspective: Color, layer: &LinearLayer, active_features: &[i32]) -> NnueAccumulator {
    let mut new_acc = NnueAccumulator {
        v: [[0; INPUT_SIZE]; 2],
    };
    for i in 0..INPUT_SIZE {
        new_acc.v[perspective as usize][i] = layer.bias[i];
    }

    for a in active_features {
        for i in 0..INPUT_SIZE {
            new_acc.v[perspective as usize][i] += layer.weights[*a as usize][i];
        }
    }
    new_acc
}

fn update_accumulator(
    layer: &LinearLayer,
    prev_acc: &NnueAccumulator,
    removed_features: &[i32],
    added_features: &[i32],
    perspective: Color,
) -> NnueAccumulator {
    let mut new_acc = NnueAccumulator {
        v: [[0; INPUT_SIZE]; 2],
    };
    let perspective = perspective as usize;

    for i in 0..INPUT_SIZE {
        new_acc.v[perspective][i] = prev_acc.v[perspective][i];
    }

    for r in removed_features {
        for i in 0..INPUT_SIZE {
            new_acc.v[perspective][i] -= layer.weights[*r as usize][i];
        }
    }

    for a in added_features {
        for i in 0..INPUT_SIZE {
            new_acc.v[perspective][i] += layer.weights[*a as usize][i];
        }
    }

    new_acc
}

fn linear(layer: &LinearLayer, input: &[f32]) -> Vec<f32> {
    let mut output = Vec::new();
    for i in 0..layer.output_size {
        output[i] = layer.bias[i] as f32;
    }

    for i in 0..layer.input_size {
        for j in 0..layer.output_size {
            output[j] += input[i] * layer.weights[i][j] as f32;
        }
    }
    output
}

fn crelu(size: usize, input: &[f32]) -> Vec<f32> {
    let mut output = Vec::new();
    for i in 0..size {
        output.push(1f32.min(0f32.max(input[i])));
    }
    output
}

fn nnue_evaluate(board: &Board) -> i32 {
    let mut input = [0.; INPUT_SIZE];
    let stm = board.to_move;
    // for i in 0..M {
    //     input[i] = pos.accumulator[stm as usize][i];
    //     input[M + i] = pos.accumulator[stm.opp() as usize][i];
    // }
    17
}
