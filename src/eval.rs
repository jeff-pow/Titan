use std::cmp::min;

use crate::{
    board::Board,
    pieces::{piece_value, Color, PieceName},
    square::Square,
};

/// https://www.chessprogramming.org/PeSTO%27s_Evaluation_Function

#[rustfmt::skip]
const MG_PAWN_TABLE: [i32; 64] = [
     0,   0,   0,   0,   0,   0,  0,   0,
    98, 134,  61,  95,  68, 126, 34, -11,
    -6,   7,  26,  31,  65,  56, 25, -20,
   -14,  13,   6,  21,  23,  12, 17, -23,
   -27,  -2,  -5,  12,  17,   6, 10, -25,
   -26,  -4,  -4, -10,   3,   3, 33, -12,
   -35,  -1, -20, -23, -15,  24, 38, -22,
     0,   0,   0,   0,   0,   0,  0,   0
];

#[rustfmt::skip]
const EG_PAWN_TABLE: [i32; 64] = [
     0,   0,   0,   0,   0,   0,   0,   0,
   178, 173, 158, 134, 147, 132, 165, 187,
    94, 100,  85,  67,  56,  53,  82,  84,
    32,  24,  13,   5,  -2,   4,  17,  17,
    13,   9,  -3,  -7,  -7,  -8,   3,  -1,
     4,   7,  -6,   1,   0,  -5,  -1,  -8,
    13,   8,   8,  10,  13,   0,   2,  -7,
     0,   0,   0,   0,   0,   0,   0,   0,
];

#[rustfmt::skip]
const MG_KNIGHT_TABLE: [i32; 64] = [
   -167, -89, -34, -49,  61, -97, -15, -107,
    -73, -41,  72,  36,  23,  62,   7,  -17,
    -47,  60,  37,  65,  84, 129,  73,   44,
     -9,  17,  19,  53,  37,  69,  18,   22,
    -13,   4,  16,  13,  28,  19,  21,   -8,
    -23,  -9,  12,  10,  19,  17,  25,  -16,
    -29, -53, -12,  -3,  -1,  18, -14,  -19,
   -105, -21, -58, -33, -17, -28, -19,  -23,
];

#[rustfmt::skip]
const EG_KNIGHT_TABLE: [i32; 64] = [
   -58, -38, -13, -28, -31, -27, -63, -99,
   -25,  -8, -25,  -2,  -9, -25, -24, -52,
   -24, -20,  10,   9,  -1,  -9, -19, -41,
   -17,   3,  22,  22,  22,  11,   8, -18,
   -18,  -6,  16,  25,  16,  17,   4, -18,
   -23,  -3,  -1,  15,  10,  -3, -20, -22,
   -42, -20, -10,  -5,  -2, -20, -23, -44,
   -29, -51, -23, -15, -22, -18, -50, -64,
];

#[rustfmt::skip]
const MG_BISHOP_TABLE: [i32; 64] = [
   -29,   4, -82, -37, -25, -42,   7,  -8,
   -26,  16, -18, -13,  30,  59,  18, -47,
   -16,  37,  43,  40,  35,  50,  37,  -2,
    -4,   5,  19,  50,  37,  37,   7,  -2,
    -6,  13,  13,  26,  34,  12,  10,   4,
     0,  15,  15,  15,  14,  27,  18,  10,
     4,  15,  16,   0,   7,  21,  33,   1,
   -33,  -3, -14, -21, -13, -12, -39, -21,
];

#[rustfmt::skip]
const EG_BISHOP_TABLE: [i32; 64] = [
   -14, -21, -11,  -8, -7,  -9, -17, -24,
    -8,  -4,   7, -12, -3, -13,  -4, -14,
     2,  -8,   0,  -1, -2,   6,   0,   4,
    -3,   9,  12,   9, 14,  10,   3,   2,
    -6,   3,  13,  19,  7,  10,  -3,  -9,
   -12,  -3,   8,  10, 13,   3,  -7, -15,
   -14, -18,  -7,  -1,  4,  -9, -15, -27,
   -23,  -9, -23,  -5, -9, -16,  -5, -17,
];

#[rustfmt::skip]
const MG_ROOK_TABLE: [i32; 64] = [
    32,  42,  32,  51, 63,  9,  31,  43,
    27,  32,  58,  62, 80, 67,  26,  44,
    -5,  19,  26,  36, 17, 45,  61,  16,
   -24, -11,   7,  26, 24, 35,  -8, -20,
   -36, -26, -12,  -1,  9, -7,   6, -23,
   -45, -25, -16, -17,  3,  0,  -5, -33,
   -44, -16, -20,  -9, -1, 11,  -6, -71,
   -19, -13,   1,  17, 16,  7, -37, -26,
];

#[rustfmt::skip]
const EG_ROOK_TABLE: [i32; 64] = [
   13, 10, 18, 15, 12,  12,   8,   5,
   11, 13, 13, 11, -3,   3,   8,   3,
    7,  7,  7,  5,  4,  -3,  -5,  -3,
    4,  3, 13,  1,  2,   1,  -1,   2,
    3,  5,  8,  4, -5,  -6,  -8, -11,
   -4,  0, -5, -1, -7, -12,  -8, -16,
   -6, -6,  0,  2, -9,  -9, -11,  -3,
   -9,  2,  3, -1, -5, -13,   4, -20,
];

#[rustfmt::skip]
const MG_QUEEN_TABLE: [i32; 64] = [
   -28,   0,  29,  12,  59,  44,  43,  45,
   -24, -39,  -5,   1, -16,  57,  28,  54,
   -13, -17,   7,   8,  29,  56,  47,  57,
   -27, -27, -16, -16,  -1,  17,  -2,   1,
    -9, -26,  -9, -10,  -2,  -4,   3,  -3,
   -14,   2, -11,  -2,  -5,   2,  14,   5,
   -35,  -8,  11,   2,   8,  15,  -3,   1,
    -1, -18,  -9,  10, -15, -25, -31, -50,
];

