use crate::{
    board::board::Board,
    moves::{moves::Direction, moves::Direction::*},
    types::{
        bitboard::Bitboard,
        pieces::{Color, Piece, PieceName},
        square::Square,
    },
};

use super::{
    attack_boards::{king_attacks, knight_attacks, RANK2, RANK3, RANK6, RANK7},
    magics::{bishop_attacks, queen_attacks, rook_attacks},
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

    fn generate_castling_moves(&self, moves: &mut MoveList) {
        if self.to_move == Color::White {
            if self.can_castle(Castle::WhiteKing)
                && self.occupancies().empty(Square(5))
                && self.occupancies().empty(Square(6))
                && !self.square_under_attack(Color::Black, Square(4))
                && !self.square_under_attack(Color::Black, Square(5))
                && !self.square_under_attack(Color::Black, Square(6))
            {
                moves.push(Move::new(Square(4), Square(6), MoveType::CastleMove, Piece::WhiteKing));
            }

            if self.can_castle(Castle::WhiteQueen)
                && self.occupancies().empty(Square(1))
                && self.occupancies().empty(Square(2))
                && self.occupancies().empty(Square(3))
                && !self.square_under_attack(Color::Black, Square(2))
                && !self.square_under_attack(Color::Black, Square(3))
                && !self.square_under_attack(Color::Black, Square(4))
            {
                moves.push(Move::new(Square(4), Square(2), MoveType::CastleMove, Piece::WhiteKing));
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
                    Piece::BlackKing,
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
                    Piece::BlackKing,
                ));
            }
        }
    }

    fn generate_pawn_moves(&self, gen_type: MGT, moves: &mut MoveList) {
        let piece = Piece::new(PieceName::Pawn, self.to_move);
        let pawns = self.bitboard(self.to_move, PieceName::Pawn);
        let vacancies = !self.occupancies();
        let enemies = self.color(!self.to_move);
        let non_promotions = match self.to_move {
            Color::White => pawns & !RANK7,
            Color::Black => pawns & !RANK2,
        };
        let promotions = match self.to_move {
            Color::White => pawns & RANK7,
            Color::Black => pawns & RANK2,
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
            Color::White => RANK3,
            Color::Black => RANK6,
        };

        if matches!(gen_type, MGT::All | MGT::QuietsOnly) {
            // Single and double pawn pushes w/o captures
            let push_one = vacancies & non_promotions.shift(up);
            let push_two = vacancies & (push_one & rank3_bb).shift(up);
            for dest in push_one {
                let src = dest.shift(down);
                moves.push(Move::new(src, dest, MoveType::Normal, piece));
            }
            for dest in push_two {
                let src = dest.shift(down).shift(down);
                moves.push(Move::new(src, dest, MoveType::DoublePush, piece));
            }
        }

        // Promotions - captures and straight pushes
        // Always generate all promotions because they are so good
        if promotions != Bitboard::EMPTY {
            let no_capture_promotions = promotions.shift(up) & vacancies;
            let left_capture_promotions = promotions.shift(up_left) & enemies;
            let right_capture_promotions = promotions.shift(up_right) & enemies;
            for dest in no_capture_promotions {
                gen_promotions(piece, dest.shift(down), dest, moves);
            }
            for dest in left_capture_promotions {
                gen_promotions(piece, dest.shift(down_right), dest, moves);
            }
            for dest in right_capture_promotions {
                gen_promotions(piece, dest.shift(down_left), dest, moves);
            }
        }

        if matches!(gen_type, MGT::All | MGT::CapturesOnly) {
            // Captures that do not lead to promotions
            if non_promotions != Bitboard::EMPTY {
                let left_captures = non_promotions.shift(up_left) & enemies;
                let right_captures = non_promotions.shift(up_right) & enemies;
                for dest in left_captures {
                    let src = dest.shift(down_right);
                    moves.push(Move::new(src, dest, MoveType::Normal, piece));
                }
                for dest in right_captures {
                    let src = dest.shift(down_left);
                    moves.push(Move::new(src, dest, MoveType::Normal, piece));
                }
            }

            // En Passant
            if self.can_en_passant() {
                if let Some(x) = self.get_en_passant(down_right, piece) {
                    moves.push(x)
                }
                if let Some(x) = self.get_en_passant(down_left, piece) {
                    moves.push(x)
                }
            }
        }
    }

    fn get_en_passant(&self, dir: Direction, piece: Piece) -> Option<Move> {
        let sq = self.en_passant_square?.checked_shift(dir)?;
        let pawn = sq.bitboard() & self.bitboard(self.to_move, PieceName::Pawn);
        if pawn != Bitboard::EMPTY {
            let dest = self.en_passant_square?;
            let src = dest.checked_shift(dir)?;
            return Some(Move::new(src, dest, MoveType::EnPassant, piece));
        }
        None
    }

    fn generate_bitboard_moves(&self, piece_name: PieceName, gen_type: MGT, moves: &mut MoveList) {
        // Don't calculate any moves if no pieces of that type exist for the given color
        let occ_bitself = self.bitboard(self.to_move, piece_name);
        let piece_moving = Piece::new(piece_name, self.to_move);
        for sq in occ_bitself {
            let occupancies = self.occupancies();
            let attack_bitself = match piece_name {
                PieceName::King => king_attacks(sq),
                PieceName::Queen => queen_attacks(sq, occupancies),
                PieceName::Rook => rook_attacks(sq, occupancies),
                PieceName::Bishop => bishop_attacks(sq, occupancies),
                PieceName::Knight => knight_attacks(sq),
                _ => panic!(),
            };
            let attacks = match gen_type {
                MoveGenerationType::CapturesOnly => attack_bitself & self.color(!self.to_move),
                MoveGenerationType::QuietsOnly => attack_bitself & !self.occupancies(),
                MoveGenerationType::All => attack_bitself & (!self.color(self.to_move)),
            };
            for dest in attacks {
                moves.push(Move::new(sq, dest, MoveType::Normal, piece_moving));
            }
        }
    }
}

fn gen_promotions(piece: Piece, src: Square, dest: Square, moves: &mut MoveList) {
    const PROMOS: [MoveType; 4] = [
        MoveType::QueenPromotion,
        MoveType::RookPromotion,
        MoveType::BishopPromotion,
        MoveType::KnightPromotion,
    ];
    for promo in PROMOS {
        moves.push(Move::new(src, dest, promo, piece));
    }
}
