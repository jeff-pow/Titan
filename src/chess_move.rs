use core::fmt;
use std::{
    fmt::Display,
    num::{NonZero, NonZeroU32},
};

use crate::{
    board::Board,
    chess_move::Direction::{East, North, NorthEast, NorthWest, South, SouthEast, SouthWest, West},
    types::{
        bitboard::Bitboard,
        pieces::{Piece, PieceName},
        square::Square,
    },
};

use MoveType::{
    BishopPromotion, CastleMove, DoublePush, EnPassant, KnightPromotion, Normal, QueenPromotion, RookPromotion,
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MoveType {
    Normal = 0,

    QueenPromotion = 1,
    RookPromotion = 2,
    BishopPromotion = 3,
    KnightPromotion = 4,

    DoublePush = 5,

    CastleMove = 6,

    EnPassant = 7,
}

const _: () = assert!(std::mem::size_of::<Move>() == std::mem::size_of::<Option<Move>>());

/// A move needs 16 bits to be stored, but extra information is stored in more bits
///
/// bit  0-5: origin square (from 0 to 63)
/// bit  6-11: destination square (from 0 to 63)
/// bit 12-15: special move flag: normal move(0), promotion (1), en passant (2), castling (3)
/// bit 16-19: piece moving - useful in continuation history
/// NOTE: en passant bit is set only when a pawn can be captured
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Move(pub NonZeroU32);

impl Move {
    pub const NULL: Option<Self> = None;

    pub const fn new(origin: Square, destination: Square, move_type: MoveType) -> Self {
        let m = origin.0 | (destination.0 << 6) | ((move_type as u32) << 12);
        unsafe { Self(NonZero::new_unchecked(m)) }
    }

    pub fn is_capture(self, board: &Board) -> bool {
        board.occupancies().occupied(self.to())
    }

    pub fn is_castle(self) -> bool {
        self.flag() == CastleMove
    }

    pub fn flag(self) -> MoveType {
        unsafe { std::mem::transmute((self.0.get() >> 12) as u8 & 0b1111) }
    }

    pub fn is_en_passant(self) -> bool {
        self.flag() == EnPassant
    }

    pub fn promotion(self) -> Option<PieceName> {
        match self.flag() {
            QueenPromotion => Some(PieceName::Queen),
            RookPromotion => Some(PieceName::Rook),
            BishopPromotion => Some(PieceName::Bishop),
            KnightPromotion => Some(PieceName::Knight),
            _ => None,
        }
    }

    pub const fn from(self) -> Square {
        Square(self.0.get() & 0b11_1111)
    }

    pub const fn to(self) -> Square {
        Square(self.0.get() >> 6 & 0b11_1111)
    }

    pub fn is_tactical(self, board: &Board) -> bool {
        self.promotion().is_some() || self.is_en_passant() || board.occupancies().occupied(self.to())
    }

    pub const fn as_u16(self) -> u16 {
        self.0.get() as u16
    }

    /// To Short Algebraic Notation
    pub fn to_san(self) -> String {
        let mut str = String::new();
        let arr = ["a", "b", "c", "d", "e", "f", "g", "h"];
        let origin_number = self.from().rank() + 1;
        let origin_letter = self.from().file();
        let end_number = self.to().rank() + 1;
        let end_letter = self.to().file();
        str += arr[origin_letter as usize];
        str += &origin_number.to_string();
        str += arr[end_letter as usize];
        str += &end_number.to_string();
        if let Some(p) = self.promotion() {
            match p {
                PieceName::Queen => str += "q",
                PieceName::Rook => str += "r",
                PieceName::Bishop => str += "b",
                PieceName::Knight => str += "n",
                _ => (),
            }
        }
        str
    }

    pub fn castle_type(self) -> Castle {
        debug_assert!(self.is_castle());
        if self.to().dist(self.from()) != 2 {
            Castle::None
        } else if self.to() == Square(2) {
            Castle::WhiteQueen
        } else if self.to() == Square(6) {
            Castle::WhiteKing
        } else if self.to() == Square(58) {
            Castle::BlackQueen
        } else if self.to() == Square(62) {
            Castle::BlackKing
        } else {
            unreachable!()
        }
    }

    /// Method converts a san move provided by UCI framework into a Move struct
    pub fn from_san(str: &str, board: &Board) -> Self {
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
        let piece_moving = board.piece_at(origin_sq);
        assert!(piece_moving != Piece::None);
        let captured = board.piece_at(dest_sq);
        let castle = match piece_moving.name() {
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
        let en_passant =
            { piece_moving.name() == PieceName::Pawn && captured == Piece::None && start_column != end_column };
        let double_push = { piece_moving.name() == PieceName::Pawn && origin_sq.dist(dest_sq) == 2 };
        let move_type = {
            if castle {
                CastleMove
            } else if en_passant {
                EnPassant
            } else if promotion.is_some() {
                promotion.map_or(Normal, |p| match p {
                    PieceName::Knight => KnightPromotion,
                    PieceName::Bishop => BishopPromotion,
                    PieceName::Rook => RookPromotion,
                    PieceName::Queen => QueenPromotion,
                    _ => Normal,
                })
            } else if double_push {
                DoublePush
            } else {
                Normal
            }
        };
        Move::new(origin_sq, dest_sq, move_type)
    }
}

impl Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut str = String::new();
        str += &self.to_san();
        write!(f, "{str}")
    }
}

