use core::fmt;
use std::fmt::Display;

use crate::{
    board::board::Board,
    moves::moves::Direction::*,
    types::{pieces::PieceName, square::Square},
};

use strum_macros::EnumIter;

use MoveType::*;
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MoveType {
    Normal = 0b0000,

    QueenPromotion = 0b0001,
    RookPromotion = 0b0010,
    BishopPromotion = 0b0011,
    KnightPromotion = 0b0100,

    DoublePush = 0b0110,

    CastleMove = 0b0111,

    Capture = 0b1000,
    EnPassant = 0b1101,

    CaptureQueenPromotion = 0b1001,
    CaptureRookPromotion = 0b1010,
    CaptureBishopPromotion = 0b1011,
    CaptureKnightPromotion = 0b1100,
}

/// Cardinal directions from the point of view of white side
#[derive(EnumIter, Copy, Clone, Debug, PartialEq, Eq)]
pub enum Direction {
    North = 8,
    NorthWest = 7,
    West = -1,
    SouthWest = -9,
    South = -8,
    SouthEast = -7,
    East = 1,
    NorthEast = 9,
}

impl Direction {
    /// Returns the opposite direction of the given direction
    pub fn opp(self) -> Self {
        match self {
            North => South,
            NorthWest => SouthEast,
            West => East,
            SouthWest => NorthEast,
            South => North,
            SouthEast => NorthWest,
            East => West,
            NorthEast => SouthWest,
        }
    }

    pub fn to_xy(self) -> (i32, i32) {
        match self {
            North => (0, 1),
            NorthWest => (-1, 1),
            West => (-1, 0),
            SouthWest => (-1, -1),
            South => (0, -1),
            SouthEast => (1, -1),
            East => (1, 0),
            NorthEast => (1, 1),
        }
    }
}

/// A move needs 16 bits to be stored
///
/// bit  0-5: origin square (from 0 to 63)
/// bit  6-11: destination square (from 0 to 63)
/// bit 12-15: special move flag: normal move(0), promotion (1), en passant (2), castling (3)
/// bit 16-19: piece moving - useful in continuation history
/// NOTE: en passant bit is set only when a pawn can be captured
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Move(pub u32);

impl Move {
    pub const NULL: Move = Move(0);
    pub fn new(origin: Square, destination: Square, move_type: MoveType, piece_moving: PieceName) -> Self {
        let piece = piece_moving.idx() as u32;
        let m = origin.0 | (destination.0 << 6) | ((move_type as u32) << 12) | (piece << 16);
        Move(m)
    }

    pub fn is_capture(self, board: &Board) -> bool {
        board.occupancies().occupied(self.dest_square())
    }

    pub fn is_castle(self) -> bool {
        let castle_flag = (self.0 >> 12) & 0b1111;
        castle_flag == MoveType::CastleMove as u32
    }

    pub fn piece_moving(self) -> PieceName {
        let piece_flag = (self.0 >> 16) & 0b1111;
        PieceName::from(piece_flag as usize)
    }

    fn flag(self) -> MoveType {
        unsafe { std::mem::transmute((self.0 >> 12) as u8 & 0b1111) }
    }

    pub fn is_en_passant(self) -> bool {
        let en_passant_flag = (self.0 >> 12) & 0b1111;
        en_passant_flag == MoveType::EnPassant as u32
    }

    pub fn promotion(self) -> Option<PieceName> {
        match self.flag() {
            QueenPromotion | CaptureQueenPromotion => Some(PieceName::Queen),
            RookPromotion | CaptureRookPromotion => Some(PieceName::Rook),
            BishopPromotion | CaptureBishopPromotion => Some(PieceName::Bishop),
            KnightPromotion | CaptureKnightPromotion => Some(PieceName::Knight),
            _ => None,
        }
    }

    pub fn origin_square(self) -> Square {
        Square(self.0 & 0b111111)
    }

