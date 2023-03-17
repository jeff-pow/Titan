use core::fmt;
use std::fmt::Display;

use crate::{
    moves::Castle,
    moves::{in_check, EnPassant, Move, Promotion},
    pieces::Color,
    pieces::PieceName,
    Piece,
};

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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
    pub num_moves: i32,
    pub num_white_pieces: i8,
    pub num_black_pieces: i8,
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
            num_moves: 0,
            num_white_pieces: 0,
            num_black_pieces: 0,
        }
    }

    pub fn make_move(&mut self, m: &Move) {
        // Special case if the move is an en_passant
        if m.en_passant != EnPassant::None {
            let end_idx = m.end_idx as usize;
            match self.to_move {
                Color::White => {
                    self.board[end_idx - 8] = None;
                }
                Color::Black => {
                    self.board[end_idx + 8] = None;
                }
            }
        }

        let mut piece =
            &mut self.board[m.starting_idx as usize].expect("There should be a piece here");
        piece.current_square = m.end_idx;
        self.board[m.end_idx as usize] = Option::from(*piece);
        self.board[m.starting_idx as usize] = None;

        // Move rooks if a castle is applied
        match m.castle {
            Castle::WhiteQueenCastle => {
                let mut rook = &mut self.board[0].expect("Piece should be here: 0");
                rook.current_square = 3;
                self.board[3] = Option::from(*rook);
                self.board[0] = None;
                self.white_queen_castle = false;
                self.white_king_castle = false;
            }
            Castle::WhiteKingCastle => {
                let mut rook = &mut self.board[7].expect("Piece should be here: 7");
                rook.current_square = 5;
                self.board[5] = Option::from(*rook);
                self.board[7] = None;
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
        if let Some(capture) = m.capture {
            match capture.piece_name {
                PieceName::King | PieceName::Pawn => (),
                _ => match self.to_move {
                    Color::White => self.num_black_pieces -= 1,
                    Color::Black => self.num_white_pieces -= 1,
                },
            }
        }
        // If move is a promotion, a pawn is promoted
        match m.promotion {
            Promotion::Queen => {
                self.board[m.end_idx as usize] = Some(Piece {
                    current_square: m.end_idx,
                    color: piece.color,
                    piece_name: PieceName::Queen,
                });
            }
            Promotion::Rook => {
                self.board[m.end_idx as usize] = Some(Piece {
                    current_square: m.end_idx,
                    color: piece.color,
                    piece_name: PieceName::Rook,
                });
            }
            Promotion::Bishop => {
                self.board[m.end_idx as usize] = Some(Piece {
                    current_square: m.end_idx,
                    color: piece.color,
                    piece_name: PieceName::Bishop,
                });
            }
            Promotion::Knight => {
                self.board[m.end_idx as usize] = Some(Piece {
                    current_square: m.end_idx,
                    color: piece.color,
                    piece_name: PieceName::Knight,
                });
            }
            Promotion::None => (),
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
        // If the end index of a move is 16 squares from the start, an en passant is possible
        let mut en_passant = false;
        if piece.piece_name == PieceName::Pawn {
            match piece.color {
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
        // If en passant was not performed that move, the ability to do it goes away
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

    #[allow(dead_code)]
    pub fn unmake_move(&mut self, m: &Move) {
        if m.en_passant != EnPassant::None {
            let end_idx = m.end_idx as usize;
            match self.to_move {
                Color::White => {
                    self.board[end_idx + 8] = m.capture;
                    self.en_passant_square = m.end_idx;
                }
                Color::Black => {
                    self.board[end_idx - 8] = m.capture;
                    self.en_passant_square = m.end_idx;
                }
            }
        } else {
            self.en_passant_square = -1;
        }
        let mut piece = &mut self.board[m.end_idx as usize].expect("There should be a piece here");
        piece.current_square = m.starting_idx;
        self.board[m.starting_idx as usize] = Option::from(*piece);
        if m.en_passant == EnPassant::None {
            self.board[m.end_idx as usize] = m.capture;
        } else {
            self.board[m.end_idx as usize] = None;
        }
        match m.castle {
            Castle::WhiteQueenCastle => {
                let mut rook = &mut self.board[3].expect("Piece should be here: 0");
                rook.current_square = 0;
                self.board[0] = Option::from(*rook);
                self.board[3] = None;
                self.white_queen_castle = true;
                self.white_king_castle = true;
            }
            Castle::WhiteKingCastle => {
                let mut rook = &mut self.board[5].expect("Piece should be here: 7");
                rook.current_square = 7;
                self.board[7] = Option::from(*rook);
                self.board[5] = None;
                self.white_queen_castle = true;
                self.white_king_castle = true;
            }
            Castle::BlackKingCastle => {
                let mut rook = &mut self.board[61].expect("Piece should be here: 63");
                rook.current_square = 63;
                self.board[63] = Option::from(*rook);
                self.board[61] = None;
                self.black_queen_castle = true;
                self.black_king_castle = true;
            }
            Castle::BlackQueenCastle => {
                let mut rook = &mut self.board[59].expect("Piece should be here: 56");
                rook.current_square = 56;
                self.board[56] = Option::from(*rook);
                self.board[59] = None;
                self.black_queen_castle = true;
                self.black_king_castle = true;
            }
            Castle::None => (),
        }
        if let Some(capture) = m.capture {
            match capture.piece_name {
                PieceName::King | PieceName::Pawn => (),
                _ => match self.to_move {
                    Color::White => self.num_black_pieces += 1,
                    Color::Black => self.num_white_pieces += 1,
                },
            }
        }
        match m.promotion {
            Promotion::Queen | Promotion::Rook | Promotion::Bishop | Promotion::Knight => {
                let color = self.board[m.starting_idx as usize].unwrap().color;
                self.board[m.starting_idx as usize] =
                    Some(Piece::new(color, PieceName::Pawn, m.starting_idx));
            }
            Promotion::None => (),
        }
        if piece.piece_name == PieceName::King {
            match piece.color {
                Color::White => {
                    self.white_king_square = piece.current_square;
                    self.white_king_castle = true;
                    self.white_queen_castle = true;
                }
                Color::Black => {
                    self.black_king_square = piece.current_square;
                    self.black_queen_castle = true;
                    self.black_king_castle = true;
                }
            }
        }
        if piece.piece_name == PieceName::Rook {
            match m.starting_idx {
                0 => self.white_queen_castle = true,
                7 => self.white_king_castle = true,
                56 => self.black_queen_castle = true,
                63 => self.black_king_castle = true,
                _ => (),
            }
        }
        // If a rook is captured, castling is no longer possible
        if let Some(cap) = m.capture {
            if cap.piece_name == PieceName::Rook {
                match cap.current_square {
                    0 => self.white_queen_castle = true,
                    7 => self.white_king_castle = true,
                    56 => self.black_queen_castle = true,
                    63 => self.black_king_castle = true,
                    _ => (),
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

    #[allow(dead_code)]
    pub fn eq(&self, board: &Board) -> bool {
        for i in 0..63 {
            let s1 = self.board[i];
            let s2 = board.board[i];
            if s1.is_none() && s2.is_some() || s1.is_some() && s2.is_none() {
                return false;
            }
            if s1.is_none() && s2.is_none() {
                continue;
            }
            let s1 = s1.unwrap();
            let s2 = s2.unwrap();
            if s1.piece_name != s2.piece_name
                || s1.color != s2.color
                || s1.current_square != s2.current_square
            {
                return false;
            }
        }
        true
    }

    #[allow(dead_code)]
    pub fn position_eval(&self) -> i32 {
        let mut white = 0;
        let mut black = 0;
        for square in self.board {
            match square {
                None => continue,
                Some(piece) => match piece.color {
                    Color::White => {
                        white += piece.value();
                    }
                    Color::Black => {
                        black += piece.value();
                    }
                },
            }
        }
        if self.to_move == Color::White {
            white - black
        } else {
            black - white
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

#[cfg(test)]
mod board_tests {
    use crate::{fen, moves::generate_all_moves};

    #[test]
    fn test_undo() {
        let mut board = fen::build_board("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - -");
        //let mut board = fen::build_board("8/8/8/8/8/4p1p1/5P2/8 w - - 0 1");
        //let mut board = fen::build_board("8/8/8/8/8/4r1r1/5P2/8 w - - 0 1");
        println!("{}", board);
        let cloned_board = board;
        assert!(board.eq(&cloned_board));
        let moves = generate_all_moves(&board);
        for m in moves {
            board.make_move(&m);
            println!("{}", board);
            let second_moves = generate_all_moves(&board);
            for s_m in second_moves {
                board.make_move(&s_m);
                println!("{}", board);
                board.unmake_move(&s_m);
            }
            board.unmake_move(&m);
            println!("----------------------------");
        }
        assert!(board.eq(&cloned_board));
    }
}
