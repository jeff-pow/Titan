use core::fmt;
use std::fmt::Display;

use crate::{
    board,
    moves::Castle,
    moves::{in_check, EnPassant, Move, Promotion},
    pieces::Color,
    pieces::{opposite_color, PieceName, NUM_PIECES},
    Piece,
};

#[repr(C)]
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Board {
    pub board: [[u64; NUM_PIECES]; 2],
    pub to_move: Color,
    pub black_king_castle: bool,
    pub black_queen_castle: bool,
    pub white_king_castle: bool,
    pub white_queen_castle: bool,
    pub en_passant_square: i8,
    pub black_king_square: i8,
    pub white_king_square: i8,
    pub num_moves: i32,
}

fn flip_board(board: &Board) -> Board {
    let mut flipped_board = *board;
    flipped_board
        .board
        .iter_mut()
        .flatten()
        .for_each(|x| *x ^= 56);
    flipped_board
}

impl Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut str = String::new();
        let flipped_board = flip_board(self);
        for idx in 0..64 {
            if idx % 8 == 0 {
                str += &(8 - idx / 8).to_string();
                str += " ";
            }
            str += " | ";
            match self.to_move {
                Color::White => {
                    if self.square_contains_piece(PieceName::King, Color::White, idx) {
                        str += "K"
                    }
                    if self.square_contains_piece(PieceName::Queen, Color::White, idx) {
                        str += "Q"
                    }
                    if self.square_contains_piece(PieceName::Rook, Color::White, idx) {
                        str += "R"
                    }
                    if self.square_contains_piece(PieceName::Bishop, Color::White, idx) {
                        str += "B"
                    }
                    if self.square_contains_piece(PieceName::Knight, Color::White, idx) {
                        str += "N"
                    }
                    if self.square_contains_piece(PieceName::Pawn, Color::White, idx) {
                        str += "P"
                    }
                }
                Color::Black => {
                    if self.square_contains_piece(PieceName::King, Color::Black, idx) {
                        str += "k"
                    }
                    if self.square_contains_piece(PieceName::Queen, Color::Black, idx) {
                        str += "q"
                    }
                    if self.square_contains_piece(PieceName::Rook, Color::Black, idx) {
                        str += "r"
                    }
                    if self.square_contains_piece(PieceName::Bishop, Color::Black, idx) {
                        str += "b"
                    }
                    if self.square_contains_piece(PieceName::Knight, Color::Black, idx) {
                        str += "n"
                    }
                    if self.square_contains_piece(PieceName::Pawn, Color::Black, idx) {
                        str += "p"
                    }
                }
            }
            if (idx + 1) % 8 == 0 && idx != 0 {
                str += " |\n";
            }
        }
        str += "     a   b   c   d   e   f   g   h\n";
        write!(f, "{}", str)
    }
}

impl fmt::Debug for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut str = String::new();
        str += match self.to_move {
            Color::White => "White to move\n",
            Color::Black => "Black to move\n",
        };
        str += &self.to_string();
        str += "Castles available: ";
        if self.white_king_castle {
            str += "K"
        };
        if self.white_queen_castle {
            str += "Q"
        };
        if self.black_king_castle {
            str += "k"
        };
        if self.black_queen_castle {
            str += "q"
        };
        str += "\n";
        str += "Num moves made: ";
        str += &self.num_moves.to_string();
        write!(f, "{}", str)
    }
}

impl Board {
    pub fn new() -> Self {
        Board {
            board: [[0, 0, 0, 0, 0, 0], [0, 0, 0, 0, 0, 0]],
            black_king_castle: false,
            black_queen_castle: false,
            white_king_castle: false,
            white_queen_castle: false,
            to_move: Color::White,
            en_passant_square: -1,
            white_king_square: -1,
            black_king_square: -1,
            num_moves: 0,
        }
    }

