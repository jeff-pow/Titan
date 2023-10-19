use core::fmt;
use std::sync::Arc;
use strum::IntoEnumIterator;

use crate::{
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
    bitboards: [[Bitboard; NUM_PIECES]; 2],
    pub color_occupancies: [Bitboard; 2],
    pub occupancies: Bitboard,
    pub array_board: [Option<Piece>; 64],
    pub material_val: [i32; 2],
    pub to_move: Color,
    castling: [bool; 4],
    pub en_passant_square: Option<Square>,
    pub num_moves: i32,
    pub half_moves: i32,
    pub zobrist_hash: u64,
    pub history: History,
    pub zobrist_consts: Arc<Zobrist>,
    pub mg: Arc<MoveGenerator>,
    pub prev_move: Move,
}

impl Default for Board {
    fn default() -> Self {
        let bitboards = [[Bitboard::EMPTY; 6]; 2];
        Board {
            bitboards,
            color_occupancies: [Bitboard::EMPTY; 2],
            occupancies: Bitboard::EMPTY,
            array_board: [None; 64],
            material_val: [0; 2],
            castling: [false; 4],
            to_move: Color::White,
            en_passant_square: None,
            num_moves: 0,
            half_moves: 0,
            zobrist_hash: 0,
            history: History::default(),
            zobrist_consts: Arc::new(Zobrist::default()),
            mg: Arc::new(MoveGenerator::default()),
            prev_move: Move::NULL,
        }
    }
}

impl Board {
    #[inline(always)]
    pub fn can_en_passant(&self) -> bool {
        self.en_passant_square.is_some()
    }

    #[inline(always)]
    pub fn castling(&self, c: Castle) -> bool {
        match c {
            Castle::None => panic!(),
            _ => self.castling[c as usize],
        }
    }

    #[inline(always)]
    pub fn set_castling(&mut self, c: Castle, b: bool) {
        match c {
            Castle::None => panic!(),
            _ => self.castling[c as usize] = b,
        }
    }

    #[inline(always)]
    pub fn bitboard(&self, side: Color, piece: PieceName) -> Bitboard {
        self.bitboards[side.idx()][piece.idx()]
    }

    #[inline(always)]
    pub fn is_draw(&self) -> bool {
        check_for_3x_repetition(self) || self.half_moves >= 100
    }

    #[inline(always)]
    pub fn square_occupied(&self, piece_type: PieceName, color: Color, sq: Square) -> bool {
        self.bitboards[color.idx()][piece_type.idx()].square_occupied(sq)
    }

    #[inline(always)]
    pub fn gen_color_occupancies(&mut self, color: Color) {
        // It's interesting to me that xor and bitwise or both seem to work here, only one piece should
        // be on a square at a time though so ¯\_(ツ)_/¯
        self.color_occupancies[color.idx()] = self.bitboards[color.idx()].iter().fold(Bitboard::EMPTY, |a, b| a ^ *b)
    }

    #[inline(always)]
    pub fn gen_occupancies(&mut self) {
        self.occupancies = self.bitboards.iter().flatten().fold(Bitboard::EMPTY, |a, b| a ^ *b)
    }

