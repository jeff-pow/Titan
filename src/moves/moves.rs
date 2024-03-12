use core::fmt;
use std::fmt::Display;

use crate::{
    board::board::Board,
    moves::moves::Direction::{
        East, North, NorthEast, NorthWest, South, SouthEast, SouthWest, West,
    },
    types::{
        bitboard::Bitboard,
        pieces::{Piece, PieceName},
        square::Square,
    },
};

use MoveType::{
    BishopPromotion, CastleMove, DoublePush, EnPassant, KnightPromotion, Normal, QueenPromotion,
    RookPromotion,
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

/// A move needs 16 bits to be stored, but extra information is stored in more bits
///
/// bit  0-5: origin square (from 0 to 63)
/// bit  6-11: destination square (from 0 to 63)
/// bit 12-15: special move flag: normal move(0), promotion (1), en passant (2), castling (3)
/// bit 16-19: piece moving - useful in continuation history
/// NOTE: en passant bit is set only when a pawn can be captured
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Move(pub u32);

impl Move {
    pub const NULL: Self = Self(0);

    pub const fn new(
        origin: Square,
        destination: Square,
        move_type: MoveType,
        piece_moving: Piece,
    ) -> Self {
        let m = origin.0
            | (destination.0 << 6)
            | ((move_type as u32) << 12)
            | ((piece_moving as u32) << 16);
        Self(m)
    }

    pub fn is_capture(self, board: &Board) -> bool {
        board.occupancies().occupied(self.to())
    }

    pub fn is_castle(self) -> bool {
        self.flag() == MoveType::CastleMove
    }

    pub fn piece_moving(self) -> Piece {
        let piece_flag = (self.0 >> 16) & 0b1111;
        Piece::from_u32(piece_flag)
    }

    pub fn flag(self) -> MoveType {
        unsafe { std::mem::transmute((self.0 >> 12) as u8 & 0b1111) }
    }

    pub fn is_en_passant(self) -> bool {
        self.flag() == MoveType::EnPassant
    }

    pub fn promotion(self) -> Option<Piece> {
        match self.flag() {
            QueenPromotion => Some(Piece::new(PieceName::Queen, self.piece_moving().color())),
            RookPromotion => Some(Piece::new(PieceName::Rook, self.piece_moving().color())),
            BishopPromotion => Some(Piece::new(PieceName::Bishop, self.piece_moving().color())),
            KnightPromotion => Some(Piece::new(PieceName::Knight, self.piece_moving().color())),
            _ => None,
        }
    }

    pub const fn from(self) -> Square {
        Square(self.0 & 0b11_1111)
    }

    pub const fn to(self) -> Square {
        Square(self.0 >> 6 & 0b11_1111)
    }

    pub fn is_tactical(self, board: &Board) -> bool {
        (self.promotion().is_some()
            || self.is_en_passant()
            || board.occupancies().occupied(self.to()))
            && self != Self::NULL
    }

    pub const fn as_u16(self) -> u16 {
        self.0 as u16
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
            match p.name() {
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
    let en_passant = {
        piece_moving.name() == PieceName::Pawn
            && captured == Piece::None
            && start_column != end_column
    };
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
    Move::new(origin_sq, dest_sq, move_type, piece_moving)
}

impl Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut str = String::new();
        str += "Start: ";
        str += &self.from().to_string();
        str += " End: ";
        str += &self.to().to_string();
        str += " Castle: ";
        str += &self.is_castle().to_string();
        str += " Promotion: ";
        match self.promotion().map_or(PieceName::Pawn, Piece::name) {
            PieceName::Queen => str += "Queen ",
            PieceName::Rook => str += "Rook ",
            PieceName::Bishop => str += "Bishop ",
            PieceName::Knight => str += "Knight ",
            _ => str += "None ",
        }
        str += " En Passant: ";
        str += &self.is_en_passant().to_string();
        str += "  ";
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

    pub(crate) const fn rook_dest(self) -> Square {
        match self {
            Self::WhiteKing => Square(5),
            Self::WhiteQueen => Square(3),
            Self::BlackKing => Square(61),
            Self::BlackQueen => Square(59),
            Self::None => panic!("Invalid castle"),
        }
    }

    pub(crate) const fn rook_src(self) -> Square {
        match self {
            Self::WhiteKing => Square(7),
            Self::WhiteQueen => Square(0),
            Self::BlackKing => Square(63),
            Self::BlackQueen => Square(56),
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
        let normal_move = Move::new(Square(10), Square(20), MoveType::Normal, Piece::WhitePawn);
        assert_eq!(normal_move.from(), Square(10));
        assert_eq!(normal_move.to(), Square(20));
        assert!(!normal_move.is_castle());
        assert!(!normal_move.is_en_passant());
        assert_eq!(normal_move.promotion(), None);
        assert_eq!(normal_move.piece_moving(), Piece::WhitePawn);

        let promotion_move = Move::new(Square(15), Square(25), QueenPromotion, Piece::WhitePawn);
        assert_eq!(promotion_move.from(), Square(15));
        assert_eq!(promotion_move.to(), Square(25));
        assert!(!promotion_move.is_castle());
        assert!(!promotion_move.is_en_passant());
        assert_eq!(promotion_move.promotion(), Some(Piece::WhiteQueen));
        assert_eq!(promotion_move.piece_moving(), Piece::WhitePawn);

        let castle_move = Move::new(Square(4), Square(2), CastleMove, Piece::WhiteKing);
        assert_eq!(castle_move.from(), Square(4));
        assert_eq!(castle_move.to(), Square(2));
        assert!(castle_move.is_castle());
        assert!(!castle_move.is_en_passant());
        assert_eq!(castle_move.promotion(), None);
        assert_eq!(castle_move.piece_moving(), Piece::WhiteKing);

        let en_passant_move =
            Move::new(Square(7), Square(5), MoveType::EnPassant, Piece::BlackPawn);
        assert_eq!(en_passant_move.from(), Square(7));
        assert_eq!(en_passant_move.to(), Square(5));
        assert!(!en_passant_move.is_castle());
        assert!(en_passant_move.is_en_passant());
        assert_eq!(en_passant_move.promotion(), None);
        assert_eq!(en_passant_move.piece_moving(), Piece::BlackPawn);
    }

    #[test]
    fn test_promotion_conversion() {
        let knight_promotion = Move::new(Square(0), Square(7), KnightPromotion, Piece::WhitePawn);
        assert_eq!(knight_promotion.promotion(), Some(Piece::WhiteKnight));
        assert_eq!(knight_promotion.piece_moving(), Piece::WhitePawn);

        let bishop_promotion = Move::new(Square(15), Square(23), BishopPromotion, Piece::WhitePawn);
        assert_eq!(bishop_promotion.promotion(), Some(Piece::WhiteBishop));
        assert_eq!(bishop_promotion.piece_moving(), Piece::WhitePawn);

        let rook_promotion = Move::new(Square(28), Square(31), RookPromotion, Piece::BlackPawn);
        assert_eq!(rook_promotion.promotion(), Some(Piece::BlackRook));
        assert_eq!(rook_promotion.piece_moving(), Piece::BlackPawn);

        let queen_promotion = Move::new(Square(62), Square(61), QueenPromotion, Piece::BlackPawn);
        assert_eq!(queen_promotion.promotion(), Some(Piece::BlackQueen));
        assert_eq!(queen_promotion.piece_moving(), Piece::BlackPawn);
    }
}
