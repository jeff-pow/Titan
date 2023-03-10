use core::fmt;
use std::{fmt::Display, cell::RefCell};

use crate::{moves::Castle, moves::{Move, Promotion}, pieces::Color, pieces::PieceName, Piece};

#[repr(C)]
#[derive(Clone, Debug)]
pub struct Board {
    pub board: [Option<Piece>; 64],
    pub to_move: Color,
    pub black_king_castle: bool,
    pub black_queen_castle: bool,
    pub white_king_castle: bool,
    pub white_queen_castle: bool,
    pub en_passant_square: i8,
    pub black_king_square: i8,
    pub white_king_square: i8,
    pub white_pieces: RefCell<Vec<Piece>>,
    pub black_pieces: RefCell<Vec<Piece>>,
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
            en_passant_square: -1,
            white_king_square: -1,
            black_king_square: -1,
            white_pieces: RefCell::new(Vec::with_capacity(16)),
            black_pieces: RefCell::new(Vec::with_capacity(16)),
        }
    }

    pub fn make_move(&mut self, chess_move: &Move) {
        // Special case if the move is an en_passant
        if chess_move.piece_moving == PieceName::Pawn && chess_move.end_idx == self.en_passant_square {
            let end_idx = chess_move.end_idx as usize;
            match self.to_move {
                Color::White => {
                    if self.board[end_idx].is_none()
                        && self.board[end_idx + 8].is_none()
                        && self.board[end_idx - 8].is_some()
                        && self.board[end_idx - 8].unwrap().piece_name == PieceName::Pawn {

                        let p = &self.board[end_idx - 8].unwrap();
                        self.black_pieces.borrow_mut().retain(|piece| piece.current_square != p.current_square);
                        self.board[end_idx - 8] = None;
                    }
                }
                Color::Black => {
                    if self.board[end_idx].is_none()
                        && self.board[end_idx - 8].is_none()
                        && self.board[end_idx + 8].is_some()
                        && self.board[end_idx + 8].unwrap().piece_name == PieceName::Pawn {

                        let p = &self.board[end_idx + 8].unwrap();
                        self.white_pieces.borrow_mut().retain(|piece| piece.current_square != p.current_square);
                        self.board[end_idx + 8] = None;
                    }
                }
            }
        }

        // If the end idx of the board contains a piece, remove that piece from the vector that
        // stores piece positions
        if let Some(p) = &self.board[chess_move.end_idx as usize] {
            match self.to_move {
                Color::White => {
                    self.black_pieces.borrow_mut().retain(|piece| piece.current_square != p.current_square)
                }
                Color::Black => {
                    self.white_pieces.borrow_mut().retain(|piece| piece.current_square != p.current_square);
                }
            }
        }

        let mut piece = &mut self.board[chess_move.starting_idx as usize].
            expect("There should be a piece here");
        piece.current_square = chess_move.end_idx;
        self.board[chess_move.end_idx as usize] = Option::from(*piece);
        self.board[chess_move.starting_idx as usize] = None;
        match self.to_move {
            Color::White => {
                if let Some(mut p) = self.white_pieces.borrow_mut().iter_mut().find(|x| x.current_square == chess_move.starting_idx) {
                    p.current_square = chess_move.end_idx;
                }
            }
            Color::Black => {
                if let Some(mut p) = self.black_pieces.borrow_mut().iter_mut().find(|x| x.current_square == chess_move.starting_idx) {
                    p.current_square = chess_move.end_idx;
                }
            }
        }

        // Move rooks if a castle is applied
        match chess_move.castle {
            Castle::WhiteQueenCastle => {
                let mut rook = &mut self.board[0].expect("Piece should be here: 0");
                rook.current_square = 5;
                self.board[5] = Option::from(*rook);
                self.board[7] = None;
                self.white_queen_castle = false;
                self.white_king_castle = false;
            }
            Castle::WhiteKingCastle => {
                let mut rook = &mut self.board[7].expect("Piece should be here: 7");
                rook.current_square = 3;
                self.board[3] = Option::from(*rook);
                self.board[0] = None;
                self.white_queen_castle = false;
                self.white_king_castle = false;
            }
            Castle::BlackKingCastle => {
                let mut rook = &mut self.board[63].expect("Piece should be here: 63");
                rook.current_square = 61;
                self.board[61] = Option::from(*rook);
                self.board[63] = None;
                self.black_queen_castle = false;
                self.black_king_castle = false;
            }
            Castle::BlackQueenCastle => {
                let mut rook = &mut self.board[56].expect("Piece should be here: 56");
                rook.current_square = 59;
                self.board[59] = Option::from(*rook);
                self.board[56] = None;
                self.black_queen_castle = false;
                self.black_king_castle = false;
            }
            Castle::None => (),
        }
        // If move is a promotion, a pawn is promoted
        match chess_move.promotion {
            Promotion::Queen => {
                match self.to_move {
                    Color::White => {
                        self.white_pieces.borrow_mut().retain(|piece| piece.current_square != chess_move.end_idx);
                        self.white_pieces.borrow_mut().push(Piece::new(piece.color, PieceName::Queen, chess_move.end_idx));
                    }
                    Color::Black => {
                        self.black_pieces.borrow_mut().retain(|piece| piece.current_square != chess_move.end_idx);
                        self.black_pieces.borrow_mut().push(Piece::new(piece.color, PieceName::Queen, chess_move.end_idx));
                    }
                }
                self.board[chess_move.end_idx as usize] = Some(Piece {
                    current_square: chess_move.end_idx,
                    color: piece.color,
                    piece_name: PieceName::Queen,
                });
            }
            Promotion::Rook => {
                match self.to_move {
                    Color::White => {
                        self.white_pieces.borrow_mut().retain(|piece| piece.current_square != chess_move.end_idx);
                        self.white_pieces.borrow_mut().push(Piece::new(piece.color, PieceName::Rook, chess_move.end_idx));
                    }
                    Color::Black => {
                        self.black_pieces.borrow_mut().retain(|piece| piece.current_square != chess_move.end_idx);
                        self.black_pieces.borrow_mut().push(Piece::new(piece.color, PieceName::Rook, chess_move.end_idx));
                    }
                }
                self.board[chess_move.end_idx as usize] = Some(Piece {
                    current_square: chess_move.end_idx,
                    color: piece.color,
                    piece_name: PieceName::Rook,
                });
            }
            Promotion::Bishop => {
                match self.to_move {
                    Color::White => {
                        self.white_pieces.borrow_mut().retain(|piece| piece.current_square != chess_move.end_idx);
                        self.white_pieces.borrow_mut().push(Piece::new(piece.color, PieceName::Bishop, chess_move.end_idx));
                    }
                    Color::Black => {
                        self.black_pieces.borrow_mut().retain(|piece| piece.current_square != chess_move.end_idx);
                        self.black_pieces.borrow_mut().push(Piece::new(piece.color, PieceName::Bishop, chess_move.end_idx));
                    }
                }
                self.board[chess_move.end_idx as usize] = Some(Piece {
                    current_square: chess_move.end_idx,
                    color: piece.color,
                    piece_name: PieceName::Bishop,
                });
            }
            Promotion::Knight => {
                match self.to_move {
                    Color::White => {
                        self.white_pieces.borrow_mut().retain(|piece| piece.current_square != chess_move.end_idx);
                        self.white_pieces.borrow_mut().push(Piece::new(piece.color, PieceName::Knight, chess_move.end_idx));
                    }
                    Color::Black => {
                        self.black_pieces.borrow_mut().retain(|piece| piece.current_square != chess_move.end_idx);
                        self.black_pieces.borrow_mut().push(Piece::new(piece.color, PieceName::Knight, chess_move.end_idx));
                    }
                }
                self.board[chess_move.end_idx as usize] = Some(Piece {
                    current_square: chess_move.end_idx,
                    color: piece.color,
                    piece_name: PieceName::Knight,
                });
            }
            Promotion::None => (),
        }
        // Change the side to move after making a move
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
        // If the end index of a move is 16 squares from the start, an en passant is possible
        let mut en_passant = false;
        if piece.piece_name == PieceName::Pawn {
            match piece.color {
                Color::White => {
                    if chess_move.starting_idx == chess_move.end_idx - 16 {
                        en_passant = true;
                        self.en_passant_square = chess_move.end_idx - 8;
                    }
                }
                Color::Black => {
                    if chess_move.end_idx + 16 == chess_move.starting_idx {
                        en_passant = true;
                        self.en_passant_square = chess_move.starting_idx - 8;
                    }
                }
            }
        }
        if !en_passant {
            self.en_passant_square = -1;
        }
        // If a rook is captured, en_passant is no longer possible
        if let Some(cap) = chess_move.capture {
            if cap.piece_name == PieceName::Rook {
                match cap.current_square {
                    0 => self.white_queen_castle = false,
                    7 => self.white_king_castle = false,
                    56 => self.black_queen_castle = false,
                    63 => self.black_king_castle = false,
                    _ => (),
                }
            }
        }
    }

    pub fn evaluation(&self) -> i32 {
        let mut white = 0;
        let mut black = 0;
        for square in self.board {
            match square {
                None => continue,
                Some(piece) => {
                    match piece.color {
                        Color::White => {
                            white += piece.value();
                        }
                        Color::Black => {
                            black += piece.value();
                        }
                    }
                }
            }
        }
        if self.to_move == Color::White { white - black } else { black - white }
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

#[cfg(test)]
mod board_tests {
    use crate::fen;

    #[test]
    fn test_board_eval() {
    }
}