    #[inline(always)]
    pub fn color_occupancies(&self, color: Color) -> Bitboard {
        self.color_occupancies[color.idx()]
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
    pub fn name_by_bitboards(&self, sq: Square) -> Option<PieceName> {
        if sq.bitboard() & self.occupancies() == Bitboard::EMPTY {
            return None;
        }
        let color = if sq.bitboard() & self.color_occupancies(Color::White) != Bitboard::EMPTY {
            Color::White
        } else {
            Color::Black
        };
        for p in PieceName::iter().rev() {
            let bb = self.bitboard(color, p);
            for s in bb {
                if sq.bitboard() & s.bitboard() != Bitboard::EMPTY {
                    return Some(p);
                }
            }
        }
        unreachable!()
    }

    #[inline(always)]
    pub fn piece_at(&self, sq: Square) -> Option<PieceName> {
        self.array_board[sq.idx()].map(|piece| piece.name)
    }

    #[inline(always)]
    pub fn has_non_pawns(&self, side: Color) -> bool {
        self.occupancies()
            ^ self.bitboards[side.idx()][PieceName::King.idx()]
            ^ self.bitboards[side.idx()][PieceName::Pawn.idx()]
            != Bitboard::EMPTY
    }

    #[inline(always)]
    pub fn place_piece(&mut self, piece_type: PieceName, color: Color, sq: Square) {
        self.bitboards[color.idx()][piece_type.idx()] |= sq.bitboard();
        self.array_board[sq.idx()] = Some(Piece::new(piece_type, color));
        self.material_val[color.idx()] += piece_type.value();
        self.occupancies |= sq.bitboard();
        self.color_occupancies[color.idx()] |= sq.bitboard();
    }

    #[inline(always)]
    fn remove_piece(&mut self, sq: Square) {
        if let Some(piece) = self.array_board[sq.idx()] {
            self.array_board[sq.idx()] = None;
            self.bitboards[piece.color.idx()][piece.name.idx()] &= !sq.bitboard();
            self.material_val[piece.color.idx()] -= piece.value();
            self.occupancies &= !sq.bitboard();
            self.color_occupancies[piece.color.idx()] &= !sq.bitboard();
        }
    }

    #[inline(always)]
    pub fn king_square(&self, color: Color) -> Square {
        self.bitboard(color, PieceName::King).get_lsb()
    }

    #[inline(always)]
    pub fn in_check(&self, side: Color) -> bool {
        let king_square = self.king_square(side);
        if !king_square.is_valid() {
            return true;
        }
        self.square_under_attack(!side, king_square)
    }

    #[inline(always)]
    pub fn attackers(&self, sq: Square, occupancy: Bitboard) -> Bitboard {
        self.attackers_for_side(Color::White, sq, occupancy) | self.attackers_for_side(Color::Black, sq, occupancy)
    }

    #[inline(always)]
    pub fn attackers_for_side(&self, attacker: Color, sq: Square, occupancy: Bitboard) -> Bitboard {
        let pawn_attacks = self.mg.pawn_attacks(sq, !attacker) & self.bitboard(attacker, PieceName::Pawn);
        let knight_attacks = self.mg.knight_attacks(sq) & self.bitboard(attacker, PieceName::Knight);
        let bishop_attacks = self.mg.bishop_attacks(sq, occupancy) & self.bitboard(attacker, PieceName::Bishop);
        let rook_attacks = self.mg.rook_attacks(sq, occupancy) & self.bitboard(attacker, PieceName::Rook);
        let queen_attacks = (rook_attacks | bishop_attacks) & self.bitboard(attacker, PieceName::Queen);
        let king_attacks = self.mg.king_attacks(sq) & self.bitboard(attacker, PieceName::King);
        pawn_attacks | knight_attacks | bishop_attacks | rook_attacks | queen_attacks | king_attacks
    }

    #[inline(always)]
    // Function left with lots of variables to improve debugability...
    pub fn square_under_attack(&self, attacker: Color, sq: Square) -> bool {
        let attacker_occupancy = self.bitboards[attacker.idx()];
        let occupancy = self.occupancies();
        let pawn_attacks = self.mg.pawn_attacks(sq, !attacker);
        let knight_attacks = self.mg.knight_attacks(sq);
        let bishop_attacks = self.mg.bishop_attacks(sq, occupancy);
        let rook_attacks = self.mg.rook_attacks(sq, occupancy);
        let queen_attacks = rook_attacks | bishop_attacks;
        let king_attacks = self.mg.king_attacks(sq);

        let king_attacks_overlap = king_attacks & attacker_occupancy[PieceName::King.idx()];
        let queen_attacks_overlap = queen_attacks & attacker_occupancy[PieceName::Queen.idx()];
        let rook_attacks_overlap = rook_attacks & attacker_occupancy[PieceName::Rook.idx()];
        let bishop_attacks_overlap = bishop_attacks & attacker_occupancy[PieceName::Bishop.idx()];
        let knight_attacks_overlap = knight_attacks & attacker_occupancy[PieceName::Knight.idx()];
        let pawn_attacks_overlap = pawn_attacks & attacker_occupancy[PieceName::Pawn.idx()];

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
            Color::White => self.material_val[Color::White.idx()] - self.material_val[Color::Black.idx()],
            Color::Black => self.material_val[Color::Black.idx()] - self.material_val[Color::White.idx()],
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
                if self.is_quiet(m) {
                    return self.occupancies().square_is_empty(dest);
                } else {
                    let attacks = self.mg.pawn_attacks(origin, origin_color);
                    let enemy_color = self.color_at(origin).unwrap();
                    return attacks & m.dest_square().bitboard() & self.color_occupancies(!enemy_color)
                        != Bitboard::EMPTY;
                }
            }
            PieceName::King => self.mg.king_attacks(origin),
            PieceName::Queen => self.mg.rook_attacks(origin, occupancies) | self.mg.bishop_attacks(origin, occupancies),
            PieceName::Rook => self.mg.rook_attacks(origin, occupancies),
            PieceName::Bishop => self.mg.bishop_attacks(origin, occupancies),
            PieceName::Knight => self.mg.knight_attacks(origin),
        };

