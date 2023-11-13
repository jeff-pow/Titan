use core::fmt;
use strum::IntoEnumIterator;

use crate::moves::moves::Castle;
use crate::{
    eval::nnue::Accumulator,
    moves::{
        movegenerator::MG,
        moves::Move,
        moves::{Direction::*, CASTLING_RIGHTS},
    },
    types::{
        bitboard::Bitboard,
        pieces::{Color, Piece, PieceName, NUM_PIECES},
        square::Square,
    },
};

use super::move_history::BoardHistory;

#[derive(Copy, Clone, PartialEq)]
pub struct Board {
    bitboards: [Bitboard; NUM_PIECES],
    color_occupancies: [Bitboard; 2],
    pub array_board: [Option<Piece>; 64],
    pub to_move: Color,
    pub castling_rights: u32,
    pub en_passant_square: Option<Square>,
    prev_move: Move,
    pub num_moves: usize,
    pub half_moves: usize,
    pub zobrist_hash: u64,
    pub history: BoardHistory,
    pub accumulator: Accumulator,
    pub in_check: bool,
}

impl Default for Board {
    fn default() -> Self {
        Self {
            bitboards: [Bitboard::EMPTY; 6],
            color_occupancies: [Bitboard::EMPTY; 2],
            array_board: [None; 64],
            castling_rights: 0,
            to_move: Color::White,
            en_passant_square: None,
            num_moves: 0,
            half_moves: 0,
            zobrist_hash: 0,
            history: BoardHistory::default(),
            prev_move: Move::NULL,
            accumulator: Accumulator::default(),
            in_check: false,
        }
    }
}

impl Board {
    pub fn can_en_passant(&self) -> bool {
        self.en_passant_square.is_some()
    }

    pub fn can_nmp(&self) -> bool {
        self.prev_move != Move::NULL
    }

    pub fn can_castle(&self, c: Castle) -> bool {
        match c {
            Castle::WhiteKing => self.castling_rights & Castle::WhiteKing as u32 != 0,
            Castle::WhiteQueen => self.castling_rights & Castle::WhiteQueen as u32 != 0,
            Castle::BlackKing => self.castling_rights & Castle::BlackKing as u32 != 0,
            Castle::BlackQueen => self.castling_rights & Castle::BlackQueen as u32 != 0,
            _ => panic!(),
        }
    }

    pub fn bitboard(&self, side: Color, piece: PieceName) -> Bitboard {
        self.bitboards[piece] & self.color_occupancies(side)
    }

    fn is_material_draw(&self) -> bool {
        // If we have any pawns, checkmate is still possible
        if self.piece_bitboard(PieceName::Pawn) != Bitboard::EMPTY {
            return false;
        }
        let piece_count = self.occupancies().count_bits();
        // King vs King can't checkmate
        if piece_count == 2
               // If there's three pieces and a singular knight or bishop, stalemate is impossible
            || (piece_count == 3 && ((self.piece_bitboard(PieceName::Knight).count_bits() == 1)
            || (self.piece_bitboard(PieceName::Bishop).count_bits() == 1)))
        {
            return true;
        } else if piece_count == 4 {
            // No combination of two knights and a king can checkmate
            if self.piece_bitboard(PieceName::Knight).count_bits() == 2 {
                return true;
            }
            // If there is one bishop per side, checkmate is impossible
            if self.color_occupancies(Color::White).count_bits() == 2
                && self.piece_bitboard(PieceName::Bishop).count_bits() == 2
            {
                return true;
            }
        }

        false
    }

    // Returns the type of piece captured by a move, if any
    pub fn capture(&self, m: Move) -> Option<PieceName> {
        if m.is_en_passant() {
            Some(PieceName::Pawn)
        } else {
            self.piece_at(m.dest_square())
        }
    }

    pub fn piece_bitboard(&self, p: PieceName) -> Bitboard {
        self.bitboards[p]
    }

    pub fn is_draw(&self) -> bool {
        self.history.check_for_3x_repetition(self.zobrist_hash) || self.half_moves >= 100 || self.is_material_draw()
    }