    pub fn dest_square(self) -> Square {
        Square(self.0 >> 6 & 0b111111)
    }

    pub fn as_u16(self) -> u16 {
        self.0 as u16
    }

    /// To Short Algebraic Notation
    pub fn to_san(self) -> String {
        let mut str = String::new();
        let arr = ["a", "b", "c", "d", "e", "f", "g", "h"];
        let origin_number = self.origin_square().rank() + 1;
        let origin_letter = self.origin_square().file();
        let end_number = self.dest_square().rank() + 1;
        let end_letter = self.dest_square().file();
        str += arr[origin_letter as usize];
        str += &origin_number.to_string();
        str += arr[end_letter as usize];
        str += &end_number.to_string();
        match self.promotion() {
            Some(PieceName::Queen) => str += "q",
            Some(PieceName::Rook) => str += "r",
            Some(PieceName::Bishop) => str += "b",
            Some(PieceName::Knight) => str += "n",
            _ => (),
        }
        str
    }

    pub fn castle_type(self) -> Castle {
        debug_assert!(self.is_castle());
        if self.dest_square().dist(self.origin_square()) != 2 {
            Castle::None
        } else if self.dest_square() == Square(2) {
            Castle::WhiteQueen
        } else if self.dest_square() == Square(6) {
            Castle::WhiteKing
        } else if self.dest_square() == Square(58) {
            Castle::BlackQueen
        } else if self.dest_square() == Square(62) {
            Castle::BlackKing
        } else {
            unreachable!()
        }
    }
}

/// Method converts a san move provided by UCI framework into a Move struct
pub fn from_san(str: &str, board: &Board) -> Move {
    let vec: Vec<char> = str.chars().collect();

    // Using base 20 allows program to convert letters directly to numbers instead of matching
    // against letters or some other workaround
    let start_column = vec[0].to_digit(20).unwrap() - 10;
    let start_row = (vec[1].to_digit(10).unwrap() - 1) * 8;
    let origin_sq = Square(start_row + start_column);

    let end_column = vec[2].to_digit(20).unwrap() - 10;
    let end_row = (vec[3].to_digit(10).unwrap() - 1) * 8;
    let dest_sq = Square(end_row + end_column);

    let promotion = if vec.len() > 4 {
        match vec[4] {
            'q' => Some(PieceName::Queen),
            'r' => Some(PieceName::Rook),
            'b' => Some(PieceName::Bishop),
            'n' => Some(PieceName::Knight),
            _ => panic!(),
        }
    } else {
        None
    };
    let piece_moving = board.piece_at(origin_sq).expect("There should be a piece here...");
    let captured = board.piece_at(dest_sq);
    let castle = match piece_moving {
        PieceName::King => {
            if origin_sq.dist(dest_sq) != 2 {
                Castle::None
            } else if dest_sq == Square(2) {
                Castle::WhiteQueen
            } else if dest_sq == Square(6) {
                Castle::WhiteKing
            } else if dest_sq == Square(58) {
                Castle::BlackQueen
            } else if dest_sq == Square(62) {
                Castle::BlackKing
            } else {
                unreachable!()
            }
        }
        _ => Castle::None,
    };
    let castle = castle != Castle::None;
    let en_passant = { piece_moving == PieceName::Pawn && captured.is_none() && start_column != end_column };
    let move_type = {
        if castle {
            CastleMove
        } else if en_passant {
            EnPassant
        } else if promotion.is_some() {
            match promotion {
                Some(p) => match p {
                    PieceName::Knight => KnightPromotion,
                    PieceName::Bishop => BishopPromotion,
                    PieceName::Rook => RookPromotion,
                    PieceName::Queen => QueenPromotion,
                    _ => Normal,
                },
                None => Normal,
            }
        } else {
            Normal
        }
    };
    Move::new(origin_sq, dest_sq, move_type, piece_moving)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Castle {
    WhiteKing = 1,
    WhiteQueen = 2,
    BlackKing = 4,
    BlackQueen = 8,
    None,
}

#[rustfmt::skip]
pub const CASTLING_RIGHTS: [u32; 64] = [
    13, 15, 15, 15, 12, 15, 15, 14,
    15, 15, 15, 15, 15, 15, 15, 15,
    15, 15, 15, 15, 15, 15, 15, 15,
    15, 15, 15, 15, 15, 15, 15, 15,
    15, 15, 15, 15, 15, 15, 15, 15,
    15, 15, 15, 15, 15, 15, 15, 15,
    15, 15, 15, 15, 15, 15, 15, 15,
    7, 15, 15, 15, 3, 15, 15, 11,
];

impl Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut str = String::new();
        str += "Start: ";
        str += &self.origin_square().to_string();
        str += " End: ";
        str += &self.dest_square().to_string();
        str += " Castle: ";
        str += &self.is_castle().to_string();
        str += " Promotion: ";
        match self.promotion() {
            Some(PieceName::Queen) => str += "Queen ",
            Some(PieceName::Rook) => str += "Rook ",
            Some(PieceName::Bishop) => str += "Bishop ",
            Some(PieceName::Knight) => str += "Knight ",
            _ => str += "None ",
        }
        str += " En Passant: ";
        str += &self.is_en_passant().to_string();
        str += "  ";
        str += &self.to_san();
        write!(f, "{}", str)
    }
}

