use crate::{
    board::board::Board,
    moves::{moves::Direction, moves::Direction::*},
    types::{
        bitboard::Bitboard,
        pieces::{Color, PieceName},
        square::Square,
    },
};

use super::{
    attack_boards::{king_attacks, knight_attacks, RANKS},
    magics::{bishop_attacks, rook_attacks},
    movelist::MoveList,
    moves::{Castle, Move, MoveType},
};

#[allow(clippy::upper_case_acronyms)]
pub type MGT = MoveGenerationType;
#[derive(Copy, Clone, PartialEq)]
pub enum MoveGenerationType {
    CapturesOnly,
    QuietsOnly,
    All,
}

impl Board {
    /// Generates all pseudolegal moves
    #[must_use]
    pub fn generate_moves(&self, gen_type: MGT) -> MoveList {
        let mut moves = MoveList::default();
        self.generate_bitboard_moves(PieceName::Knight, gen_type, &mut moves);
        self.generate_bitboard_moves(PieceName::King, gen_type, &mut moves);
        self.generate_bitboard_moves(PieceName::Queen, gen_type, &mut moves);
        self.generate_bitboard_moves(PieceName::Rook, gen_type, &mut moves);
        self.generate_bitboard_moves(PieceName::Bishop, gen_type, &mut moves);
        self.generate_pawn_moves(gen_type, &mut moves);
        if gen_type == MGT::QuietsOnly || gen_type == MGT::All {
            self.generate_castling_moves(&mut moves);
        }
        moves
    }

    pub(crate) fn generate_castling_moves(&self, moves: &mut MoveList) {
        if self.to_move == Color::White {
            if self.can_castle(Castle::WhiteKing)
                && self.occupancies().empty(Square(5))
                && self.occupancies().empty(Square(6))
                && !self.square_under_attack(Color::Black, Square(4))
                && !self.square_under_attack(Color::Black, Square(5))
                && !self.square_under_attack(Color::Black, Square(6))
            {
                moves.push(Move::new(Square(4), Square(6), MoveType::CastleMove, PieceName::King));
            }

            if self.can_castle(Castle::WhiteQueen)
                && self.occupancies().empty(Square(1))
                && self.occupancies().empty(Square(2))
                && self.occupancies().empty(Square(3))
                && !self.square_under_attack(Color::Black, Square(2))
                && !self.square_under_attack(Color::Black, Square(3))
                && !self.square_under_attack(Color::Black, Square(4))
            {
                moves.push(Move::new(Square(4), Square(2), MoveType::CastleMove, PieceName::King));
            }
        } else {
            if self.can_castle(Castle::BlackKing)
                && self.occupancies().empty(Square(61))
                && self.occupancies().empty(Square(62))
                && !self.square_under_attack(Color::White, Square(60))
                && !self.square_under_attack(Color::White, Square(61))
                && !self.square_under_attack(Color::White, Square(62))
            {
                moves.push(Move::new(
                    Square(60),
                    Square(62),
                    MoveType::CastleMove,
                    PieceName::King,
                ));
            }

            if self.can_castle(Castle::BlackQueen)
                && self.occupancies().empty(Square(57))
                && self.occupancies().empty(Square(58))
                && self.occupancies().empty(Square(59))
                && !self.square_under_attack(Color::White, Square(58))
                && !self.square_under_attack(Color::White, Square(59))
                && !self.square_under_attack(Color::White, Square(60))
            {
                moves.push(Move::new(
                    Square(60),
                    Square(58),
                    MoveType::CastleMove,
                    PieceName::King,
                ));
            }
        }
    }

