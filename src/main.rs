mod moves;
mod pieces;
mod uci;

use std::mem;
use crate::moves::{generate_all_moves, Move};
use pieces::Piece;

mod board;
mod fen;

fn main() {
    // uci::main_loop();
    let board = fen::build_board(fen::ONE_PIECE);
    board.print();
    let moves = generate_all_moves(&board);
    for m in moves.iter() {
        m.print();
        println!("{}", m.to_lan());
        let mut cloned_board = board.clone();
        cloned_board.make_move(m);
        cloned_board.print();
        println!("---------------------------------------------------------");
    }
    println!("Size of board: {}", mem::size_of_val(&board));
    println!("Moves: {}", moves.len());
}