    pub fn color_occupancies(&self, color: Color) -> Bitboard {
        self.color_occupancies[color]
    }

    pub fn occupancies(&self) -> Bitboard {
        self.color_occupancies(Color::White) | self.color_occupancies(Color::Black)
    }

    pub fn color_at(&self, sq: Square) -> Option<Color> {
        self.array_board[sq].map(|piece| piece.color)
        // self.color_occupancies
        //     .iter()
        //     .position(|x| *x & sq.bitboard() != Bitboard::EMPTY)
        //     .map(Color::from)
    }

    pub fn piece_at(&self, sq: Square) -> Option<PieceName> {
        self.array_board[sq].map(|piece| piece.name)
        // self.bitboards
        //     .iter()
        //     .position(|x| *x & sq.bitboard() != Bitboard::EMPTY)
        //     .map(PieceName::from)
    }

    pub fn has_non_pawns(&self, side: Color) -> bool {
        self.occupancies() ^ self.bitboard(side, PieceName::King) ^ self.bitboard(side, PieceName::Pawn)
            != Bitboard::EMPTY
    }

    pub fn place_piece(&mut self, piece_type: PieceName, color: Color, sq: Square) {
        self.array_board[sq] = Some(Piece::new(piece_type, color));
        self.bitboards[piece_type] ^= sq.bitboard();
        self.color_occupancies[color] ^= sq.bitboard();
        self.accumulator.add_feature(piece_type, color, sq);
    }

    fn remove_piece(&mut self, sq: Square) {
        if let Some(piece) = self.array_board[sq] {
            self.array_board[sq] = None;
            self.bitboards[piece.name] ^= sq.bitboard();
            self.color_occupancies[piece.color] ^= sq.bitboard();
            self.accumulator.remove_feature(piece.name, piece.color, sq);
        }
    }

    pub fn king_square(&self, color: Color) -> Square {
        self.bitboard(color, PieceName::King).get_lsb()
    }

    pub fn attackers(&self, sq: Square, occupancy: Bitboard) -> Bitboard {
        self.attackers_for_side(Color::White, sq, occupancy) | self.attackers_for_side(Color::Black, sq, occupancy)
    }

    pub fn attackers_for_side(&self, attacker: Color, sq: Square, occupancy: Bitboard) -> Bitboard {
        let bishops = self.bitboard(attacker, PieceName::Queen) | self.bitboard(attacker, PieceName::Bishop);
        let rooks = self.bitboard(attacker, PieceName::Queen) | self.bitboard(attacker, PieceName::Rook);

        let pawn_attacks = MG.pawn_attacks(sq, !attacker) & self.bitboard(attacker, PieceName::Pawn);
        let knight_attacks = MG.knight_attacks(sq) & self.bitboard(attacker, PieceName::Knight);
        let bishop_attacks = MG.bishop_attacks(sq, occupancy) & bishops;
        let rook_attacks = MG.rook_attacks(sq, occupancy) & rooks;
        let king_attacks = MG.king_attacks(sq) & self.bitboard(attacker, PieceName::King);
        pawn_attacks | knight_attacks | bishop_attacks | rook_attacks | king_attacks
    }

    pub fn square_under_attack(&self, attacker: Color, sq: Square) -> bool {
        self.attackers_for_side(attacker, sq, self.occupancies()) != Bitboard::EMPTY
    }

    fn in_check(&self, side: Color) -> bool {
        let king_square = self.king_square(side);
        if !king_square.is_valid() {
            return true;
        }
        self.square_under_attack(!side, king_square)
    }

    pub fn add_to_history(&mut self) {
        self.history.push(self.zobrist_hash);
    }

    fn material_val(&self, c: Color) -> i32 {
        self.bitboard(c, PieceName::Queen).count_bits() * PieceName::Queen.value()
            + self.bitboard(c, PieceName::Rook).count_bits() * PieceName::Rook.value()
            + self.bitboard(c, PieceName::Bishop).count_bits() * PieceName::Bishop.value()
            + self.bitboard(c, PieceName::Knight).count_bits() * PieceName::Knight.value()
            + self.bitboard(c, PieceName::Pawn).count_bits() * PieceName::Pawn.value()
    }

