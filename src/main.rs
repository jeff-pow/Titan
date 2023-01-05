mod uci;
mod pieces;
mod moves;
use pieces::{Piece};
mod board;
mod fen;

fn main() {
    // uci::main_loop();
    let board = fen::build_board(fen::TEST_FEN);
    board.print_board();
    /*
    println!();
    println!();
    let board = fen::build_board(fen::STARTING_FEN);
    board.print_board();
     */
}
