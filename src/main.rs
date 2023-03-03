mod moves;
mod pieces;
mod uci;
use pieces::Piece;
use crate::moves::{check_check, generate_all_moves};

mod board;
mod fen;

fn main() {
    // Ran into an error with | position startpos moves e2e4 a7a5 e1e2 a5a4 e2f3 a4a3 g1h3 a3b2 c1b2 b7b5 b2f6 b5b4 f6e7 b4b3 c2b3 c7c5 d2d4 c5c4 d4d5 c4c3 b1c3 d7d6 d1d4 f7f5 e4f5 g7g5 d4f6 g5g4 f3g4 h7h5 g4f4 h5h4 f6f7 e8d7 f7e8 d8e8 e7f8 h8h5 f4g4 h5g5 g4g5 g8f6 g5f6 e8d8 f8e7 d8a5 e7d8 a5b5 d8b6 b5c5 c3a4
    //uci::main_loop();
    print_moves();
}

#[allow(dead_code)]
fn print_moves() {
    //let board = fen::build_board("rnb5/3k4/1B1p1K2/2qP1P2/N6p/1P5N/P4PPP/R4B1R b - - 8 26");
    let board = fen::build_board(fen::STARTING_FEN);
    let mut moves = generate_all_moves(&board);
    check_check(&board, &mut moves);
    for m in moves.iter() {
        println!("{}", m);
        let mut cloned_board = board.clone();
        cloned_board.make_move(m);
        println!("{}", cloned_board);
        println!("---------------------------------------------------------");
    }
    println!("{} moves possible", moves.len());
}