    pub fn material_balance(&self) -> i32 {
        match self.to_move {
            Color::White => self.material_val(Color::White) - self.material_val(Color::Black),
            Color::Black => self.material_val(Color::Black) - self.material_val(Color::White),
        }
    }

    pub fn is_valid(&self, m: Move) -> bool {
        if m == Move::NULL {
            return false;
        }

        assert!(m.origin_square().is_valid() && m.dest_square().is_valid());

        let origin = m.origin_square();
        let dest = m.dest_square();
        let color = self.color_at(origin);
        let piece = self.piece_at(origin);

        if piece.is_none() {
            return false;
        }

        let origin_color = color.unwrap();
        let dest_color = self.color_at(dest);

        if dest_color.is_some() && dest_color.unwrap() == origin_color {
            return false;
        }

        let occupancies = self.occupancies();
        let attack_bitboard = match piece.unwrap() {
            PieceName::Pawn => {
                return if self.is_quiet(m) {
                    self.occupancies().empty(dest)
                } else {
                    let attacks = MG.pawn_attacks(origin, origin_color);
                    let enemy_color = self.color_at(origin).unwrap();
                    attacks & m.dest_square().bitboard() & self.color_occupancies(!enemy_color) != Bitboard::EMPTY
                }
            }
            PieceName::King => MG.king_attacks(origin),
            PieceName::Queen => MG.rook_attacks(origin, occupancies) | MG.bishop_attacks(origin, occupancies),
            PieceName::Rook => MG.rook_attacks(origin, occupancies),
            PieceName::Bishop => MG.bishop_attacks(origin, occupancies),
            PieceName::Knight => MG.knight_attacks(origin),
        };

        let enemy_occupancies = !self.color_occupancies(self.to_move);
        let attacks = attack_bitboard & enemy_occupancies;

        attacks & dest.bitboard() != Bitboard::EMPTY
    }

    /// Returns true if a move does not capture a piece, and false if a piece is captured
    pub fn is_quiet(&self, m: Move) -> bool {
        self.occupancies().empty(m.dest_square())
    }

    /// Function makes a move and modifies board state to reflect the move that just happened.
    /// Returns true if a move was legal, and false if it was illegal.
    #[must_use]
    pub fn make_move(&mut self, m: Move) -> bool {
        let piece_moving = m.piece_moving();
        assert_eq!(piece_moving, m.piece_moving());
        let capture = self.capture(m);
        self.remove_piece(m.dest_square());
        self.place_piece(piece_moving, self.to_move, m.dest_square());
        self.remove_piece(m.origin_square());

        // Move rooks if a castle move is applied
        if m.is_castle() {
            match m.castle_type() {
                Castle::WhiteKing => {
                    self.place_piece(PieceName::Rook, self.to_move, Square(5));
                    self.remove_piece(Square(7));
                }
                Castle::WhiteQueen => {
                    self.place_piece(PieceName::Rook, self.to_move, Square(3));
                    self.remove_piece(Square(0));
                }
                Castle::BlackKing => {
                    self.place_piece(PieceName::Rook, self.to_move, Square(61));
                    self.remove_piece(Square(63));
                }
                Castle::BlackQueen => {
                    self.place_piece(PieceName::Rook, self.to_move, Square(59));
                    self.remove_piece(Square(56));
                }
                Castle::None => (),
            }
        } else if let Some(p) = m.promotion() {
            self.remove_piece(m.dest_square());
            self.place_piece(p, self.to_move, m.dest_square());
        } else if m.is_en_passant() {
            match self.to_move {
                Color::White => {
                    self.remove_piece(m.dest_square().shift(South));
                }
                Color::Black => {
                    self.remove_piece(m.dest_square().shift(North));
                }
            }
        }

        // If the end index of a move is 16 squares from the start (and a pawn moved), an en passant is possible
        // if m.flag() == MoveType::DoublePush {
        //     match self.to_move {
        //         Color::White => {
        //             self.en_passant_square = Some(m.dest_square().shift(South));
        //         }
        //         Color::Black => {
        //             self.en_passant_square = Some(m.origin_square().shift(South));
        //         }
        //     }
        // }
        self.en_passant_square = None;
        if piece_moving == PieceName::Pawn {
            match self.to_move {
                Color::White => {
                    if m.origin_square() == m.dest_square().shift(South).shift(South) {
                        self.en_passant_square = Some(m.dest_square().shift(South));
                    }
                }
                Color::Black => {
                    if m.dest_square().shift(North).shift(North) == m.origin_square() {
                        self.en_passant_square = Some(m.origin_square().shift(South));
                    }
                }
            }
        }

        // If a piece isn't captured or a pawn isn't moved, increment the half move clock.
        // Otherwise set it to zero
        if capture.is_none() && piece_moving != PieceName::Pawn {
            self.half_moves += 1;
        } else {
            self.half_moves = 0;
        }

        self.castling_rights &= CASTLING_RIGHTS[m.origin_square()] & CASTLING_RIGHTS[m.dest_square()];

        self.to_move = !self.to_move;

        self.num_moves += 1;

        self.add_to_history();

        self.prev_move = m;

        self.in_check = self.in_check(self.to_move);

        self.zobrist_hash = self.generate_hash();

        // Return false if the move leaves the opposite side in check, denoting an invalid move
        !self.in_check(!self.to_move)
    }

