use core::fmt;
use strum::IntoEnumIterator;

use crate::board::zobrist::ZOBRIST;
use crate::moves::attack_boards::{king_attacks, knight_attacks, pawn_attacks};
use crate::moves::magics::{bishop_attacks, queen_attacks, rook_attacks};
use crate::moves::movegenerator::MGT;
use crate::moves::movelist::MoveList;
use crate::moves::moves::{Castle, MoveType};
use crate::{
    eval::accumulator::Accumulator,
    moves::{
        moves::Move,
        moves::{Direction::*, CASTLING_RIGHTS},
    },
    types::{
        bitboard::Bitboard,
        pieces::{Color, Piece, PieceName, NUM_PIECES},
        square::Square,
    },
};

#[derive(Copy, Clone, PartialEq)]
pub struct Board {
    bitboards: [Bitboard; NUM_PIECES],
    color_occupancies: [Bitboard; 2],
    pub array_board: [Option<Piece>; 64],
    pub to_move: Color,
    pub castling_rights: u32,
    pub en_passant_square: Option<Square>,
    pub num_moves: usize,
    pub half_moves: usize,
    pub zobrist_hash: u64,
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
            accumulator: Accumulator::default(),
            in_check: false,
        }
    }
}

impl Board {
    pub fn bitboard(&self, side: Color, piece: PieceName) -> Bitboard {
        self.piece(piece) & self.color(side)
    }

    pub fn piece(&self, piece: PieceName) -> Bitboard {
        self.bitboards[piece]
    }

    pub fn color(&self, color: Color) -> Bitboard {
        self.color_occupancies[color]
    }

    pub fn occupancies(&self) -> Bitboard {
        self.color(Color::White) | self.color(Color::Black)
    }

    pub fn color_at(&self, sq: Square) -> Option<Color> {
        self.array_board[sq].map(|piece| piece.color)
    }

    pub fn piece_at(&self, sq: Square) -> Option<PieceName> {
        self.array_board[sq].map(|piece| piece.name)
    }

    fn is_material_draw(&self) -> bool {
        // If we have any pawns, checkmate is still possible
        if self.piece(PieceName::Pawn) != Bitboard::EMPTY {
            return false;
        }
        let piece_count = self.occupancies().count_bits();
        // King vs King can't checkmate
        if piece_count == 2
               // If there's three pieces and a singular knight or bishop, stalemate is impossible
            || (piece_count == 3 && ((self.piece(PieceName::Knight).count_bits() == 1)
            || (self.piece(PieceName::Bishop).count_bits() == 1)))
        {
            return true;
        } else if piece_count == 4 {
            // No combination of two knights and a king can checkmate
            if self.piece(PieceName::Knight).count_bits() == 2 {
                return true;
            }
            // If there is one bishop per side, checkmate is impossible
            if self.color(Color::White).count_bits() == 2
                && self.piece(PieceName::Bishop).count_bits() == 2
            {
                return true;
            }
        }

        false
    }

    /// Returns the type of piece captured by a move, if any
    pub fn capture(&self, m: Move) -> Option<PieceName> {
        if m.is_en_passant() {
            Some(PieceName::Pawn)
        } else {
            self.piece_at(m.dest_square())
        }
    }

    pub fn is_draw(&self) -> bool {
        self.half_moves >= 100 || self.is_material_draw()
    }

    pub fn has_non_pawns(&self, side: Color) -> bool {
        self.occupancies()
            ^ self.bitboard(side, PieceName::King)
            ^ self.bitboard(side, PieceName::Pawn)
            != Bitboard::EMPTY
    }

