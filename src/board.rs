use crate::{Piece, pieces::Color, pieces::PieceName, moves::Move, moves::Castle};

#[derive(Clone)]
pub struct Board {
    pub board: [Option<Piece>; 64],
    pub to_move: Color,
    pub black_king_castle: bool,
    pub black_queen_castle: bool,
    pub white_king_castle: bool,
    pub white_queen_castle: bool,
    pub en_passant_square: u8,
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
            en_passant_square: 0,
        }
    }
    
    pub fn make_move(&mut self, chess_move: &Move) {
        let mut piece = &mut self.board[chess_move.starting_idx as usize].unwrap();
        match chess_move.castle {
            Castle::WhiteKingCastle => {
                let mut rook = &mut self.board[7];
                self.board[5] = self.board[7];
                self.board[7] = None;
                piece.current_square = 5;
            }
            Castle::WhiteQueenCastle => {
                let mut rook = &mut self.board[7];
                self.board[3] = self.board[0];
                self.board[0] = None;
                piece.current_square = 3;
            }
            Castle::BlackKingCastle => {
                let mut rook = &mut self.board[63];
                self.board[61] = self.board[63];
                self.board[63] = None;
                piece.current_square = 61;
            }
            Castle::BlackQueenCastle => {
                let mut rook = &mut self.board[56];
                self.board[59] = self.board[56];
                self.board[56] = None;
                piece.current_square = 59;
            }
            Castle::None => (),
        }
        let piece_old_idx = piece.current_square;
        self.board[piece.current_square as usize] = None;
        self.board[chess_move.end_idx as usize] = Some(*piece);
        piece.current_square = chess_move.end_idx;
    }

    pub fn print(&self) {
        let flipped_board = flip_board(&self);
        for (idx, square) in flipped_board.board.iter().enumerate() {
            print!(" | ");
            if idx == 27 {
                print!("X");
                continue;
            }
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

fn flip_board(board: &Board) -> Board {
    let mut flipped_board = Board::new();
    let rows_vec: Vec<Vec<Option<Piece>>> = board.board.chunks(8).map(|e| e.into()).collect();
    let mut white_pov: Vec<Option<Piece>> = Vec::new();
    for row in rows_vec.iter().rev() {
        for square in row {
            white_pov.push(*square);
        }
    }
    let mut white_pov_arr: [Option<Piece>; 64] = [None; 64];
    for (idx, piece) in white_pov.iter().enumerate() {
        white_pov_arr[idx] = *piece;
    }
    flipped_board.board = white_pov_arr;
    flipped_board
}
