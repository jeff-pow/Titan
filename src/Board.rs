mod Pieces;
use Pieces::{Piece, Color, PieceName};

struct Board {
    board: [Option<Piece>; 64],
}

impl Board {
    pub fn new() -> Self {
        Board { board: [None; 64] }
    }
    pub fn place_piece(&mut self, piece: &mut Piece, &new_idx: u8) {
        let piece_old_idx = piece.piece_num();
        self.board[piece_old_idx] = None;
        self.board[new_idx] = piece;
    }
}