        let enemy_occupancies = !self.color_occupancies(self.to_move);
        let attacks = attack_bitboard & enemy_occupancies;

        attacks & dest.bitboard() != Bitboard::EMPTY
    }

    /// Determines if a move is "quiet" for quiescence search
    #[inline(always)]
    pub fn is_quiet(&self, m: Move) -> bool {
        self.occupancies().square_is_empty(m.dest_square())
    }

    /// Function makes a move and modifies board state to reflect the move that just happened
    pub fn make_move(&mut self, m: Move) {
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
                Castle::WhiteKing => {
                    self.place_piece(PieceName::Rook, self.to_move, Square(5));
                    self.remove_piece(Square(7));
                    self.set_castling(Castle::WhiteQueen, false);
                    self.set_castling(Castle::WhiteKing, false);
                }
                Castle::WhiteQueen => {
                    self.place_piece(PieceName::Rook, self.to_move, Square(3));
                    self.remove_piece(Square(0));
                    self.set_castling(Castle::WhiteQueen, false);
                    self.set_castling(Castle::WhiteKing, false);
                }
                Castle::BlackKing => {
                    self.place_piece(PieceName::Rook, self.to_move, Square(61));
                    self.remove_piece(Square(63));
                    self.set_castling(Castle::BlackQueen, false);
                    self.set_castling(Castle::BlackKing, false);
                }
                Castle::BlackQueen => {
                    self.place_piece(PieceName::Rook, self.to_move, Square(59));
                    self.remove_piece(Square(56));
                    self.set_castling(Castle::BlackQueen, false);
                    self.set_castling(Castle::BlackKing, false);
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
                    self.set_castling(Castle::WhiteQueen, false);
                    self.set_castling(Castle::WhiteKing, false);
                }
                Color::Black => {
                    self.set_castling(Castle::BlackQueen, false);
                    self.set_castling(Castle::BlackKing, false);
                }
            }
        }
        // If a rook moves, castling to that side is no longer possible
        if piece_moving == PieceName::Rook {
            match m.origin_square().0 {
                0 => self.set_castling(Castle::WhiteQueen, false),
                7 => self.set_castling(Castle::WhiteKing, false),
                56 => self.set_castling(Castle::BlackQueen, false),
                63 => self.set_castling(Castle::BlackKing, false),
                _ => (),
            }
        }
        // If a rook is captured, castling is no longer possible
        if let Some(cap) = capture {
            if cap == PieceName::Rook {
                match m.dest_square().0 {
                    0 => self.set_castling(Castle::WhiteQueen, false),
                    7 => self.set_castling(Castle::WhiteKing, false),
                    56 => self.set_castling(Castle::BlackQueen, false),
                    63 => self.set_castling(Castle::BlackKing, false),
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
                        self.en_passant_square = Some(m.dest_square().shift(South));
                    }
                }
                Color::Black => {
                    if m.dest_square().shift(North).shift(North) == m.origin_square() {
                        en_passant = true;
                        self.en_passant_square = Some(m.origin_square().shift(South));
                    }
                }
            }
        }
        // If this move did not create a new en passant opportunity, the ability to do it goes away
        if !en_passant {
            self.en_passant_square = None;
        }

        // If a piece isn't captured or a pawn isn't moved, increment the half move clock.
        // Otherwise set it to zero
        if capture.is_none() && piece_moving != PieceName::Pawn {
            self.half_moves += 1;
        } else {
            self.half_moves = 0;
        }

        // Change the side to move after making a move
        self.to_move = !self.to_move;

        self.num_moves += 1;

        self.zobrist_hash = self.generate_hash();

        self.add_to_history();

        self.prev_move = m;

        assert_eq!(Bitboard::EMPTY, self.color_occupancies(Color::White) & self.color_occupancies(Color::Black));
        let w = self.color_occupancies(Color::White);
        let b = self.color_occupancies(Color::Black);
        self.gen_color_occupancies(Color::White);
        self.gen_color_occupancies(Color::Black);
        debug_assert_eq!(w, self.color_occupancies(Color::White));
        debug_assert_eq!(b, self.color_occupancies(Color::Black));
    }

    #[allow(dead_code)]
    pub fn debug_bitboards(&self) {
        for color in &[Color::White, Color::Black] {
            for piece in PieceName::iter() {
                dbg!("{:?} {:?}", color, piece);
                dbg!(self.bitboards[color.idx()][piece.idx()]);
                dbg!("\n");
            }
        }
    }
}