    pub fn make_null_move(&mut self) {
        self.to_move = !self.to_move;
        self.num_moves += 1;
        self.en_passant_square = None;
        self.prev_move = Move::NULL;
        self.add_to_history();
        self.zobrist_hash = self.generate_hash();
    }

    #[allow(dead_code)]
    pub fn debug_bitboards(&self) {
        for color in &[Color::White, Color::Black] {
            for piece in PieceName::iter() {
                dbg!("{:?} {:?}", color, piece);
                dbg!(self.bitboard(*color, piece));
                dbg!("\n");
            }
        }
    }

    pub fn refresh_accumulators(&mut self) {
        self.accumulator = Accumulator::default();
        for c in Color::iter() {
            for p in PieceName::iter().rev() {
                for sq in self.bitboard(c, p) {
                    self.accumulator.add_feature(p, c, sq)
                }
            }
        }
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
                let piece = self.piece_at(Square(idx));
                let color = self.color_at(Square(idx));
                let char = match piece {
                    Some(p) => match p {
                        PieceName::Pawn => "P",
                        PieceName::Knight => "N",
                        PieceName::Bishop => "B",
                        PieceName::Rook => "R",
                        PieceName::Queen => "Q",
                        PieceName::King => "K",
                    },
                    None => "_",
                };
                let char = if let Some(c) = color {
                    match c {
                        Color::White => char.to_uppercase(),
                        Color::Black => char.to_lowercase(),
                    }
                } else {
                    "_".to_string()
                };
                str += char.as_str();

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
        if self.can_castle(Castle::WhiteKing) {
            str += "K"
        };
        if self.can_castle(Castle::WhiteQueen) {
            str += "Q"
        };
        if self.can_castle(Castle::BlackKing) {
            str += "k"
        };
        if self.can_castle(Castle::BlackQueen) {
            str += "q"
        };
        str += "\n";
        str += "En Passant Square: ";
        if let Some(s) = &self.en_passant_square {
            str += &s.to_string();
        } else {
            str += "None";
        }
        str += "\n";
        str += "Num moves made: ";
        str += &self.num_moves.to_string();
        str += "\n";
        str += "Prev move: ";
        str += &self.prev_move.to_san();
        str += "\n";

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
        assert!(board.bitboard(Color::White, PieceName::Rook).occupied(Square(0)));
    }

    #[test]
    fn test_remove_piece() {
        let mut board = fen::build_board(fen::STARTING_FEN);
        board.remove_piece(Square(0));
        assert!(board.bitboard(Color::White, PieceName::Rook).empty(Square(0)));
        assert!(board.occupancies().empty(Square(0)));
    }
}
