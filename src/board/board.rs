use core::fmt;

use crate::{
    board::zobrist::ZOBRIST,
    eval::accumulator::Delta,
    moves::{
        attack_boards::{king_attacks, knight_attacks, pawn_attacks, pawn_set_attacks, RANKS},
        magics::{bishop_attacks, queen_attacks, rook_attacks},
        moves::{
            Castle,
            Direction::{North, South},
            Move, MoveType, CASTLING_RIGHTS,
        },
    },
    types::{
        bitboard::Bitboard,
        pieces::{Color, Piece, PieceName, NUM_PIECES},
        square::Square,
    },
};

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Board {
    bitboards: [Bitboard; NUM_PIECES],
    color_occupancies: [Bitboard; 2],
    mailbox: [Piece; 64],
    pub to_move: Color,
    pub castling_rights: u32,
    pub en_passant_square: Option<Square>,
    pub num_moves: usize,
    pub half_moves: usize,
    pub zobrist_hash: u64,
    pub in_check: bool,
    pub(crate) delta: Delta,
    threats: Bitboard,
}

impl Default for Board {
    fn default() -> Self {
        Self {
            bitboards: [Bitboard::EMPTY; 6],
            color_occupancies: [Bitboard::EMPTY; 2],
            mailbox: [Piece::None; 64],
            castling_rights: 0,
            to_move: Color::White,
            en_passant_square: None,
            num_moves: 0,
            half_moves: 0,
            zobrist_hash: 0,
            in_check: false,
            threats: Bitboard::EMPTY,
            delta: Delta::default(),
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

    pub fn piece_at(&self, sq: Square) -> Piece {
        self.mailbox[sq]
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
        }
        if piece_count == 4 {
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
    pub fn capture(&self, m: Move) -> Piece {
        if m.is_en_passant() {
            Piece::new(PieceName::Pawn, !self.to_move)
        } else {
            self.piece_at(m.to())
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

    pub const fn can_en_passant(&self) -> bool {
        self.en_passant_square.is_some()
    }

    pub const fn can_castle(&self, c: Castle) -> bool {
        match c {
            Castle::WhiteKing => self.castling_rights & Castle::WhiteKing as u32 != 0,
            Castle::WhiteQueen => self.castling_rights & Castle::WhiteQueen as u32 != 0,
            Castle::BlackKing => self.castling_rights & Castle::BlackKing as u32 != 0,
            Castle::BlackQueen => self.castling_rights & Castle::BlackQueen as u32 != 0,
            Castle::None => panic!(),
        }
    }

    pub fn place_piece<const NNUE: bool>(&mut self, piece: Piece, sq: Square) {
        let color = piece.color();
        let name = piece.name();
        self.mailbox[sq] = piece;
        self.bitboards[piece.name()] ^= sq.bitboard();
        self.color_occupancies[color] ^= sq.bitboard();
        self.zobrist_hash ^= ZOBRIST.piece_square_hashes[color][name][sq];
        if NNUE {
            // acc.add_feature(name, color, sq);
            self.delta.add(piece, sq);
        }
    }

    fn remove_piece<const NNUE: bool>(&mut self, sq: Square) {
        let piece = self.mailbox[sq];
        if piece != Piece::None {
            self.mailbox[sq] = Piece::None;
            self.bitboards[piece.name()] ^= sq.bitboard();
            self.color_occupancies[piece.color()] ^= sq.bitboard();
            self.zobrist_hash ^= ZOBRIST.piece_square_hashes[piece.color()][piece.name()][sq];
            if NNUE {
                // acc.remove_feature(piece.name(), piece.color(), sq);
                self.delta.remove(piece, sq);
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

    fn threats_in_check(&self) -> bool {
        self.king_square(self.to_move).bitboard() & self.threats != Bitboard::EMPTY
    }

    pub(crate) const fn threats(&self) -> Bitboard {
        self.threats
    }

    pub(crate) fn calculate_threats(&mut self) {
        let attacker = !self.to_move;
        let mut threats = Bitboard::EMPTY;

        threats |= pawn_set_attacks(self.bitboard(attacker, PieceName::Pawn), attacker);

        let rooks =
            (self.piece(PieceName::Rook) | self.piece(PieceName::Queen)) & self.color(attacker);
        rooks.into_iter().for_each(|sq| threats |= rook_attacks(sq, self.occupancies()));

        let bishops =
            (self.piece(PieceName::Bishop) | self.piece(PieceName::Queen)) & self.color(attacker);
        bishops.into_iter().for_each(|sq| threats |= bishop_attacks(sq, self.occupancies()));

        self.bitboard(attacker, PieceName::Knight)
            .into_iter()
            .for_each(|sq| threats |= knight_attacks(sq));

        threats |= king_attacks(self.king_square(attacker));

        self.threats = threats;
    }

    pub(crate) fn is_pseudo_legal(&self, m: Move) -> bool {
        if m == Move::NULL {
            return false;
        }

        let from = m.from();
        let to = m.to();

        let moved_piece = self.piece_at(from);
        let captured_piece = self.piece_at(to);
        let is_capture = captured_piece != Piece::None;
        let is_pawn_double_push = m.flag() == MoveType::DoublePush;

        if moved_piece != m.piece_moving() {
            return false;
        }

        if moved_piece == Piece::None {
            return false;
        }

        if moved_piece.color() != self.to_move {
            return false;
        }

        if is_capture && captured_piece.color() == self.to_move {
            return false;
        }

        if m.is_castle() {
            if self.in_check {
                return false;
            }
            if moved_piece.name() != PieceName::King {
                return false;
            }
            let castle = m.castle_type();
            if !self.can_castle(castle) {
                return false;
            }

            if self.occupancies() & castle.empty_squares() != Bitboard::EMPTY {
                return false;
            }
            if castle.check_squares() & self.threats() != Bitboard::EMPTY {
                return false;
            }

            return true;
        }

        match moved_piece.name() {
            PieceName::Pawn => {
                if is_capture && is_pawn_double_push {
                    return false;
                }
                let should_be_promoting = to.bitboard() & RANKS[7] != Bitboard::EMPTY
                    || to.bitboard() & RANKS[0] != Bitboard::EMPTY;
                if should_be_promoting && m.promotion().is_none() {
                    return false;
                }
                let up = match self.to_move {
                    Color::White => North,
                    Color::Black => South,
                };
                if m.is_en_passant() {
                    return Some(to) == self.en_passant_square;
                }
                if is_pawn_double_push {
                    let one_forward = from.shift(up);
                    return self.piece_at(one_forward) == Piece::None
                        && to == one_forward.shift(up);
                }
                if !is_capture {
                    return to == from.shift(up) && captured_piece == Piece::None;
                }
                // Captures
                (pawn_set_attacks(from.bitboard(), self.to_move) & to.bitboard()) != Bitboard::EMPTY
            }
            PieceName::Knight => to.bitboard() & knight_attacks(from) != Bitboard::EMPTY,
            PieceName::Bishop => {
                to.bitboard() & bishop_attacks(from, self.occupancies()) != Bitboard::EMPTY
            }
            PieceName::Rook => {
                to.bitboard() & rook_attacks(from, self.occupancies()) != Bitboard::EMPTY
            }
            PieceName::Queen => {
                to.bitboard() & queen_attacks(from, self.occupancies()) != Bitboard::EMPTY
            }
            PieceName::King => to.bitboard() & king_attacks(from) != Bitboard::EMPTY,
            PieceName::None => panic!(),
        }
    }

    /// Function makes a move and modifies board state to reflect the move that just happened.
    /// Returns true if a move was legal, and false if it was illegal.
    #[must_use]
    pub fn make_move<const NNUE: bool>(&mut self, m: Move) -> bool {
        assert_eq!(self.delta.num_add, 0);
        assert_eq!(self.delta.num_sub, 0);
        let piece_moving = m.piece_moving();
        assert_eq!(piece_moving, self.piece_at(m.from()));
        let capture = self.capture(m);
        self.remove_piece::<NNUE>(m.to());

        if m.promotion().is_none() {
            self.place_piece::<NNUE>(piece_moving, m.to());
        }

        self.remove_piece::<NNUE>(m.from());

        // Move rooks if a castle move is applied
        if m.is_castle() {
            let rook = Piece::new(PieceName::Rook, self.to_move);
            self.place_piece::<NNUE>(rook, m.castle_type().rook_dest());
            self.remove_piece::<NNUE>(m.castle_type().rook_src());
        } else if let Some(p) = m.promotion() {
            self.place_piece::<NNUE>(p, m.to());
        } else if m.is_en_passant() {
            match self.to_move {
                Color::White => {
                    self.remove_piece::<NNUE>(m.to().shift(South));
                }
                Color::Black => {
                    self.remove_piece::<NNUE>(m.to().shift(North));
                }
            }
        }

        // If we are in check after all pieces have been moved, this move is illegal and we return
        // false to denote so
        if self.in_check(self.to_move) {
            return false;
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
                    self.en_passant_square = Some(m.to().shift(South));
                }
                Color::Black => {
                    self.en_passant_square = Some(m.to().shift(North));
                }
            }
        }
        // Xor in the new en passant square hash
        if let Some(sq) = self.en_passant_square {
            self.zobrist_hash ^= ZOBRIST.en_passant[sq];
        }

        // If a piece isn't captured and a pawn isn't moved, increment the half move clock.
        // Otherwise set it to zero

        if capture == Piece::None && piece_moving.name() != PieceName::Pawn {
            self.half_moves += 1;
        } else {
            self.half_moves = 0;
        }

        self.zobrist_hash ^= ZOBRIST.castling[self.castling_rights as usize];
        self.castling_rights &= CASTLING_RIGHTS[m.from()] & CASTLING_RIGHTS[m.to()];
        self.zobrist_hash ^= ZOBRIST.castling[self.castling_rights as usize];

        self.to_move = !self.to_move;
        self.zobrist_hash ^= ZOBRIST.turn_hash;

        self.num_moves += 1;

        self.calculate_threats();
        self.in_check = self.threats_in_check();

        // This move is valid, so we return true to denote this fact
        true
    }

    pub fn make_null_move(&mut self) {
        self.to_move = !self.to_move;
        self.zobrist_hash ^= ZOBRIST.turn_hash;
        self.num_moves += 1;
        self.half_moves += 1;
        if let Some(sq) = self.en_passant_square {
            self.zobrist_hash ^= ZOBRIST.en_passant[sq];
        }
        self.en_passant_square = None;
        self.calculate_threats();
    }

    pub fn debug_bitboards(&self) {
        for color in Color::iter() {
            for piece in PieceName::iter() {
                dbg!("{:?} {:?}", color, piece);
                dbg!(self.bitboard(color, piece));
                dbg!("\n");
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
                let char = piece.char();

                str += &char;

                str.push_str(" | ");
            }

            str.push('\n');
        }

        str.push_str("    a   b   c   d   e   f   g   h\n");

        write!(f, "{str}")
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
            str += "K";
        };
        if self.can_castle(Castle::WhiteQueen) {
            str += "Q";
        };
        if self.can_castle(Castle::BlackKing) {
            str += "k";
        };
        if self.can_castle(Castle::BlackQueen) {
            str += "q";
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

        write!(f, "{str}")
    }
}

#[cfg(test)]
mod board_tests {
    use super::*;
    use crate::board::fen;
    #[test]
    fn test_place_piece() {
        let mut board = Board::default();
        board.place_piece::<false>(Piece::WhiteRook, Square(0));
        assert!(board.bitboard(Color::White, PieceName::Rook).occupied(Square(0)));
    }

    #[test]
    fn test_remove_piece() {
        let board = fen::build_board(fen::STARTING_FEN);

        let mut c = board;
        c.remove_piece::<false>(Square(0));
        assert!(c.bitboard(Color::White, PieceName::Rook).empty(Square(0)));
        assert!(c.occupancies().empty(Square(0)));
        assert_ne!(c, board);

        let mut c = board;
        c.remove_piece::<false>(Square(27));
        assert_eq!(board, c);
    }
}