#[cfg(test)]
mod move_test {
    use super::*;

    #[test]
    fn test_move_creation() {
        let normal_move = Move::new(Square(10), Square(20), MoveType::Normal, PieceName::Pawn);
        assert_eq!(normal_move.origin_square(), Square(10));
        assert_eq!(normal_move.dest_square(), Square(20));
        assert!(!normal_move.is_castle());
        assert!(!normal_move.is_en_passant());
        assert_eq!(normal_move.promotion(), None);

        let promotion_move = Move::new(Square(15), Square(25), QueenPromotion, PieceName::Pawn);
        assert_eq!(promotion_move.origin_square(), Square(15));
        assert_eq!(promotion_move.dest_square(), Square(25));
        assert!(!promotion_move.is_castle());
        assert!(!promotion_move.is_en_passant());
        assert_eq!(promotion_move.promotion(), Some(PieceName::Queen));

        let castle_move = Move::new(Square(4), Square(2), CastleMove, PieceName::King);
        assert_eq!(castle_move.origin_square(), Square(4));
        assert_eq!(castle_move.dest_square(), Square(2));
        assert!(castle_move.is_castle());
        assert!(!castle_move.is_en_passant());
        assert_eq!(castle_move.promotion(), None);

        let en_passant_move = Move::new(Square(7), Square(5), MoveType::EnPassant, PieceName::King);
        assert_eq!(en_passant_move.origin_square(), Square(7));
        assert_eq!(en_passant_move.dest_square(), Square(5));
        assert!(!en_passant_move.is_castle());
        assert!(en_passant_move.is_en_passant());
        assert_eq!(en_passant_move.promotion(), None);
    }

    #[test]
    fn test_promotion_conversion() {
        let knight_promotion = Move::new(Square(0), Square(7), KnightPromotion, PieceName::Pawn);
        assert_eq!(knight_promotion.promotion(), Some(PieceName::Knight));

        let bishop_promotion = Move::new(Square(15), Square(23), BishopPromotion, PieceName::Pawn);
        assert_eq!(bishop_promotion.promotion(), Some(PieceName::Bishop));

        let rook_promotion = Move::new(Square(28), Square(31), RookPromotion, PieceName::Pawn);
        assert_eq!(rook_promotion.promotion(), Some(PieceName::Rook));

        let queen_promotion = Move::new(Square(62), Square(61), CaptureQueenPromotion, PieceName::Pawn);
        assert_eq!(queen_promotion.promotion(), Some(PieceName::Queen));
    }
}
