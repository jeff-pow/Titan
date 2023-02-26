use crate::{moves::Castle, moves::Move, pieces::Color, pieces::PieceName, Piece};

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
        self.board[chess_move.end_idx as usize] = self.board[chess_move.starting_idx as usize];
        self.board[piece.current_square as usize] = None;
        piece.current_square = chess_move.end_idx;
        // Move rooks if a castle is applied
        match chess_move.castle {
            Castle::WhiteKingCastle => {
                let mut rook = &mut self.board[0].unwrap();
                rook.current_square = 5;
                self.board[5] = self.board[7];
                self.board[7] = None;
            }
            Castle::WhiteQueenCastle => {
                let mut rook = &mut self.board[7].unwrap();
                rook.current_square = 3;
                self.board[3] = self.board[0];
                self.board[0] = None;
            }
            Castle::BlackKingCastle => {
                let mut rook = &mut self.board[63].unwrap();
                rook.current_square = 61;
                self.board[61] = self.board[63];
                self.board[63] = None;
            }
            Castle::BlackQueenCastle => {
                let mut rook = &mut self.board[56].unwrap();
                rook.current_square = 59;
                self.board[59] = self.board[56];
                self.board[56] = None;
            }
            Castle::None => (),
        }
        match chess_move.promotion {
            true => {
                self.board[chess_move.end_idx as usize] = Some(Piece {
                    current_square: chess_move.end_idx,
                    color: piece.color,
                    piece_name: PieceName::Queen,
                })
            }
            false => {}
        }
        // Method changes the side to move after making a move
        match self.to_move {
            Color::White => self.to_move = Color::Black,
            Color::Black => self.to_move = Color::White,
        }
    }

    pub fn print(&self) {
        let flipped_board = flip_board(self);
        for (idx, square) in flipped_board.board.iter().enumerate() {
            if idx % 8 == 0 {
                print!("{} ", 8 - idx / 8);
            }
            print!(" | ");
            match square {
                None => print!("_"),
                Some(piece) => match piece.color {
                    Color::White => match piece.piece_name {
                        PieceName::King => print!("K"),
                        PieceName::Queen => print!("Q"),
                        PieceName::Rook => print!("R"),
                        PieceName::Bishop => print!("B"),
                        PieceName::Knight => print!("N"),
                        PieceName::Pawn => print!("P"),
                    },
                    Color::Black => match piece.piece_name {
                        PieceName::King => print!("k"),
                        PieceName::Queen => print!("q"),
                        PieceName::Rook => print!("r"),
                        PieceName::Bishop => print!("b"),
                        PieceName::Knight => print!("n"),
                        PieceName::Pawn => print!("p"),
                    },
                },
            }
            if (idx + 1) % 8 == 0 && idx != 0 {
                println!(" |");
            }
        }
        println!("     a   b   c   d   e   f   g   h");
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
