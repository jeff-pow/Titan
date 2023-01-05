use crate::{Piece, pieces::Color, pieces::PieceName};

pub struct Board {
    pub board: [Option<Piece>; 64],
    pub to_move: Color,
    pub black_king_castle: bool,
    pub black_queen_castle: bool,
    pub white_king_castle: bool,
    pub white_queen_castle: bool,
}

impl Board {
    pub fn new() -> Self {
        Board { 
            board: [None; 64],
            black_king_castle: true,
            black_queen_castle: true,
            white_king_castle: true,
            white_queen_castle: true,
            to_move: Color::White,
        }
    }
    
    pub fn place_piece(&mut self, piece: &mut Piece, new_idx: u8) {
        let piece_old_idx = piece.current_square;
        self.board[piece_old_idx as usize] = None;
        self.board[new_idx as usize] = Some(*piece);
        piece.change_square(new_idx);
    }

    pub fn print_board(&self) {
        for (idx, square) in self.board.iter().rev().enumerate() {
            print!(" | ");
            match square {
                None => print!("_"),
                Some(piece) => {
                    match piece.color {
                        Color::White => {
                            match piece.piece_name {
                                PieceName::King => print!("K"),
                                PieceName::Queen => print!("Q"),
                                PieceName::Rook => print!("R"),
                                PieceName::Bishop => print!("B"),
                                PieceName::Knight => print!("N"),
                                PieceName::Pawn => print!("P"),
                            }
                        }
                        Color::Black => {
                            match piece.piece_name {
                                PieceName::King => print!("k"),
                                PieceName::Queen => print!("q"),
                                PieceName::Rook => print!("r"),
                                PieceName::Bishop => print!("b"),
                                PieceName::Knight => print!("n"),
                                PieceName::Pawn => print!("p"),
                            }
                        }
                    }
                }
            }
            if (idx + 1) % 8 == 0  && idx != 0 {
                println!(" |");
            }
        }
    }
}
