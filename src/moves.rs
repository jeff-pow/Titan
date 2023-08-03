use core::fmt;
use std::fmt::Display;

use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::{board::Board, pieces::Color, pieces::Piece, pieces::PieceName};

#[derive(Clone, Copy, Debug)]
pub struct Move {
    pub starting_idx: i8,
    pub end_idx: i8,
    pub castle: Castle,
    pub promotion: Promotion,
    pub piece_moving: PieceName,
    pub capture: Option<PieceName>,
    pub en_passant: EnPassant,
    pub move_type: MoveType,
}

#[derive(Clone, Copy, Debug)]
pub enum MoveType {
    Capture,
    Quiet,
}

impl Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut str = String::new();
        str += "Start: ";
        str += &self.starting_idx.to_string();
        str += " End: ";
        str += &self.end_idx.to_string();
        str += " Castle: ";
        match self.castle {
            Castle::None => str += "No Castle ",
            Castle::WhiteKingCastle => str += "White King castle ",
            Castle::WhiteQueenCastle => str += "white queen castle ",
            Castle::BlackKingCastle => str += "black king castle ",
            Castle::BlackQueenCastle => str += "black queen castle ",
        }
        str += " Promotion: ";
        match self.promotion {
            Promotion::Queen => str += "Queen ",
            Promotion::Rook => str += "Rook ",
            Promotion::Bishop => str += "Bishop ",
            Promotion::Knight => str += "Knight ",
            Promotion::None => str += "None ",
        }
        match self.capture {
            None => {
                str += " Nothing Captured ";
            }
            Some(piece_name) => match piece_name {
                PieceName::King => str += " Captured a King  ",
                PieceName::Queen => str += " Captured a Queen ",
                PieceName::Rook => str += " Captured a Rook ",
                PieceName::Bishop => str += " Captured a Bishop ",
                PieceName::Knight => str += " Captured a Knight ",
                PieceName::Pawn => str += " Captured a Pawn ",
            },
        }
        match self.piece_moving {
            PieceName::King => str += " King moving ",
            PieceName::Queen => str += " Queen moving ",
            PieceName::Bishop => str += " Bishop moving ",
            PieceName::Rook => str += " Rook moving ",
            PieceName::Knight => str += " Knight moving ",
            PieceName::Pawn => str += " Pawn moving ",
        }
        str += &self.to_lan();
        write!(f, "{}", str)
    }
}

impl Move {
    /// To Long Algebraic Notation
    pub fn to_lan(self) -> String {
        let mut str = String::new();
        let arr = ["a", "b", "c", "d", "e", "f", "g", "h"];
        let y_origin = self.starting_idx / 8 + 1;
        let x_origin = self.starting_idx % 8;
        let y_end = self.end_idx / 8 + 1;
        let x_end = self.end_idx % 8;
        str += arr[x_origin as usize];
        str += &y_origin.to_string();
        str += arr[x_end as usize];
        str += &y_end.to_string();
        match self.promotion {
            Promotion::Queen => str += "q",
            Promotion::Rook => str += "r",
            Promotion::Bishop => str += "b",
            Promotion::Knight => str += "n",
            Promotion::None => (),
        }
        str
    }

    /// Constructor for new moves - Mostly a placeholder for initializing variables that will
    /// certainly be changed at some other point during the runtime of the function
    pub fn invalid() -> Self {
        Move {
            starting_idx: -1,
            end_idx: -1,
            castle: Castle::None,
            promotion: Promotion::None,
            piece_moving: PieceName::King,
            capture: None,
            en_passant: EnPassant::None,
            move_type: MoveType::Quiet,
        }
    }
}

/// Method converts a lan move provided by UCI framework into a Move struct
pub fn from_lan(str: &str, board: &Board) -> Move {
    let vec: Vec<char> = str.chars().collect();

    // Using base 20 allows program to convert letters directly to numbers instead of matching
    // against letters or some other workaround
    let start_column = vec[0].to_digit(20).unwrap() - 10;
    let start_row = (vec[1].to_digit(10).unwrap() - 1) * 8;
    let starting_idx = (start_row + start_column) as i8;

    let end_column = vec[2].to_digit(20).unwrap() - 10;
    let end_row = (vec[3].to_digit(10).unwrap() - 1) * 8;
    let end_idx = (end_row + end_column) as i8;

    let mut promotion = Promotion::None;
    if vec.len() > 4 {
        promotion = match vec[4] {
            'q' => Promotion::Queen,
            'r' => Promotion::Rook,
            'b' => Promotion::Bishop,
            'n' => Promotion::Knight,
            _ => panic!(),
        };
    }
    let piece_moving = board
        .piece_on_square(starting_idx as usize)
        .expect("There should be a piece here...");
    let castle = match piece_moving {
        PieceName::King => {
            if i8::abs_diff(starting_idx, end_idx) != 2 {
                Castle::None
            } else if end_idx == 2 {
                Castle::WhiteQueenCastle
            } else if end_idx == 6 {
                Castle::WhiteKingCastle
            } else if end_idx == 58 {
                Castle::BlackQueenCastle
            } else if end_idx == 62 {
                Castle::BlackKingCastle
            } else {
                unreachable!()
            }
        }
        _ => Castle::None,
    };
    let capture = board.piece_on_square(end_idx as usize);
    let move_type = if capture.is_some() {
        MoveType::Capture
    } else {
        MoveType::Quiet
    };
    Move {
        starting_idx,
        end_idx,
        castle,
        promotion,
        piece_moving,
        capture,
        en_passant: EnPassant::None,
        move_type,
    }
}

#[derive(Clone, Copy, Debug, EnumIter, PartialEq)]
pub enum Promotion {
    Queen,
    Rook,
    Bishop,
    Knight,
    None,
}

#[derive(Clone, Copy, Debug, EnumIter, PartialEq)]
pub enum EnPassant {
    NW,
    NE,
    SW,
    SE,
    None,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Castle {
    None,
    WhiteKingCastle,
    WhiteQueenCastle,
    BlackKingCastle,
    BlackQueenCastle,
}

/// Cardinal directions from the point of view of white side
#[derive(EnumIter, Copy, Clone, Debug, PartialEq, Eq)]
#[repr(i8)]
enum Direction {
    North = 8,
    NorthWest = 7,
    West = -1,
    SouthWest = -9,
    South = -8,
    SouthEast = -7,
    East = 1,
    NorthEast = 9,
}

#[inline]
pub fn coordinates(idx: usize) -> (usize, usize) {
    (idx / 8, idx % 8)
}

pub fn in_check(board: &Board, color: Color) -> bool {
    todo!()
}

pub fn generate_all_moves(board: &Board) -> Vec<Move> {
    todo!()
}
