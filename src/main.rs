mod uci;
mod pieces;
mod moves;
use pieces::{Piece};
mod board;
mod fen;

fn main() {
    //uci::main_loop();
    let string = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    let string2 = "rnbqkbnr/pp1ppppp/8/2p5/4P3/8/PPPP1PPP/RNBQKBNR w KQkq c6 0 2";
    fen::build_board(string);
}
