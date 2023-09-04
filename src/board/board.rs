use core::fmt;
use std::sync::Arc;
use strum::IntoEnumIterator;

use crate::{
    eval::nnue::{NnueAccumulator, M},
    moves::{movegenerator::MoveGenerator, moves::Castle, moves::Direction::*, moves::Move, moves::Promotion},
    types::{
        bitboard::Bitboard,
        pieces::{Color, Piece, PieceName, NUM_PIECES},
        square::Square,
    },
};

use super::{history::History, zobrist::Zobrist};

#[derive(Clone)]
pub struct Board {
    pub bitboards: [[Bitboard; NUM_PIECES]; 2],
    pub color_occupancies: [Bitboard; 2],
    pub occupancies: Bitboard,
    pub array_board: [Option<Piece>; 64],
    pub material_val: [i32; 2],
    pub to_move: Color,
    pub black_king_castle: bool,
    pub black_queen_castle: bool,
    pub white_king_castle: bool,
    pub white_queen_castle: bool,
    pub en_passant_square: Square,
    pub black_king_square: Square,
    pub white_king_square: Square,
    pub num_moves: i32,
    pub half_moves: i32,
    pub zobrist_hash: u64,
    pub history: History,
    pub zobrist_consts: Arc<Zobrist>,
    pub mg: Arc<MoveGenerator>,
    pub accumulator: NnueAccumulator,
}

impl Default for Board {
    fn default() -> Self {
        Board {
            bitboards: [[Bitboard::EMPTY; 6]; 2],
            color_occupancies: [Bitboard::EMPTY; 2],
            occupancies: Bitboard::EMPTY,
            array_board: [None; 64],
            material_val: [0; 2],
            black_king_castle: false,
            black_queen_castle: false,
            white_king_castle: false,
            white_queen_castle: false,
            to_move: Color::White,
            en_passant_square: Square::INVALID,
            white_king_square: Square::INVALID,
            black_king_square: Square::INVALID,
            num_moves: 0,
            half_moves: 0,
            zobrist_hash: 0,
            history: History::default(),
            zobrist_consts: Arc::new(Zobrist::default()),
            mg: Arc::new(MoveGenerator::default()),
            accumulator: NnueAccumulator { v: [[0; M]; 2] },
        }
    }
}

impl Board {
    #[inline(always)]
    pub fn can_en_passant(&self) -> bool {
        self.en_passant_square != Square::INVALID
    }

    #[inline(always)]
    pub fn is_draw(&self) -> bool {
        check_for_3x_repetition(self) || self.half_moves >= 100
    }

    #[inline(always)]
    pub fn square_contains_piece(&self, piece_type: PieceName, color: Color, sq: Square) -> bool {
        self.bitboards[color as usize][piece_type as usize].square_is_occupied(sq)
    }

    #[inline(always)]
    pub fn gen_color_occupancies(&mut self, color: Color) {
        // It's interesting to me that xor and bitwise or both seem to work here, only one piece should
        // be on a square at a time though so ¯\_(ツ)_/¯
        self.color_occupancies[color as usize] = self.bitboards[color as usize]
            .iter()
            .fold(Bitboard::EMPTY, |a, b| a ^ *b)
    }

    #[inline(always)]
    pub fn gen_occupancies(&mut self) {
        self.occupancies = self.bitboards.iter().flatten().fold(Bitboard::EMPTY, |a, b| a ^ *b)
    }

    #[inline(always)]
    pub fn color_occupancies(&self, color: Color) -> Bitboard {
        self.color_occupancies[color as usize]
    }

    #[inline(always)]
    pub fn occupancies(&self) -> Bitboard {
        self.occupancies
    }

    #[inline(always)]
    pub fn color_at(&self, sq: Square) -> Option<Color> {
        self.array_board[sq.idx()].map(|piece| piece.color)
    }

    #[inline(always)]
    pub fn piece_at(&self, sq: Square) -> Option<PieceName> {
        self.array_board[sq.idx()].map(|piece| piece.name)
    }