    pub(crate) fn generate_pawn_moves(&self, gen_type: MGT, moves: &mut MoveList) {
        let pawns = self.bitboard(self.to_move, PieceName::Pawn);
        let vacancies = !self.occupancies();
        let enemies = self.color(!self.to_move);
        let non_promotions = match self.to_move {
            Color::White => pawns & !RANKS[6],
            Color::Black => pawns & !RANKS[1],
        };
        let promotions = match self.to_move {
            Color::White => pawns & RANKS[6],
            Color::Black => pawns & RANKS[1],
        };

        let up = match self.to_move {
            Color::White => North,
            Color::Black => South,
        };
        let down = up.opp();

        let up_left = match self.to_move {
            Color::White => NorthWest,
            Color::Black => SouthEast,
        };
        let down_right = up_left.opp();

        let up_right = match self.to_move {
            Color::White => NorthEast,
            Color::Black => SouthWest,
        };
        let down_left = up_right.opp();

        let rank3_bb = match self.to_move {
            Color::White => RANKS[2],
            Color::Black => RANKS[5],
        };

        if matches!(gen_type, MGT::All | MGT::QuietsOnly) {
            // Single and double pawn pushes w/o captures
            let push_one = vacancies & non_promotions.shift(up);
            let push_two = vacancies & (push_one & rank3_bb).shift(up);
            for dest in push_one {
                let src = dest.shift(down);
                moves.push(Move::new(src, dest, MoveType::Normal, PieceName::Pawn));
            }
            for dest in push_two {
                let src = dest.shift(down).shift(down);
                moves.push(Move::new(src, dest, MoveType::DoublePush, PieceName::Pawn));
            }
        }

        // Promotions - captures and straight pushes
        // Promotions are generated with captures because they are so good
        if matches!(gen_type, MGT::All | MGT::CapturesOnly) && promotions != Bitboard::EMPTY {
            let no_capture_promotions = promotions.shift(up) & vacancies;
            let left_capture_promotions = promotions.shift(up_left) & enemies;
            let right_capture_promotions = promotions.shift(up_right) & enemies;
            for dest in no_capture_promotions {
                moves.push(Move::new(
                    dest.shift(down),
                    dest,
                    MoveType::QueenPromotion,
                    PieceName::Pawn,
                ));
                moves.push(Move::new(
                    dest.shift(down),
                    dest,
                    MoveType::RookPromotion,
                    PieceName::Pawn,
                ));
                moves.push(Move::new(
                    dest.shift(down),
                    dest,
                    MoveType::BishopPromotion,
                    PieceName::Pawn,
                ));
                moves.push(Move::new(
                    dest.shift(down),
                    dest,
                    MoveType::KnightPromotion,
                    PieceName::Pawn,
                ));
            }
            for dest in left_capture_promotions {
                moves.push(Move::new(
                    dest.shift(down_right),
                    dest,
                    MoveType::QueenPromotion,
                    PieceName::Pawn,
                ));
                moves.push(Move::new(
                    dest.shift(down_right),
                    dest,
                    MoveType::RookPromotion,
                    PieceName::Pawn,
                ));
                moves.push(Move::new(
                    dest.shift(down_right),
                    dest,
                    MoveType::BishopPromotion,
                    PieceName::Pawn,
                ));
                moves.push(Move::new(
                    dest.shift(down_right),
                    dest,
                    MoveType::KnightPromotion,
                    PieceName::Pawn,
                ));
            }
            for dest in right_capture_promotions {
                moves.push(Move::new(
                    dest.shift(down_left),
                    dest,
                    MoveType::QueenPromotion,
                    PieceName::Pawn,
                ));
                moves.push(Move::new(
                    dest.shift(down_left),
                    dest,
                    MoveType::RookPromotion,
                    PieceName::Pawn,
                ));
                moves.push(Move::new(
                    dest.shift(down_left),
                    dest,
                    MoveType::BishopPromotion,
                    PieceName::Pawn,
                ));
                moves.push(Move::new(
                    dest.shift(down_left),
                    dest,
                    MoveType::KnightPromotion,
                    PieceName::Pawn,
                ));
            }
        }

        if matches!(gen_type, MGT::All | MGT::CapturesOnly) {
            // Captures that do not lead to promotions
            if non_promotions != Bitboard::EMPTY {
                let left_captures = non_promotions.shift(up_left) & enemies;
                let right_captures = non_promotions.shift(up_right) & enemies;
                for dest in left_captures {
                    let src = dest.shift(down_right);
                    moves.push(Move::new(src, dest, MoveType::Normal, PieceName::Pawn));
                }
                for dest in right_captures {
                    let src = dest.shift(down_left);
                    moves.push(Move::new(src, dest, MoveType::Normal, PieceName::Pawn));
                }
            }

            // En Passant
            if self.can_en_passant() {
                if let Some(x) = self.get_en_passant(down_right) {
                    moves.push(x)
                }
                if let Some(x) = self.get_en_passant(down_left) {
                    moves.push(x)
                }
            }
        }
    }

    fn get_en_passant(&self, dir: Direction) -> Option<Move> {
        let sq = self.en_passant_square?.checked_shift(dir)?;
        let pawn = sq.bitboard() & self.bitboard(self.to_move, PieceName::Pawn);
        if pawn != Bitboard::EMPTY {
            let dest = self.en_passant_square?;
            let src = dest.checked_shift(dir)?;
            return Some(Move::new(src, dest, MoveType::EnPassant, PieceName::Pawn));
        }
        None
    }

    fn generate_bitboard_moves(&self, piece_name: PieceName, gen_type: MGT, moves: &mut MoveList) {
        // Don't calculate any moves if no pieces of that type exist for the given color
        let occ_bitself = self.bitboard(self.to_move, piece_name);
        for sq in occ_bitself {
            let occupancies = self.occupancies();
            let attack_bitself = match piece_name {
                PieceName::King => king_attacks(sq),
                PieceName::Queen => rook_attacks(sq, occupancies) | bishop_attacks(sq, occupancies),
                PieceName::Rook => rook_attacks(sq, occupancies),
                PieceName::Bishop => bishop_attacks(sq, occupancies),
                PieceName::Knight => knight_attacks(sq),
                PieceName::Pawn => panic!(),
            };
            let attacks = match gen_type {
                MoveGenerationType::CapturesOnly => attack_bitself & self.color(!self.to_move),
                MoveGenerationType::QuietsOnly => attack_bitself & !self.occupancies(),
                MoveGenerationType::All => attack_bitself & (!self.color(self.to_move)),
            };
            for dest in attacks {
                moves.push(Move::new(sq, dest, MoveType::Normal, piece_name));
            }
        }
    }
}