/// Function checks for the presence of the board in the game. If the board position will have occurred three times,
/// returns true indicating the position would be a stalemate due to the threefold repetition rule
pub fn check_for_3x_repetition(board: &Board) -> bool {
    let arr = board.history.arr;
    let len = board.history.len;
    let mut count = 0;
    for i in (0..len).rev() {
        if arr[i] == board.zobrist_hash {
            count += 1;
        }
    }
    count >= 3
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
                if self.square_occupied(PieceName::King, Color::White, Square(idx)) {
                    str += "K"
                } else if self.square_occupied(PieceName::Queen, Color::White, Square(idx)) {
                    str += "Q"
                } else if self.square_occupied(PieceName::Rook, Color::White, Square(idx)) {
                    str += "R"
                } else if self.square_occupied(PieceName::Bishop, Color::White, Square(idx)) {
                    str += "B"
                } else if self.square_occupied(PieceName::Knight, Color::White, Square(idx)) {
                    str += "N"
                } else if self.square_occupied(PieceName::Pawn, Color::White, Square(idx)) {
                    str += "P"
                } else if self.square_occupied(PieceName::King, Color::Black, Square(idx)) {
                    str += "k"
                } else if self.square_occupied(PieceName::Queen, Color::Black, Square(idx)) {
                    str += "q"
                } else if self.square_occupied(PieceName::Rook, Color::Black, Square(idx)) {
                    str += "r"
                } else if self.square_occupied(PieceName::Bishop, Color::Black, Square(idx)) {
                    str += "b"
                } else if self.square_occupied(PieceName::Knight, Color::Black, Square(idx)) {
                    str += "n"
                } else if self.square_occupied(PieceName::Pawn, Color::Black, Square(idx)) {
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
        if self.castling(Castle::WhiteKing) {
            str += "K"
        };
        if self.castling(Castle::WhiteQueen) {
            str += "Q"
        };
        if self.castling(Castle::BlackKing) {
            str += "k"
        };
        if self.castling(Castle::BlackQueen) {
            str += "q"
        };
        str += "\n";
        if let Some(s) = &self.en_passant_square {
            str += "En Passant Square: ";
            str += &s.to_string();
        }
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
        assert!(board.bitboards[Color::White.idx()][Rook.idx()].square_occupied(Square(0)));
    }

    #[test]
    fn test_remove_piece() {
        let mut board = fen::build_board(fen::STARTING_FEN);
        board.remove_piece(Square(0));
        assert!(board.bitboards[Color::White.idx()][Rook.idx()].square_is_empty(Square(0)));
        assert!(board.occupancies().square_is_empty(Square(0)));
    }
}
