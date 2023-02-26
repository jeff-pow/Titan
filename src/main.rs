mod moves;
mod pieces;
mod uci;
use pieces::Piece;
mod board;
mod fen;

fn main() {
    uci::main_loop();
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
