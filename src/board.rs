use core::fmt;

use crate::attack_boards::{gen_pawn_attack_board, king_attacks, knight_attacks};
use crate::bitboard::Bitboard;
use crate::moves::Direction;
use crate::pleco_magics::{bishop_attacks, rook_attacks};
use crate::square::Square;
use crate::{
    moves::{Castle, Direction::*, Move, Promotion},
    pieces::Color,
    pieces::{opposite_color, PieceName, NUM_PIECES},
};

#[repr(C)]
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Board {
    pub board: [[Bitboard; NUM_PIECES]; 2],
    pub to_move: Color,
    pub black_king_castle: bool,
    pub black_queen_castle: bool,
    pub white_king_castle: bool,
    pub white_queen_castle: bool,
    pub en_passant_square: Square,
    pub black_king_square: Square,
    pub white_king_square: Square,
    pub num_moves: i32,
    pub zobrist_hash: u64,
}

impl Board {
    pub fn new() -> Self {
        Board {
            board: [[Bitboard::empty(); 6]; 2],
            black_king_castle: false,
            black_queen_castle: false,
            white_king_castle: false,
            white_queen_castle: false,
            to_move: Color::White,
            en_passant_square: Square::INVALID,
            white_king_square: Square::INVALID,
            black_king_square: Square::INVALID,
            num_moves: 0,
            zobrist_hash: 0,
        }
    }

    pub fn can_en_passant(&self) -> bool {
        self.en_passant_square != Square::INVALID
    }

    #[inline]
    pub fn square_contains_piece(&self, piece_type: PieceName, color: Color, sq: Square) -> bool {
        self.board[color as usize][piece_type as usize].square_is_occupied(sq)
    }

    #[inline]
    pub fn color_occupancies(&self, color: Color) -> Bitboard {
        // It's odd to me that xor and bitwise or both seem to work here, only one piece should
        // be on a square at a time though so ¯\_(ツ)_/¯
        self.board[color as usize]
            .iter()
            .fold(Bitboard::empty(), |a, b| a ^ *b)
    }

    #[inline]
    pub fn occupancies(&self) -> Bitboard {
        self.board
            .iter()
            .flatten()
            .fold(Bitboard::empty(), |a, b| a ^ *b)
    }

    #[inline]
    pub fn color_on_square(&self, sq: Square) -> Option<Color> {
        let white_occ = self.color_occupancies(Color::White);
        let black_occ = self.color_occupancies(Color::Black);
        if white_occ & sq.bitboard() != Bitboard::empty() {
            return Some(Color::White);
        }
        if black_occ & sq.bitboard() != Bitboard::empty() {
            return Some(Color::Black);
        }
        None
    }

    #[inline]
    pub fn piece_on_square(&self, sq: Square) -> Option<PieceName> {
        let piece_names = [
            PieceName::King,
            PieceName::Queen,
            PieceName::Rook,
            PieceName::Bishop,
            PieceName::Knight,
            PieceName::Pawn,
        ];

        for color in &[Color::White, Color::Black] {
            for &piece_name in &piece_names {
                if self.square_contains_piece(piece_name, *color, sq) {
                    return Some(piece_name);
                }
            }
        }

        None
    }

    pub fn place_piece(&mut self, piece_type: PieceName, color: Color, sq: Square) {
        self.board[color as usize][piece_type as usize] |= sq.bitboard();
        if piece_type == PieceName::King {
            match color {
                Color::White => self.white_king_square = sq,
                Color::Black => self.black_king_square = sq,
            }
        }
    }

    fn remove_piece(&mut self, piece_type: PieceName, color: Color, sq: Square) {
        self.board[color as usize][piece_type as usize] &= !sq.bitboard();
    }

    pub fn side_in_check(&self, side: Color) -> bool {
        let king_square = match side {
            Color::White => self.white_king_square,
            Color::Black => self.black_king_square,
        };
        self.square_under_attack(opposite_color(side), king_square)
    }