    pub fn can_en_passant(&self) -> bool {
        self.en_passant_square.is_some()
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

    pub fn place_piece<const NNUE: bool>(
        &mut self,
        piece_type: PieceName,
        color: Color,
        sq: Square,
    ) {
        self.array_board[sq] = Some(Piece::new(piece_type, color));
        self.bitboards[piece_type] ^= sq.bitboard();
        self.color_occupancies[color] ^= sq.bitboard();
        self.zobrist_hash ^= ZOBRIST.piece_square_hashes[color][piece_type][sq];
        if NNUE {
            self.accumulator.add_feature(piece_type, color, sq);
        }
    }

    fn remove_piece<const NNUE: bool>(&mut self, sq: Square) {
        if let Some(piece) = self.array_board[sq] {
            self.array_board[sq] = None;
            self.bitboards[piece.name] ^= sq.bitboard();
            self.color_occupancies[piece.color] ^= sq.bitboard();
            self.zobrist_hash ^= ZOBRIST.piece_square_hashes[piece.color][piece.name][sq];
            if NNUE {
                self.accumulator.remove_feature(piece.name, piece.color, sq);
            }
        }
    }

    pub fn king_square(&self, color: Color) -> Square {
        self.bitboard(color, PieceName::King).get_lsb()
    }

    pub fn attackers(&self, sq: Square, occupancy: Bitboard) -> Bitboard {
        self.attackers_for_side(Color::White, sq, occupancy)
            | self.attackers_for_side(Color::Black, sq, occupancy)
    }

    pub fn attackers_for_side(&self, attacker: Color, sq: Square, occupancy: Bitboard) -> Bitboard {
        let bishops = self.piece(PieceName::Queen) | self.piece(PieceName::Bishop);
        let rooks = self.piece(PieceName::Queen) | self.piece(PieceName::Rook);
        let pawn_attacks = pawn_attacks(sq, !attacker) & self.piece(PieceName::Pawn);
        let knight_attacks = knight_attacks(sq) & self.piece(PieceName::Knight);
        let bishop_attacks = bishop_attacks(sq, occupancy) & bishops;
        let rook_attacks = rook_attacks(sq, occupancy) & rooks;
        let king_attacks = king_attacks(sq) & self.piece(PieceName::King);
        (pawn_attacks | knight_attacks | bishop_attacks | rook_attacks | king_attacks)
            & self.color(attacker)
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

    pub(crate) fn is_pseudo_legal(&self, m: Move) -> bool {
        let piece_moving = self.piece_at(m.origin_square());
        if m == Move::NULL
            || self.color_at(m.origin_square()).map_or(true, |c| c != self.to_move)
            || piece_moving.is_none()
            || self.piece_at(m.origin_square()) != piece_moving
        {
            return false;
        }

        if let Some(PieceName::King) = self.capture(m) {
            return false;
        }

        match piece_moving.unwrap() {
            PieceName::Knight => {
                m.flag() == MoveType::Normal
                    && knight_attacks(m.origin_square())
                        & !self.color(self.to_move)
                        & m.dest_square().bitboard()
                        != Bitboard::EMPTY
            }
            PieceName::Bishop => {
                m.flag() == MoveType::Normal
                    && bishop_attacks(m.origin_square(), self.occupancies())
                        & !self.color(self.to_move)
                        & m.dest_square().bitboard()
                        != Bitboard::EMPTY
            }
            PieceName::Rook => {
                m.flag() == MoveType::Normal
                    && rook_attacks(m.origin_square(), self.occupancies())
                        & !self.color(self.to_move)
                        & m.dest_square().bitboard()
                        != Bitboard::EMPTY
            }
            PieceName::Queen => {
                m.flag() == MoveType::Normal
                    && queen_attacks(m.origin_square(), self.occupancies())
                        & !self.color(self.to_move)
                        & m.dest_square().bitboard()
                        != Bitboard::EMPTY
            }
            PieceName::Pawn => {
                let mut list = MoveList::default();
                // Verifying pawn moves, like castles, is also a pain that I didn't feel like
                // typing the checks for :). So just generate the moves and find if it exists
                self.generate_pawn_moves(MGT::All, &mut list);
                for pawn in list.arr.iter().map(|e| e.m) {
                    if m == pawn {
                        return true;
                    }
                }
                false
            }
            PieceName::King => {
                if self.square_under_attack(!self.to_move, self.king_square(self.to_move)) {
                    return false;
                }
                if m.flag() == MoveType::CastleMove {
                    let mut list = MoveList::default();
                    // Verifying castling moves is sort of a PITA, so we just generate the moves
                    // and match it against the move in question to determine if it's legal
                    self.generate_castling_moves(&mut list);
                    for castle in list.arr.iter().map(|e| e.m) {
                        if m == castle {
                            return true;
                        }
                    }
                }
                m.flag() == MoveType::Normal
                    && king_attacks(m.origin_square())
                        & !self.color(self.to_move)
                        & m.dest_square().bitboard()
                        != Bitboard::EMPTY
            }
        }
    }

    /// Returns true if a move does not capture a piece, and false if a piece is captured
    pub fn is_quiet(&self, m: Move) -> bool {
        self.occupancies().empty(m.dest_square())
    }

    /// Function makes a move and modifies board state to reflect the move that just happened.
    /// Returns true if a move was legal, and false if it was illegal.
    #[must_use]
    pub fn make_move<const NNUE: bool>(&mut self, m: Move) -> bool {
        let dest = m.dest_square();
        let src = m.origin_square();
        let str = self.to_string();
        let r = m.to_san();
        let board = self.clone();

        let piece_moving = m.piece_moving();
        assert_eq!(piece_moving, m.piece_moving());
        let capture = self.capture(m);
        self.remove_piece::<NNUE>(m.dest_square());
        self.place_piece::<NNUE>(piece_moving, self.to_move, m.dest_square());
        self.remove_piece::<NNUE>(m.origin_square());

        // Move rooks if a castle move is applied
        if m.is_castle() {
            match m.castle_type() {
                Castle::WhiteKing => {
                    self.place_piece::<NNUE>(PieceName::Rook, self.to_move, Square(5));
                    self.remove_piece::<NNUE>(Square(7));
                }
                Castle::WhiteQueen => {
                    self.place_piece::<NNUE>(PieceName::Rook, self.to_move, Square(3));
                    self.remove_piece::<NNUE>(Square(0));
                }
                Castle::BlackKing => {
                    self.place_piece::<NNUE>(PieceName::Rook, self.to_move, Square(61));
                    self.remove_piece::<NNUE>(Square(63));
                }
                Castle::BlackQueen => {
                    self.place_piece::<NNUE>(PieceName::Rook, self.to_move, Square(59));
                    self.remove_piece::<NNUE>(Square(56));
                }
                Castle::None => (),
            }
        } else if let Some(p) = m.promotion() {
            self.remove_piece::<NNUE>(m.dest_square());
            self.place_piece::<NNUE>(p, self.to_move, m.dest_square());
        } else if m.is_en_passant() {
            match self.to_move {
                Color::White => {
                    self.remove_piece::<NNUE>(m.dest_square().shift(South));
                }
                Color::Black => {
                    self.remove_piece::<NNUE>(m.dest_square().shift(North));
                }
            }
        }

        // Xor out the old en passant square hash
        if let Some(sq) = self.en_passant_square {
            self.zobrist_hash ^= ZOBRIST.en_passant[sq];
        }
        // If the end index of a move is 16 squares from the start (and a pawn moved), an en passant is possible
        self.en_passant_square = None;
        if m.flag() == MoveType::DoublePush {
            match self.to_move {
                Color::White => {
                    self.en_passant_square = Some(m.dest_square().shift(South));
                }
                Color::Black => {
                    self.en_passant_square = Some(m.dest_square().shift(North));
                }
            }
        }
        // Xor in the new en passant square hash
        if let Some(sq) = self.en_passant_square {
            self.zobrist_hash ^= ZOBRIST.en_passant[sq];
        }

        // If a piece isn't captured and a pawn isn't moved, increment the half move clock.
        // Otherwise set it to zero
        if capture.is_none() && piece_moving != PieceName::Pawn {
            self.half_moves += 1;
        } else {
            self.half_moves = 0;
        }

        self.zobrist_hash ^= ZOBRIST.castling[self.castling_rights as usize];
        self.castling_rights &=
            CASTLING_RIGHTS[m.origin_square()] & CASTLING_RIGHTS[m.dest_square()];
        self.zobrist_hash ^= ZOBRIST.castling[self.castling_rights as usize];

        self.to_move = !self.to_move;
        self.zobrist_hash ^= ZOBRIST.turn_hash;

        self.num_moves += 1;

        self.in_check = self.in_check(self.to_move);

        // Return false if the move leaves the opposite side in check, denoting an invalid move
        let ret = !self.in_check(!self.to_move);
        let a = self.to_string();
        if ret {
            assert_eq!((self.color_occupancies[0] & self.bitboards[PieceName::King]).count_bits(), 1, "{} {}", self.zobrist_hash, m.0);
            assert_eq!((self.color_occupancies[1] & self.bitboards[PieceName::King]).count_bits(), 1);
        }
        ret
    }

    pub fn make_null_move(&mut self) {
        self.to_move = !self.to_move;
        self.zobrist_hash ^= ZOBRIST.turn_hash;
        self.num_moves += 1;
        if let Some(sq) = self.en_passant_square {
            self.zobrist_hash ^= ZOBRIST.en_passant[sq];
        }
        self.en_passant_square = None;
    }

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
        board.place_piece::<false>(Rook, Color::White, Square(0));
        assert!(board.bitboard(Color::White, PieceName::Rook).occupied(Square(0)));
    }

    #[test]
    fn test_remove_piece() {
        let mut board = fen::build_board(fen::STARTING_FEN);
        board.remove_piece::<false>(Square(0));
        assert!(board.bitboard(Color::White, PieceName::Rook).empty(Square(0)));
        assert!(board.occupancies().empty(Square(0)));
    }
}
