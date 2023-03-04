use core::fmt;
use std::fmt::Display;

use crate::{moves::Castle, moves::Move, pieces::Color, pieces::PieceName, Piece};

#[derive(Clone, Debug)]
pub struct Board {
    pub board: [Option<Piece>; 64],
    pub to_move: Color,
    pub black_king_castle: bool,
    pub black_queen_castle: bool,
    pub white_king_castle: bool,
    pub white_queen_castle: bool,
    pub en_passant_square: u8,
    pub black_king_square: i8,
    pub white_king_square: i8,
}

impl Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut str = String::new();
        let flipped_board = flip_board(self);
        for (idx, square) in flipped_board.board.iter().enumerate() {
            if idx % 8 == 0 {
                str += &(8 - idx / 8).to_string();
                str += " ";
            }
            str += " | ";
            match square {
                None => str += "_",
                Some(piece) => match piece.color {
                    Color::White => match piece.piece_name {
                        PieceName::King => str += "K",
                        PieceName::Queen => str += "Q",
                        PieceName::Rook => str += "R",
                        PieceName::Bishop => str += "B",
                        PieceName::Knight => str += "N",
                        PieceName::Pawn => str += "P",
                    },
                    Color::Black => match piece.piece_name {
                        PieceName::King => str += "k",
                        PieceName::Queen => str += "q",
                        PieceName::Rook => str += "r",
                        PieceName::Bishop => str += "b",
                        PieceName::Knight => str += "n",
                        PieceName::Pawn => str += "p",
                    },
                },
            }
            if (idx + 1) % 8 == 0 && idx != 0 {
                str += " |\n";
            }
        }
        str += "     a   b   c   d   e   f   g   h\n";
        write!(f, "{}", str)
    }
}

impl Board {
    pub fn new() -> Self {
        Board {
            board: [None; 64],
            black_king_castle: false,
            black_queen_castle: false,
            white_king_castle: false,
            white_queen_castle: false,
            to_move: Color::White,
            en_passant_square: 0,
            white_king_square: -1,
            black_king_square: -1,
        }
    }

    pub fn make_move(&mut self, chess_move: &Move) {
        let mut piece = &mut self.board[chess_move.starting_idx as usize].
            expect("There should be a piece here");
        piece.current_square = chess_move.end_idx;
        self.board[chess_move.end_idx as usize] = Option::from(*piece);
        self.board[chess_move.starting_idx as usize] = None;
        // Move rooks if a castle is applied
        match chess_move.castle {
            Castle::WhiteKingCastle => {
                let mut rook = &mut self.board[0].expect("Piece should be here: 0");
                rook.current_square = 5;
                self.board[5] = self.board[7];
                self.board[7] = None;
                self.white_queen_castle = false;
                self.white_king_castle = false;
            }
            Castle::WhiteQueenCastle => {
                let mut rook = &mut self.board[7].expect("Piece should be here: 7");
                rook.current_square = 3;
                self.board[3] = self.board[0];
                self.board[0] = None;
                self.white_queen_castle = false;
                self.white_king_castle = false;
            }
            Castle::BlackKingCastle => {
                let mut rook = &mut self.board[63].expect("Piece should be here: 63");
                rook.current_square = 61;
                self.board[61] = self.board[63];
                self.board[63] = None;
                self.black_queen_castle = false;
                self.black_king_castle = false;
            }
            Castle::BlackQueenCastle => {
                let mut rook = &mut self.board[56].expect("Piece should be here: 56");
                rook.current_square = 59;
                self.board[59] = self.board[56];
                self.board[56] = None;
                self.black_queen_castle = false;
                self.black_king_castle = false;
            }
            Castle::None => (),
        }
        // If move is a promotion, a pawn is promoted
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
        // Update king square if king moves
        if piece.piece_name == PieceName::King {
            match piece.color {
                Color::White => {
                    self.white_king_square = piece.current_square;
                    self.white_king_castle = false;
                    self.white_queen_castle = false;
                }
                Color::Black => {
                    self.black_king_square = piece.current_square;
                    self.black_queen_castle = false;
                    self.black_king_castle = false;
                }
            }
        }
        // If a rook moves, castling to that side is no longer possible
        if piece.piece_name == PieceName::Rook {
            match chess_move.starting_idx {
                0 => self.white_queen_castle = false,
                7 => self.white_king_castle = false,
                56 => self.black_queen_castle = false,
                63 => self.black_king_castle = false,
                _ => (),
            }
        }
        if piece.piece_name == PieceName::Pawn {
            match piece.color {
                Color::White => {
                    if chess_move.starting_idx == chess_move.end_idx - 16 {
                        self.en_passant_square = chess_move.end_idx as u8 - 8;
                    }
                }
                Color::Black => {
                    if chess_move.end_idx + 16 == chess_move.starting_idx {
                        self.en_passant_square = chess_move.starting_idx as u8 - 8;
                    }
                }
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
