use lazy_static::lazy_static;

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
    attack_boards::{
        gen_king_attack_boards, gen_knight_attack_boards, gen_pawn_attack_boards, RANK2, RANK3,
        RANK6, RANK7,
    },
    magics::Magics,
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

lazy_static! {
    /// Object that contains the attack boards for each piece from each square
    pub static ref MG: MoveGenerator = MoveGenerator::default();
}

#[derive(Clone)]
pub struct MoveGenerator {
    king_table: [Bitboard; 64],
    knight_table: [Bitboard; 64],
    pawn_table: [[Bitboard; 64]; 2],
    magics: Magics,
}

impl Default for MoveGenerator {
    fn default() -> Self {
        let king_table = gen_king_attack_boards();
        let knight_table = gen_knight_attack_boards();
        let pawn_table = gen_pawn_attack_boards();
        let magics = Magics::default();
        Self { king_table, knight_table, pawn_table, magics }
    }
}

impl MoveGenerator {
    pub fn bishop_attacks(&self, square: Square, occupied: Bitboard) -> Bitboard {
        self.magics.bishop_attacks(occupied, square)
    }

    pub fn rook_attacks(&self, square: Square, occupied: Bitboard) -> Bitboard {
        self.magics.rook_attacks(occupied, square)
    }

    pub fn knight_attacks(&self, square: Square) -> Bitboard {
        self.knight_table[square]
    }

    pub fn king_attacks(&self, square: Square) -> Bitboard {
        self.king_table[square]
    }

    pub fn pawn_attacks(&self, square: Square, attacker: Color) -> Bitboard {
        self.pawn_table[attacker][square]
    }
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

    fn generate_pawn_moves(&self, gen_type: MGT, moves: &mut MoveList) {
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
                moves.push(Move::new(src, dest, MoveType::Normal, PieceName::Pawn));
            }
            for dest in push_two {
                let src = dest.shift(down).shift(down);
                moves.push(Move::new(src, dest, MoveType::DoublePush, PieceName::Pawn));
            }
        }

        // Promotions - captures and straight pushes
        // Always generate all promotions because they are so good
        if promotions != Bitboard::EMPTY {
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
                PieceName::King => MG.king_attacks(sq),
                PieceName::Queen => {
                    MG.magics.rook_attacks(occupancies, sq)
                        | MG.magics.bishop_attacks(occupancies, sq)
                }
                PieceName::Rook => MG.magics.rook_attacks(occupancies, sq),
                PieceName::Bishop => MG.magics.bishop_attacks(occupancies, sq),
                PieceName::Knight => MG.knight_attacks(sq),
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