    pub fn square_under_attack(&self, attacker: Color, sq: Square) -> bool {
        let attacker_occupancy = self.board[attacker as usize];
        let occupancy = self.occupancies();
        let pawn_attacks = gen_pawn_attack_board(self);
        let knight_attacks = knight_attacks(sq);
        let bishop_attacks = Bitboard(bishop_attacks(occupancy.0, sq.0));
        let rook_attacks = Bitboard(rook_attacks(occupancy.0, sq.0));
        let queen_attacks = rook_attacks | bishop_attacks;
        let king_attacks = king_attacks(sq);

        (king_attacks & attacker_occupancy[PieceName::King as usize] > Bitboard::empty())
            || (queen_attacks & attacker_occupancy[PieceName::Queen as usize] > Bitboard::empty())
            || (rook_attacks & attacker_occupancy[PieceName::Rook as usize] > Bitboard::empty())
            || (bishop_attacks & attacker_occupancy[PieceName::Bishop as usize] > Bitboard::empty())
            || (knight_attacks & attacker_occupancy[PieceName::Knight as usize] > Bitboard::empty())
            || (pawn_attacks & attacker_occupancy[PieceName::Pawn as usize] > Bitboard::empty())
    }

    /// Function makes a move and modifies board state to reflect the move that just happened
    pub fn make_move(&mut self, m: &Move) {
        // Special case if the move is an en_passant
        if m.is_en_passant() {
            let end_idx = m.dest_square();
            match self.to_move {
                Color::White => {
                    self.remove_piece(
                        PieceName::Pawn,
                        opposite_color(self.to_move),
                        m.dest_square().shift(Direction::South).unwrap(),
                    );
                }
                Color::Black => {
                    self.remove_piece(
                        PieceName::Pawn,
                        opposite_color(self.to_move),
                        end_idx.shift(North).unwrap(),
                    );
                }
            }
        }

        let piece_moving = self
            .piece_on_square(m.origin_square())
            .expect("There should be a piece here");
        let capture = self.piece_on_square(m.dest_square());
        self.place_piece(piece_moving, self.to_move, m.dest_square());
        self.remove_piece(piece_moving, self.to_move, m.origin_square());

        // Move rooks if a castle move is applied
        if m.is_castle() {
            // Determine which kind of castle, no reason for this besides I haven't changed it to
            // something nicer yet...
            let castle = match piece_moving {
                PieceName::King => {
                    if m.dest_square().dist(m.origin_square()) != 2 {
                        Castle::None
                    } else if m.dest_square() == Square(2) {
                        Castle::WhiteQueenCastle
                    } else if m.dest_square() == Square(6) {
                        Castle::WhiteKingCastle
                    } else if m.dest_square() == Square(58) {
                        Castle::BlackQueenCastle
                    } else if m.dest_square() == Square(62) {
                        Castle::BlackKingCastle
                    } else {
                        unreachable!()
                    }
                }
                _ => panic!("There should not be a king moving if castle is marked true..."),
            };
            match castle {
                Castle::WhiteQueenCastle => {
                    self.place_piece(PieceName::Rook, self.to_move, Square(3));
                    self.remove_piece(PieceName::Rook, self.to_move, Square(0));
                    self.white_queen_castle = false;
                    self.white_king_castle = false;
                }
                Castle::WhiteKingCastle => {
                    self.place_piece(PieceName::Rook, self.to_move, Square(5));
                    self.remove_piece(PieceName::Rook, self.to_move, Square(7));
                    self.white_queen_castle = false;
                    self.white_king_castle = false;
                }
                Castle::BlackKingCastle => {
                    self.place_piece(PieceName::Rook, self.to_move, Square(61));
                    self.remove_piece(PieceName::Rook, self.to_move, Square(63));
                    self.black_queen_castle = false;
                    self.black_king_castle = false;
                }
                Castle::BlackQueenCastle => {
                    self.place_piece(PieceName::Rook, self.to_move, Square(59));
                    self.remove_piece(PieceName::Rook, self.to_move, Square(56));
                    self.black_queen_castle = false;
                    self.black_king_castle = false;
                }
                Castle::None => (),
            }
        }
        // If move is a promotion, a pawn is removed from the board and replaced with a higher
        // value piece
        match m.promotion() {
            Some(Promotion::Queen) => {
                self.place_piece(PieceName::Queen, self.to_move, m.dest_square());
            }
            Some(Promotion::Rook) => {
                self.place_piece(PieceName::Rook, self.to_move, m.dest_square());
            }
            Some(Promotion::Bishop) => {
                self.place_piece(PieceName::Bishop, self.to_move, m.dest_square());
            }
            Some(Promotion::Knight) => {
                self.place_piece(PieceName::Knight, self.to_move, m.dest_square());
            }
            None => (),
        }
        // Update board's king square if king moves
        if piece_moving == PieceName::King {
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
        if piece_moving == PieceName::Rook {
            match m.origin_square().0 {
                0 => self.white_queen_castle = false,
                7 => self.white_king_castle = false,
                56 => self.black_queen_castle = false,
                63 => self.black_king_castle = false,
                _ => (),
            }
        }
        // If a rook is captured, castling is no longer possible
        if let Some(cap) = capture {
            if cap == PieceName::Rook {
                match m.dest_square().0 {
                    0 => self.white_queen_castle = false,
                    7 => self.white_king_castle = false,
                    56 => self.black_queen_castle = false,
                    63 => self.black_king_castle = false,
                    _ => (),
                }
            }
        }
        // If the end index of a move is 16 squares from the start (and a pawn moved), an en passant is possible
        let mut en_passant = false;
        if piece_moving == PieceName::Pawn {
            match self.to_move {
                Color::White => {
                    if m.origin_square()
                        == m.dest_square().shift(South).unwrap().shift(South).unwrap()
                    {
                        en_passant = true;
                        self.en_passant_square = m.dest_square().shift(South).unwrap();
                    }
                }
                Color::Black => {
                    if m.dest_square().shift(North).unwrap().shift(North).unwrap()
                        == m.origin_square()
                    {
                        en_passant = true;
                        self.en_passant_square = m.origin_square().shift(South).unwrap();
                    }
                }
            }
        }
        // If en passant was not performed this move, the ability to do it on future moves goes away
        if !en_passant {
            self.en_passant_square = Square::INVALID;
        }
        // Update castling ability based on check
        match self.to_move {
            Color::White => {
                if self.side_in_check(Color::White) {
                    self.white_king_castle = false;
                    self.white_queen_castle = false;
                }
            }
            Color::Black => {
                if self.side_in_check(Color::Black) {
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
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut str = String::new();

        for row in (0..8).rev() {
            str.push_str(&(row + 1).to_string());
            str.push_str(" | ");

            for col in 0..8 {
                let idx = row * 8 + col;

                // Append piece characters for white pieces
                if self.square_contains_piece(PieceName::King, Color::White, Square(idx)) {
                    str += "K"
                } else if self.square_contains_piece(PieceName::Queen, Color::White, Square(idx)) {
                    str += "Q"
                } else if self.square_contains_piece(PieceName::Rook, Color::White, Square(idx)) {
                    str += "R"
                } else if self.square_contains_piece(PieceName::Bishop, Color::White, Square(idx)) {
                    str += "B"
                } else if self.square_contains_piece(PieceName::Knight, Color::White, Square(idx)) {
                    str += "N"
                } else if self.square_contains_piece(PieceName::Pawn, Color::White, Square(idx)) {
                    str += "P"
                } else if self.square_contains_piece(PieceName::King, Color::Black, Square(idx)) {
                    str += "k"
                } else if self.square_contains_piece(PieceName::Queen, Color::Black, Square(idx)) {
                    str += "q"
                } else if self.square_contains_piece(PieceName::Rook, Color::Black, Square(idx)) {
                    str += "r"
                } else if self.square_contains_piece(PieceName::Bishop, Color::Black, Square(idx)) {
                    str += "b"
                } else if self.square_contains_piece(PieceName::Knight, Color::Black, Square(idx)) {
                    str += "n"
                } else if self.square_contains_piece(PieceName::Pawn, Color::Black, Square(idx)) {
                    str += "p"
                } else {
                    str += "_"
                }

                str.push_str(" | ");
            }

            str.push('\n');
        }

        str.push_str("    a   b   c   d   e   f   g   h\n");

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