    /// Function makes a move and modifies board state to reflect the move that just happened
    pub fn make_move(&mut self, m: &Move) {
        // Special case if the move is an en_passant
        if m.en_passant != EnPassant::None {
            let end_idx = m.end_idx as usize;
            match self.to_move {
                Color::White => {
                    self.remove_piece(PieceName::Pawn, opposite_color(self.to_move), end_idx - 8);
                }
                Color::Black => {
                    self.remove_piece(PieceName::Pawn, opposite_color(self.to_move), end_idx + 8);
                }
            }
        }

        self.place_piece(m.piece_moving, self.to_move, m.end_idx as usize);
        self.remove_piece(m.piece_moving, self.to_move, m.starting_idx as usize);

        // Move rooks if a castle move is applied
        match m.castle {
            Castle::WhiteQueenCastle => {
                self.place_piece(PieceName::Rook, self.to_move, 3);
                self.remove_piece(PieceName::Rook, self.to_move, 0);
                self.white_queen_castle = false;
                self.white_king_castle = false;
            }
            Castle::WhiteKingCastle => {
                self.place_piece(PieceName::Rook, self.to_move, 5);
                self.remove_piece(PieceName::Rook, self.to_move, 7);
                self.white_queen_castle = false;
                self.white_king_castle = false;
            }
            Castle::BlackKingCastle => {
                self.place_piece(PieceName::Rook, self.to_move, 61);
                self.remove_piece(PieceName::Rook, self.to_move, 63);
                self.black_queen_castle = false;
                self.black_king_castle = false;
            }
            Castle::BlackQueenCastle => {
                self.place_piece(PieceName::Rook, self.to_move, 59);
                self.remove_piece(PieceName::Rook, self.to_move, 56);
                self.black_queen_castle = false;
                self.black_king_castle = false;
            }
            Castle::None => (),
        }
        // If move is a promotion, a pawn is removed from the board and replaced with a higher
        // value piece
        match m.promotion {
            Promotion::Queen => {
                self.place_piece(PieceName::Queen, self.to_move, m.end_idx as usize);
            }
            Promotion::Rook => {
                self.place_piece(PieceName::Rook, self.to_move, m.end_idx as usize);
            }
            Promotion::Bishop => {
                self.place_piece(PieceName::Bishop, self.to_move, m.end_idx as usize);
            }
            Promotion::Knight => {
                self.place_piece(PieceName::Knight, self.to_move, m.end_idx as usize);
            }
            Promotion::None => (),
        }
        // Update board's king square if king moves
        if m.piece_moving == PieceName::King {
            match self.to_move {
                Color::White => {
                    self.white_king_castle = false;
                    self.white_queen_castle = false;
                }
                Color::Black => {
                    self.black_queen_castle = false;
                    self.black_king_castle = false;
                }
            }
        }
        // If a rook moves, castling to that side is no longer possible
        if m.piece_moving == PieceName::Rook {
            match m.starting_idx {
                0 => self.white_queen_castle = false,
                7 => self.white_king_castle = false,
                56 => self.black_queen_castle = false,
                63 => self.black_king_castle = false,
                _ => (),
            }
        }
        // If a rook is captured, castling is no longer possible
        if let Some(cap) = m.capture {
            if cap == PieceName::Rook {
                match m.end_idx {
                    0 => self.white_queen_castle = false,
                    7 => self.white_king_castle = false,
                    56 => self.black_queen_castle = false,
                    63 => self.black_king_castle = false,
                    _ => (),
                }
            }
        }
        // If the end index of a move is 16 squares from the start, an en passant is possible
        let mut en_passant = false;
        if m.piece_moving == PieceName::Pawn {
            match self.to_move {
                Color::White => {
                    if m.starting_idx == m.end_idx - 16 {
                        en_passant = true;
                        self.en_passant_square = m.end_idx - 8;
                    }
                }
                Color::Black => {
                    if m.end_idx + 16 == m.starting_idx {
                        en_passant = true;
                        self.en_passant_square = m.starting_idx - 8;
                    }
                }
            }
        }
        // If en passant was not performed this move, the ability to do it on future moves goes away
        if !en_passant {
            self.en_passant_square = -1;
        }
        // Update castling ability based on check
        match self.to_move {
            Color::White => {
                if in_check(self, Color::White) {
                    self.white_king_castle = false;
                    self.white_queen_castle = false;
                }
            }
            Color::Black => {
                if in_check(self, Color::Black) {
                    self.black_king_castle = false;
                    self.black_queen_castle = false;
                }
            }
        }
        // Change the side to move after making a move
        match self.to_move {
            Color::White => self.to_move = Color::Black,
            Color::Black => self.to_move = Color::White,
        }
        self.num_moves += 1;
    }

    pub fn square_contains_piece(&self, piece_type: PieceName, color: Color, idx: usize) -> bool {
        self.board[color as usize][piece_type as usize] & (1 << idx) != 0
    }

    pub fn piece_on_square(&self, idx: usize) -> Option<PieceName> {
        if self.square_contains_piece(PieceName::King, Color::White, idx) {
            return Some(PieceName::King);
        }
        if self.square_contains_piece(PieceName::Queen, Color::White, idx) {
            return Some(PieceName::Queen);
        }
        if self.square_contains_piece(PieceName::Rook, Color::White, idx) {
            return Some(PieceName::Rook);
        }
        if self.square_contains_piece(PieceName::Bishop, Color::White, idx) {
            return Some(PieceName::Bishop);
        }
        if self.square_contains_piece(PieceName::Knight, Color::White, idx) {
            return Some(PieceName::Knight);
        }
        if self.square_contains_piece(PieceName::Pawn, Color::White, idx) {
            return Some(PieceName::Pawn);
        }

        if self.square_contains_piece(PieceName::King, Color::Black, idx) {
            return Some(PieceName::King);
        }
        if self.square_contains_piece(PieceName::Queen, Color::Black, idx) {
            return Some(PieceName::Queen);
        }
        if self.square_contains_piece(PieceName::Rook, Color::Black, idx) {
            return Some(PieceName::Rook);
        }
        if self.square_contains_piece(PieceName::Bishop, Color::Black, idx) {
            return Some(PieceName::Bishop);
        }
        if self.square_contains_piece(PieceName::Knight, Color::Black, idx) {
            return Some(PieceName::Knight);
        }
        if self.square_contains_piece(PieceName::Pawn, Color::Black, idx) {
            return Some(PieceName::Pawn);
        }
        None
    }

    pub fn place_piece(&self, piece_type: PieceName, color: Color, idx: usize) {
        self.board[color as usize][piece_type as usize] &= 1 << idx;
        if piece_type == PieceName::King {
            match color {
                Color::White => self.white_king_square = idx as i8,
                Color::Black => self.black_king_square = idx as i8,
            }
        }
    }

    pub fn remove_piece(&self, piece_type: PieceName, color: Color, idx: usize) {
        self.board[color as usize][piece_type as usize] &= 0 << idx;
    }
}
