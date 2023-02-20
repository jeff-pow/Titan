mod moves;
mod pieces;
mod uci;
use crate::moves::generate_all_moves;
use pieces::Piece;

mod board;
mod fen;

fn main() {
    // uci::main_loop();
    let board = fen::build_board(fen::ONE_QUEEN);
    board.print();
    let moves = generate_all_moves(&board);
    for m in moves.iter() {
        m.print();
        let mut cloned_board = board.clone();
        cloned_board.make_move(m);
        cloned_board.print();
        println!("---------------------------------------------------------");
    }
    println!("Moves: {}", moves.len());
}
