mod moves;
mod pieces;
mod uci;
use crate::moves::generate_all_moves;
use moves::from_lan;
use pieces::Piece;
mod board;
mod fen;
use rand::seq::SliceRandom;

fn main() {
    uci::main_loop();
    let board = fen::build_board(fen::ONE_PIECE);
    board.print();
    let _m = generate_all_moves(&board)
        .choose(&mut rand::thread_rng())
        .unwrap();
    let m = from_lan("f2e3", &board);
    m.print();
    let mut cloned_board = board;
    cloned_board.make_move(&m);
    cloned_board.print();
}

fn print_moves(moves: &Vec<moves::Move>, board: &board::Board) {
    for m in moves.iter() {
        m.print();
        let mut cloned_board = board.clone();
        cloned_board.make_move(m);
        cloned_board.print();
        println!("---------------------------------------------------------");
    }
}
