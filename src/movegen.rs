use crate::{
    board::Board,
    chess_move::{
        Direction,
        Direction::{North, NorthEast, NorthWest, South, SouthEast, SouthWest},
    },
    types::{
        bitboard::Bitboard,
        pieces::{Color, Piece, PieceName},
        square::Square,
    },
};

use super::{
    attack_boards::{king_attacks, knight_attacks, RANKS},
    chess_move::{Castle, Move, MoveType},
    magics::{bishop_attacks, rook_attacks},
    movelist::MoveList,
};

#[allow(clippy::upper_case_acronyms)]
pub type MGT = MoveGenerationType;
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum MoveGenerationType {
    CapturesOnly,
    QuietsOnly,
    All,
}

impl Board {
    /// Generates all pseudolegal moves
    pub fn generate_moves(&self, gen_type: MGT, moves: &mut MoveList) {
        let destinations = match gen_type {
            MoveGenerationType::CapturesOnly => self.color(!self.stm),
            MoveGenerationType::QuietsOnly => !self.occupancies(),
            MoveGenerationType::All => !self.color(self.stm),
        };

        let knights = self.piece_color(self.stm, PieceName::Knight);
        let kings = self.piece_color(self.stm, PieceName::King);
        let bishops = self.piece_color(self.stm, PieceName::Bishop) | self.piece_color(self.stm, PieceName::Queen);
        let rooks = self.piece_color(self.stm, PieceName::Rook) | self.piece_color(self.stm, PieceName::Queen);

        self.jumper_moves(knights, destinations, moves, knight_attacks);
        self.jumper_moves(kings, destinations & !self.threats(), moves, king_attacks);
        self.magic_moves(rooks, destinations, moves, rook_attacks);
        self.magic_moves(bishops, destinations, moves, bishop_attacks);
        self.pawn_moves(gen_type, moves);
        if gen_type == MGT::QuietsOnly || gen_type == MGT::All {
            self.castling_moves(moves);
        }
    }

    fn castling_moves(&self, moves: &mut MoveList) {
        if self.stm == Color::White {
            if self.can_castle(Castle::WhiteKing)
                && self.threats() & Castle::WhiteKing.check_squares() == Bitboard::EMPTY
                && self.occupancies() & Castle::WhiteKing.empty_squares() == Bitboard::EMPTY
            {
                moves.push(Move::new(Square::E1, Square::G1, MoveType::CastleMove, Piece::WhiteKing));
            }
            if self.can_castle(Castle::WhiteQueen)
                && self.threats() & Castle::WhiteQueen.check_squares() == Bitboard::EMPTY
                && self.occupancies() & Castle::WhiteQueen.empty_squares() == Bitboard::EMPTY
            {
                moves.push(Move::new(Square::E1, Square::C1, MoveType::CastleMove, Piece::WhiteKing));
            }
        } else {
            if self.can_castle(Castle::BlackKing)
                && self.threats() & Castle::BlackKing.check_squares() == Bitboard::EMPTY
                && self.occupancies() & Castle::BlackKing.empty_squares() == Bitboard::EMPTY
            {
                moves.push(Move::new(Square::E8, Square::G8, MoveType::CastleMove, Piece::BlackKing));
            }
            if self.can_castle(Castle::BlackQueen)
                && self.threats() & Castle::BlackQueen.check_squares() == Bitboard::EMPTY
                && self.occupancies() & Castle::BlackQueen.empty_squares() == Bitboard::EMPTY
            {
                moves.push(Move::new(Square::E8, Square::C8, MoveType::CastleMove, Piece::BlackKing));
            }
        }
    }