#[rustfmt::skip]
const EG_QUEEN_TABLE: [i32; 64] = [
    -9,  22,  22,  27,  27,  19,  10,  20,
   -17,  20,  32,  41,  58,  25,  30,   0,
   -20,   6,   9,  49,  47,  35,  19,   9,
     3,  22,  24,  45,  57,  40,  57,  36,
   -18,  28,  19,  47,  31,  34,  39,  23,
   -16, -27,  15,   6,   9,  17,  10,   5,
   -22, -23, -30, -16, -16, -23, -36, -32,
   -33, -28, -22, -43,  -5, -32, -20, -41,
];

#[rustfmt::skip]
const MG_KING_TABLE: [i32; 64] = [
   -65,  23,  16, -15, -56, -34,   2,  13,
    29,  -1, -20,  -7,  -8,  -4, -38, -29,
    -9,  24,   2, -16, -20,   6,  22, -22,
   -17, -20, -12, -27, -30, -25, -14, -36,
   -49,  -1, -27, -39, -46, -44, -33, -51,
   -14, -14, -22, -46, -44, -30, -15, -27,
     1,   7,  -8, -64, -43, -16,   9,   8,
   -15,  36,  12, -54,   8, -28,  24,  14
];

#[rustfmt::skip]
const EG_KING_TABLE: [i32; 64] = [
   -74, -35, -18, -18, -11,  15,   4, -17,
   -12,  17,  14,  17,  17,  38,  23,  11,
    10,  17,  23,  15,  20,  45,  44,  13,
    -8,  22,  24,  27,  26,  33,  26,   3,
   -18,  -4,  21,  24,  27,  23,   9, -11,
   -19,  -3,  11,  21,  23,  16,   7,  -9,
   -27, -11,   4,  13,  14,   4,  -5, -17,
   -53, -34, -21, -11, -28, -14, -24, -43
];

fn get_mg_table(piece: PieceName) -> &'static [i32; 64] {
    match piece {
        PieceName::King => &MG_KING_TABLE,
        PieceName::Queen => &MG_QUEEN_TABLE,
        PieceName::Rook => &MG_ROOK_TABLE,
        PieceName::Bishop => &MG_BISHOP_TABLE,
        PieceName::Knight => &MG_KNIGHT_TABLE,
        PieceName::Pawn => &MG_PAWN_TABLE,
    }
}

fn get_eg_table(piece: PieceName) -> &'static [i32; 64] {
    match piece {
        PieceName::King => &EG_KING_TABLE,
        PieceName::Queen => &EG_QUEEN_TABLE,
        PieceName::Rook => &EG_ROOK_TABLE,
        PieceName::Bishop => &EG_BISHOP_TABLE,
        PieceName::Knight => &EG_KNIGHT_TABLE,
        PieceName::Pawn => &EG_PAWN_TABLE,
    }
}

fn game_phase_value(piece: PieceName) -> i32 {
    match piece {
        PieceName::King => 0,
        PieceName::Queen => 4,
        PieceName::Rook => 2,
        PieceName::Bishop => 1,
        PieceName::Knight => 1,
        PieceName::Pawn => 0,
    }
}

/// https://www.chessprogramming.org/PeSTO%27s_Evaluation_Function
pub fn eval(board: &Board) -> i32 {
    let mut white_mg = 0;
    let mut white_eg = 0;
    let mut black_mg = 0;
    let mut black_eg = 0;
    let mut game_phase = 0;

    for square in Square::iter() {
        if board.piece_on_square(square).is_none() {
            continue;
        }
        let piece_name = board.piece_on_square(square).unwrap();
        game_phase += game_phase_value(piece_name);
        if let Some(piece) = board.piece_on_square(square) {
            match board.color_on_square(square) {
                Some(Color::White) => {
                    white_mg += get_mg_table(piece)[square.0 as usize ^ 56] + piece_value(piece);
                    white_eg += get_eg_table(piece)[square.0 as usize ^ 56] + piece_value(piece);
                }
                Some(Color::Black) => {
                    black_mg += get_mg_table(piece)[square.0 as usize ^ 56] + piece_value(piece);
                    black_eg += get_eg_table(piece)[square.0 as usize ^ 56] + piece_value(piece);
                }
                None => (),
            }
        }
    }

    let mg_pts;
    let eg_pts;
    match board.to_move {
        Color::White => {
            mg_pts = white_mg - black_mg;
            eg_pts = white_eg - black_eg;
        }
        Color::Black => {
            mg_pts = black_mg - white_mg;
            eg_pts = black_eg - white_eg;
        }
    }

    // 24 represents the highest normal value a user will have unless there are promotions before
    // captures
    let mg_phase = min(game_phase, 24);
    let eg_phase = 24 - mg_phase;
    (mg_pts * mg_phase + eg_pts * eg_phase) / 24
}

#[cfg(test)]
mod eval_test {
    use crate::{eval::eval, fen};

    #[test]
    fn starting_position() {
        let board = fen::build_board(fen::STARTING_FEN);
        assert_eq!(0, eval(&board));
    }

    #[test]
    fn test_positions() {
        let worst = fen::build_board("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1");
        let less_worse =
            fen::build_board("rnbqkbnr/1ppppppp/p7/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1");
        let good = fen::build_board("rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1");
        let also_good =
            fen::build_board("rnbqkb1r/pppppppp/5n2/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1");
        assert!(eval(&less_worse) > eval(&worst));
        assert!(eval(&good) > eval(&less_worse));
        assert!(eval(&also_good) > eval(&less_worse));
    }
}