impl fmt::Debug for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut str = String::new();
        str += &self.to_san();
        write!(f, "{str}")
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Castle {
    WhiteKing = 1,
    WhiteQueen = 2,
    BlackKing = 4,
    BlackQueen = 8,
    None,
}

impl Castle {
    /// These squares may not be under attack for a castle to be valid
    pub(crate) const fn check_squares(self) -> Bitboard {
        match self {
            Self::WhiteKing => Bitboard(112),
            Self::WhiteQueen => Bitboard(28),
            Self::BlackKing => Bitboard(0x7000_0000_0000_0000),
            Self::BlackQueen => Bitboard(0x1C00_0000_0000_0000),
            Self::None => panic!("Invalid castle"),
        }
    }

    /// These squares must be unoccupied for a castle to be valid
    pub(crate) const fn empty_squares(self) -> Bitboard {
        match self {
            Self::WhiteKing => Bitboard(96),
            Self::WhiteQueen => Bitboard(14),
            Self::BlackKing => Bitboard(0x6000_0000_0000_0000),
            Self::BlackQueen => Bitboard(0xE00_0000_0000_0000),
            Self::None => panic!("Invalid castle"),
        }
    }

    pub(crate) const fn rook_to(self) -> Square {
        match self {
            Self::WhiteKing => Square::F1,
            Self::WhiteQueen => Square::D1,
            Self::BlackKing => Square::F8,
            Self::BlackQueen => Square::D8,
            Self::None => panic!("Invalid castle"),
        }
    }

    pub(crate) const fn rook_from(self) -> Square {
        match self {
            Self::WhiteKing => Square::H1,
            Self::WhiteQueen => Square::A1,
            Self::BlackKing => Square::H8,
            Self::BlackQueen => Square::A8,
            Self::None => panic!("Invalid castle"),
        }
    }
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
    7,  15, 15, 15,  3, 15, 15, 11,
];

/// Cardinal directions from the point of view of white side
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
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
    pub(crate) const fn opp(self) -> Self {
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
}

#[cfg(test)]
mod move_test {
    use super::*;

    #[test]
    fn test_move_creation() {
        let normal_move = Move::new(Square(10), Square(20), Normal);
        assert_eq!(normal_move.from(), Square(10));
        assert_eq!(normal_move.to(), Square(20));
        assert!(!normal_move.is_castle());
        assert!(!normal_move.is_en_passant());
        assert_eq!(normal_move.promotion(), None);

        let promotion_move = Move::new(Square(15), Square(25), QueenPromotion);
        assert_eq!(promotion_move.from(), Square(15));
        assert_eq!(promotion_move.to(), Square(25));
        assert!(!promotion_move.is_castle());
        assert!(!promotion_move.is_en_passant());
        assert_eq!(promotion_move.promotion(), Some(PieceName::Queen));

        let castle_move = Move::new(Square(4), Square(2), CastleMove);
        assert_eq!(castle_move.from(), Square(4));
        assert_eq!(castle_move.to(), Square(2));
        assert!(castle_move.is_castle());
        assert!(!castle_move.is_en_passant());
        assert_eq!(castle_move.promotion(), None);

        let en_passant_move = Move::new(Square(7), Square(5), EnPassant);
        assert_eq!(en_passant_move.from(), Square(7));
        assert_eq!(en_passant_move.to(), Square(5));
        assert!(!en_passant_move.is_castle());
        assert!(en_passant_move.is_en_passant());
        assert_eq!(en_passant_move.promotion(), None);
    }

    #[test]
    fn test_promotion_conversion() {
        let knight_promotion = Move::new(Square(0), Square(7), KnightPromotion);
        assert_eq!(knight_promotion.promotion(), Some(PieceName::Knight));

        let bishop_promotion = Move::new(Square(15), Square(23), BishopPromotion);
        assert_eq!(bishop_promotion.promotion(), Some(PieceName::Bishop));

        let rook_promotion = Move::new(Square(28), Square(31), RookPromotion);
        assert_eq!(rook_promotion.promotion(), Some(PieceName::Rook));

        let queen_promotion = Move::new(Square(62), Square(61), QueenPromotion);
        assert_eq!(queen_promotion.promotion(), Some(PieceName::Queen));
    }
}
