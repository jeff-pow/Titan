mod UCI;
mod Pieces;
mod Moves;
use Pieces::{Piece, PieceName, Color};

fn main() {
    UCI::main_loop();
}