    fn pawn_moves(&self, gen_type: MGT, moves: &mut MoveList) {
        let piece = Piece::new(PieceName::Pawn, self.stm);
        let pawns = self.piece_color(self.stm, PieceName::Pawn);
        let vacancies = !self.occupancies();
        let enemies = self.color(!self.stm);

        let non_promotions = pawns & if self.stm == Color::White { !RANKS[6] } else { !RANKS[1] };
        let promotions = pawns & if self.stm == Color::White { RANKS[6] } else { RANKS[1] };

        let up = if self.stm == Color::White { North } else { South };
        let right = if self.stm == Color::White { NorthEast } else { SouthWest };
        let left = if self.stm == Color::White { NorthWest } else { SouthEast };

        let rank3 = if self.stm == Color::White { RANKS[2] } else { RANKS[5] };

        if matches!(gen_type, MGT::All | MGT::QuietsOnly) {
            // Single and double pawn pushes w/o captures
            let push_one = vacancies & non_promotions.shift(up);
            let push_two = vacancies & (push_one & rank3).shift(up);
            for dest in push_one {
                let src = dest.shift(up.opp());
                moves.push(Move::new(src, dest, MoveType::Normal, piece));
            }
            for dest in push_two {
                let src = dest.shift(up.opp()).shift(up.opp());
                moves.push(Move::new(src, dest, MoveType::DoublePush, piece));
            }
        }

        // Promotions - captures and straight pushes
        // Promotions are generated with captures because they are so good
        if matches!(gen_type, MGT::All | MGT::CapturesOnly) && promotions != Bitboard::EMPTY {
            let no_capture_promotions = promotions.shift(up) & vacancies;
            let left_capture_promotions = promotions.shift(left) & enemies;
            let right_capture_promotions = promotions.shift(right) & enemies;
            for dest in no_capture_promotions {
                gen_promotions(piece, dest.shift(up.opp()), dest, moves);
            }
            for dest in left_capture_promotions {
                gen_promotions(piece, dest.shift(left.opp()), dest, moves);
            }
            for dest in right_capture_promotions {
                gen_promotions(piece, dest.shift(right.opp()), dest, moves);
            }
        }

        if matches!(gen_type, MGT::All | MGT::CapturesOnly) {
            // Captures that do not lead to promotions
            if non_promotions != Bitboard::EMPTY {
                let left_captures = non_promotions.shift(left) & enemies;
                let right_captures = non_promotions.shift(right) & enemies;
                for dest in left_captures {
                    let src = dest.shift(left.opp());
                    moves.push(Move::new(src, dest, MoveType::Normal, piece));
                }
                for dest in right_captures {
                    let src = dest.shift(right.opp());
                    moves.push(Move::new(src, dest, MoveType::Normal, piece));
                }
            }

            // En Passant
            if self.can_en_passant() {
                if let Some(x) = self.get_en_passant(left.opp(), piece) {
                    moves.push(x);
                }
                if let Some(x) = self.get_en_passant(right.opp(), piece) {
                    moves.push(x);
                }
            }
        }
    }

    fn get_en_passant(&self, dir: Direction, piece: Piece) -> Option<Move> {
        let sq = self.en_passant_square?.checked_shift(dir)?;
        let pawn = sq.bitboard() & self.piece_color(self.stm, PieceName::Pawn);
        if pawn != Bitboard::EMPTY {
            let dest = self.en_passant_square?;
            let src = dest.checked_shift(dir)?;
            return Some(Move::new(src, dest, MoveType::EnPassant, piece));
        }
        None
    }

    fn magic_moves(
        &self,
        pieces: Bitboard,
        destinations: Bitboard,
        moves: &mut MoveList,
        attack_fn: impl Fn(Square, Bitboard) -> Bitboard,
    ) {
        for src in pieces {
            for dest in attack_fn(src, self.occupancies()) & destinations {
                moves.push(Move::new(src, dest, MoveType::Normal, self.piece_at(src)));
            }
        }
    }

    fn jumper_moves(
        &self,
        pieces: Bitboard,
        destinations: Bitboard,
        moves: &mut MoveList,
        attack_fn: impl Fn(Square) -> Bitboard,
    ) {
        for src in pieces {
            for dest in attack_fn(src) & destinations {
                moves.push(Move::new(src, dest, MoveType::Normal, self.piece_at(src)));
            }
        }
    }
}

fn gen_promotions(piece: Piece, src: Square, dest: Square, moves: &mut MoveList) {
    const PROMOS: [MoveType; 4] =
        [MoveType::QueenPromotion, MoveType::RookPromotion, MoveType::BishopPromotion, MoveType::KnightPromotion];
    for promo in PROMOS {
        moves.push(Move::new(src, dest, promo, piece));
    }
}