    #[inline(always)]
    pub fn place_piece(&mut self, piece_type: PieceName, color: Color, sq: Square) {
        self.bitboards[color as usize][piece_type as usize] |= sq.bitboard();
        self.array_board[sq.idx()] = Some(Piece::new(piece_type, color));
        self.material_val[color as usize] += piece_type.value();
        self.occupancies |= sq.bitboard();
        self.color_occupancies[color as usize] |= sq.bitboard();
        if piece_type == PieceName::King {
            match color {
                Color::White => self.white_king_square = sq,
                Color::Black => self.black_king_square = sq,
            }
        }
    }

    #[inline(always)]
    fn remove_piece(&mut self, sq: Square) {
        if let Some(piece) = self.array_board[sq.idx()] {
            self.array_board[sq.idx()] = None;
            self.bitboards[piece.color as usize][piece.name as usize] &= !sq.bitboard();
            self.material_val[piece.color as usize] -= piece.value();
            self.occupancies &= !sq.bitboard();
            self.color_occupancies[piece.color as usize] &= !sq.bitboard();
        }
    }

    #[inline(always)]
    pub fn side_in_check(&self, side: Color) -> bool {
        let king_square = match side {
            Color::White => self.white_king_square,
            Color::Black => self.black_king_square,
        };
        self.square_under_attack(side.opp(), king_square)
    }

    #[inline(always)]
    // Function left with lots of variables to improve debugability...
    pub fn square_under_attack(&self, attacker: Color, sq: Square) -> bool {
        let attacker_occupancy = self.bitboards[attacker as usize];
        let occupancy = self.occupancies();
        let pawn_attacks = self.mg.pawn_attacks(sq, attacker.opp());
        let knight_attacks = self.mg.knight_attacks(sq);
        let bishop_attacks = self.mg.magics.bishop_attacks(occupancy, sq);
        let rook_attacks = self.mg.magics.rook_attacks(occupancy, sq);
        let queen_attacks = rook_attacks | bishop_attacks;
        let king_attacks = self.mg.king_attacks(sq);

        let king_attacks_overlap = king_attacks & attacker_occupancy[PieceName::King as usize];
        let queen_attacks_overlap = queen_attacks & attacker_occupancy[PieceName::Queen as usize];
        let rook_attacks_overlap = rook_attacks & attacker_occupancy[PieceName::Rook as usize];
        let bishop_attacks_overlap = bishop_attacks & attacker_occupancy[PieceName::Bishop as usize];
        let knight_attacks_overlap = knight_attacks & attacker_occupancy[PieceName::Knight as usize];
        let pawn_attacks_overlap = pawn_attacks & attacker_occupancy[PieceName::Pawn as usize];

        let is_king_attack = king_attacks_overlap != Bitboard::EMPTY;
        let is_queen_attack = queen_attacks_overlap != Bitboard::EMPTY;
        let is_rook_attack = rook_attacks_overlap != Bitboard::EMPTY;
        let is_bishop_attack = bishop_attacks_overlap != Bitboard::EMPTY;
        let is_knight_attack = knight_attacks_overlap != Bitboard::EMPTY;
        let is_pawn_attack = pawn_attacks_overlap != Bitboard::EMPTY;

        is_king_attack || is_queen_attack || is_rook_attack || is_bishop_attack || is_knight_attack || is_pawn_attack
    }

    pub fn add_to_history(&mut self) {
        self.history.push(self.zobrist_hash);
    }

    #[inline(always)]
    pub fn material_balance(&self) -> i32 {
        match self.to_move {
            Color::White => self.material_val[Color::White as usize] - self.material_val[Color::Black as usize],
            Color::Black => self.material_val[Color::Black as usize] - self.material_val[Color::White as usize],
        }
    }

