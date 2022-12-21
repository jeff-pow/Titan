mod UCI;
mod Pieces;
mod Moves;
use Pieces::{Piece, PieceName, Color};
mod Board;

fn main() {
    UCI::main_loop();
}
