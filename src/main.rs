mod uci;
mod pieces;
mod moves;
use pieces::{Piece};
use crate::moves::generate_all_moves;

mod board;
mod fen;

fn main() {
    // uci::main_loop();
    let board = fen::build_board(fen::STARTING_FEN);
    let moves = generate_all_moves(&board);
    for m in moves.iter() {
        board.print();
        m.print();
        let mut cloned_board = board.clone();
        cloned_board.make_move(m);
        cloned_board.print();
        println!("---------------------------------------------------------");
    }
}