    /// Function makes a move and modifies board state to reflect the move that just happened
    pub fn make_move(&mut self, m: &Move) {
        // Special case if the move is an en_passant
        if m.is_en_passant() {
            match self.to_move {
                Color::White => {
                    self.remove_piece(m.dest_square().shift(South));
                }
                Color::Black => {
                    self.remove_piece(m.dest_square().shift(North));
                }
            }
        }

        let piece_moving = self.piece_at(m.origin_square()).expect("There should be a piece here");
        let capture = self.piece_at(m.dest_square());
        self.remove_piece(m.dest_square());
        self.place_piece(piece_moving, self.to_move, m.dest_square());
        self.remove_piece(m.origin_square());

        // Move rooks if a castle move is applied
        if m.is_castle() {
            match m.castle_type() {
                Castle::WhiteKingCastle => {
                    self.place_piece(PieceName::Rook, self.to_move, Square(5));
                    self.remove_piece(Square(7));
                    self.white_queen_castle = false;
                    self.white_king_castle = false;
                }
                Castle::WhiteQueenCastle => {
                    self.place_piece(PieceName::Rook, self.to_move, Square(3));
                    self.remove_piece(Square(0));
                    self.white_queen_castle = false;
                    self.white_king_castle = false;
                }
                Castle::BlackKingCastle => {
                    self.place_piece(PieceName::Rook, self.to_move, Square(61));
                    self.remove_piece(Square(63));
                    self.black_queen_castle = false;
                    self.black_king_castle = false;
                }
                Castle::BlackQueenCastle => {
                    self.place_piece(PieceName::Rook, self.to_move, Square(59));
                    self.remove_piece(Square(56));
                    self.black_queen_castle = false;
                    self.black_king_castle = false;
                }
                Castle::None => (),
            }
        }
        // If move is a promotion, a pawn is removed from the board and replaced with a higher
        // value piece
        if m.promotion().is_some() {
            self.remove_piece(m.dest_square());
        }
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
        // Update board's king square if king moves and remove ability to castle
        if piece_moving == PieceName::King {
            match self.to_move {
                Color::White => {
                    self.white_king_castle = false;
                    self.white_queen_castle = false;
                    self.white_king_square = m.dest_square();
                }
                Color::Black => {
                    self.black_queen_castle = false;
                    self.black_king_castle = false;
                    self.black_king_square = m.dest_square();
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
                    if m.origin_square() == m.dest_square().shift(South).shift(South) {
                        en_passant = true;
                        self.en_passant_square = m.dest_square().shift(South);
                    }
                }
                Color::Black => {
                    if m.dest_square().shift(North).shift(North) == m.origin_square() {
                        en_passant = true;
                        self.en_passant_square = m.origin_square().shift(South);
                    }
                }
            }
        }
        // If this move did not create a new en passant opportunity, the ability to do it goes away
        if !en_passant {
            self.en_passant_square = Square::INVALID;
        }

        // If a piece isn't captured or a pawn isn't moved, increment the half move clock.
        // Otherwise set it to zero
        if capture.is_none() && piece_moving != PieceName::Pawn {
            self.half_moves += 1;
        } else {
            self.half_moves = 0;
        }

        // Change the side to move after making a move
        self.to_move = self.to_move.opp();

        self.num_moves += 1;

        self.zobrist_hash = self.generate_hash();

        self.add_to_history();
    }

    #[allow(dead_code)]
    pub fn debug_bitboards(&self) {
        for color in &[Color::White, Color::Black] {
            for piece in PieceName::iter() {
                dbg!("{:?} {:?}", color, piece);
                dbg!(self.bitboards[*color as usize][piece as usize]);
                dbg!("\n");
            }
        }
    }
}

/// Function checks for the presence of the board in the game. If the board position will have occurred three times,
/// returns true indicating the position would be a stalemate due to the threefold repetition rule
pub fn check_for_3x_repetition(board: &Board) -> bool {
    // TODO: Check if this is correct. If not, just set offset to be 1 and 0 respectively
    let _offset = if board.to_move == Color::Black { 1 } else { 0 };
    board
        .history
        .iter()
        // .skip(offset)
        // .step_by(2)
        .filter(|x| &board.zobrist_hash == x)
        .count()
        > 1
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
        str += "En Passant Square: ";
        str += &self.en_passant_square.0.to_string();
        str += "\n";
        str += "Num moves made: ";
        str += &self.num_moves.to_string();
        write!(f, "{}", str)
    }
}

#[cfg(test)]
mod board_tests {
    use super::*;
    use crate::{board::fen, types::pieces::PieceName::*};
    #[test]
    fn test_place_piece() {
        let mut board = Board::default();
        board.place_piece(Rook, Color::White, Square(0));
        assert!(board.bitboards[Color::White as usize][Rook as usize].square_is_occupied(Square(0)));
    }

    #[test]
    fn test_remove_piece() {
        let mut board = fen::build_board(fen::STARTING_FEN);
        board.remove_piece(Square(0));
        assert!(board.bitboards[Color::White as usize][Rook as usize].square_is_empty(Square(0)));
        assert!(board.occupancies().square_is_empty(Square(0)));
    }
}
