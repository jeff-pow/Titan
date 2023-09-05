use strum::IntoEnumIterator;

use crate::{
    board::board::Board,
    types::{
        bitboard::Bitboard,
        pieces::{Color, PieceName},
        square::Square,
    },
};

pub const M: usize = 768;
#[derive(Clone, Copy)]
pub struct NnueAccumulator {
    pub v: [[i16; M]; 2],
}

impl NnueAccumulator {
    fn get(&self, color: Color) -> [i16; M] {
        self.v[color as usize]
    }
}

struct LinearLayer {
    pub input_size: usize,
    pub output_size: usize,
    pub weight: Vec<Vec<i16>>,
    pub bias: Vec<i16>,
}

fn create_accumulator(perspective: Color, board: Board) -> [i32; M] {
    let mut idx = 0;
    let mut arr = [0; M];

    for piece in PieceName::iter() {
        let bb = board.bitboards[perspective as usize][piece as usize];
        for sq in bb {
            arr[calc_accumulator_idx(perspective, piece, sq)] = 1;
        }
    }
    for piece in PieceName::iter() {
        let bb = board.bitboards[perspective.opp() as usize][piece as usize];
        for sq in bb {
            arr[calc_accumulator_idx(perspective.opp(), piece, sq)] = 1;
        }
    }
    arr
}

fn calc_accumulator_idx(color: Color, piece: PieceName, sq: Square) -> usize {
    let c = color as usize * M / 2;
    let p = piece as usize * M / 2 / 6;
    let s = sq.idx();
    c + p + s
}

fn refresh_accumulator(perspective: Color, layer: &LinearLayer, active_features: &[i32]) -> NnueAccumulator {
    let mut new_acc = NnueAccumulator { v: [[0; M]; 2] };
    for i in 0..M {
        new_acc.v[perspective as usize][i] = layer.bias[i];
    }

    for a in active_features {
        for i in 0..M {
            new_acc.v[perspective as usize][i] += layer.weight[*a as usize][i];
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
    let mut new_acc = NnueAccumulator { v: [[0; M]; 2] };
    let perspective = perspective as usize;

    for i in 0..M {
        new_acc.v[perspective][i] = prev_acc.v[perspective][i];
    }

    for r in removed_features {
        for i in 0..M {
            new_acc.v[perspective][i] -= layer.weight[*r as usize][i];
        }
    }

    for a in added_features {
        for i in 0..M {
            new_acc.v[perspective][i] += layer.weight[*a as usize][i];
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
            output[j] += input[i] * layer.weight[i][j] as f32;
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
    let mut input = [0.; M];
    let stm = board.to_move;
    // for i in 0..M {
    //     input[i] = pos.accumulator[stm as usize][i];
    //     input[M + i] = pos.accumulator[stm.opp() as usize][i];
    // }
    17
}
