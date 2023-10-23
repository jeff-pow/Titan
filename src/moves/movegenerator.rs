use lazy_static::lazy_static;
use strum::IntoEnumIterator;

use crate::{
    board::board::Board,
    moves::{moves::Direction, moves::Direction::*, moves::Promotion},
    types::{
        bitboard::Bitboard,
        pieces::PieceName::*,
        pieces::{Color, PieceName},
        square::Square,
    },
};

use super::{
    attack_boards::{
        gen_king_attack_boards, gen_knight_attack_boards, gen_pawn_attack_boards, RANK2, RANK3, RANK6, RANK7,
    },
    magics::Magics,
    movelist::MoveList,
    moves::{Castle, Move, MoveType},
};

pub const WHITE_KINGSIDE_SQUARES: Bitboard = Bitboard(0b1100000);
pub const WHITE_QUEENSIDE_SQUARES: Bitboard = Bitboard(0b1110);
pub const BLACK_KINGSIDE_SQUARES: Bitboard = Bitboard(0x6000000000000000);
pub const BLACK_QUEENSIDE_SQUARES: Bitboard = Bitboard(0xe00000000000000);

#[allow(clippy::upper_case_acronyms)]
pub type MGT = MoveGenerationType;
#[derive(Copy, Clone, PartialEq)]
pub enum MoveGenerationType {
    CapturesOnly,
    QuietsOnly,
    All,
}

lazy_static! {
    pub static ref MOVEGENERATOR: MoveGenerator = MoveGenerator::default();
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
        Self {
            king_table,
            knight_table,
            pawn_table,
            magics,
        }
    }
}

impl MoveGenerator {
    #[inline(always)]
    pub fn bishop_attacks(&self, square: Square, occupied: Bitboard) -> Bitboard {
        self.magics.bishop_attacks(occupied, square)
    }

    #[inline(always)]
    pub fn rook_attacks(&self, square: Square, occupied: Bitboard) -> Bitboard {
        self.magics.rook_attacks(occupied, square)
    }

    pub fn knight_attacks(&self, square: Square) -> Bitboard {
        self.knight_table[square.idx()]
    }

    pub fn king_attacks(&self, square: Square) -> Bitboard {
        self.king_table[square.idx()]
    }

    pub fn pawn_attacks(&self, square: Square, attacker: Color) -> Bitboard {
        self.pawn_table[attacker.idx()][square.idx()]
    }
}

/// Generates all moves with no respect to legality via leaving itself in check
pub fn generate_psuedolegal_moves(board: &Board, gen_type: MGT) -> MoveList {
    let mut moves = MoveList::default();
    moves.append(&generate_bitboard_moves(board, PieceName::Knight, gen_type));
    moves.append(&generate_bitboard_moves(board, PieceName::King, gen_type));
    moves.append(&generate_bitboard_moves(board, PieceName::Queen, gen_type));
    moves.append(&generate_bitboard_moves(board, PieceName::Rook, gen_type));
    moves.append(&generate_bitboard_moves(board, PieceName::Bishop, gen_type));
    moves.append(&generate_pawn_moves(board, gen_type));
    if gen_type == MGT::QuietsOnly || gen_type == MGT::All {
        moves.append(&generate_castling_moves(board));
    }
    moves
}

fn generate_castling_moves(board: &Board) -> MoveList {
    let mut moves = MoveList::default();
    let (kingside_vacancies, queenside_vacancies) = match board.to_move {
        Color::White => (WHITE_KINGSIDE_SQUARES, WHITE_QUEENSIDE_SQUARES),
        Color::Black => (BLACK_KINGSIDE_SQUARES, BLACK_QUEENSIDE_SQUARES),
    };
    let (can_kingside, can_queenside) = match board.to_move {
        Color::White => (board.castling(Castle::WhiteKing), board.castling(Castle::WhiteQueen)),
        Color::Black => (board.castling(Castle::BlackKing), board.castling(Castle::BlackQueen)),
    };
    let (kingside_dest, queenside_dest) = match board.to_move {
        Color::White => (Square(6), Square(2)),
        Color::Black => (Square(62), Square(58)),
    };
    let king_sq = board.king_square(board.to_move);
    'kingside: {
        if can_kingside && (kingside_vacancies & board.occupancies()) == Bitboard::EMPTY {
            let range = match board.to_move {
                Color::White => 4..=6,
                Color::Black => 60..=62,
            };
            for check_sq in range {
                if board.square_under_attack(!board.to_move, Square(check_sq)) {
                    break 'kingside;
                }
            }
            moves.push(Move::new(king_sq, kingside_dest, None, MoveType::Castle));
        }
    }
    'queenside: {
        if can_queenside && (queenside_vacancies & board.occupancies()) == Bitboard::EMPTY {
            let range = match board.to_move {
                Color::White => 2..=4,
                Color::Black => 58..=60,
            };
            for check_sq in range {
                if board.square_under_attack(!board.to_move, Square(check_sq)) {
                    break 'queenside;
                }
            }
            moves.push(Move::new(king_sq, queenside_dest, None, MoveType::Castle));
        }
    }
    moves
}

fn generate_pawn_moves(board: &Board, gen_type: MGT) -> MoveList {
    let mut moves = MoveList::default();
    let pawns = board.bitboard(board.to_move, Pawn);
    let vacancies = !board.occupancies();
    let enemies = board.color_occupancies(!board.to_move);
    let non_promotions = match board.to_move {
        Color::White => pawns & !RANK7,
        Color::Black => pawns & !RANK2,
    };
    let promotions = match board.to_move {
        Color::White => pawns & RANK7,
        Color::Black => pawns & RANK2,
    };

    let up = match board.to_move {
        Color::White => North,
        Color::Black => South,
    };
    let down = up.opp();

    let up_left = match board.to_move {
        Color::White => NorthWest,
        Color::Black => SouthEast,
    };
    let down_right = up_left.opp();

    let up_right = match board.to_move {
        Color::White => NorthEast,
        Color::Black => SouthWest,
    };
    let down_left = up_right.opp();

    let rank3_bb = match board.to_move {
        Color::White => RANK3,
        Color::Black => RANK6,
    };

    if matches!(gen_type, MGT::All | MGT::QuietsOnly) {
        // Single and double pawn pushes w/o captures
        let push_one = vacancies & non_promotions.shift(up);
        let push_two = vacancies & (push_one & rank3_bb).shift(up);
        for dest in push_one {
            let src = dest.checked_shift(down).expect("Valid shift");
            moves.push(Move::new(src, dest, None, MoveType::Normal));
        }
        for dest in push_two {
            let src = dest
                .checked_shift(down)
                .expect("Valid shift")
                .checked_shift(down)
                .expect("Valid shift");
            moves.push(Move::new(src, dest, None, MoveType::Normal));
        }
    }

    // Promotions - captures and straight pushes
    if promotions != Bitboard::EMPTY {
        let no_capture_promotions = promotions.shift(up) & vacancies;
        let left_capture_promotions = promotions.shift(up_left) & enemies;
        let right_capture_promotions = promotions.shift(up_right) & enemies;
        if matches!(gen_type, MGT::All | MGT::QuietsOnly) {
            for dest in no_capture_promotions {
                generate_promotions(dest, down, &mut moves);
            }
        }
        if matches!(gen_type, MGT::All | MGT::CapturesOnly) {
            for dest in left_capture_promotions {
                generate_promotions(dest, down_right, &mut moves);
            }
            for dest in right_capture_promotions {
                generate_promotions(dest, down_left, &mut moves);
            }
        }
    }

    if matches!(gen_type, MGT::All | MGT::CapturesOnly) {
        // Captures that do not lead to promotions
        if non_promotions != Bitboard::EMPTY {
            let mut left_captures = non_promotions.shift(up_left) & enemies;
            let mut right_captures = non_promotions.shift(up_right) & enemies;
            while left_captures > Bitboard::EMPTY {
                let dest = left_captures.pop_lsb();
                let src = dest.checked_shift(down_right).expect("Valid shift");
                moves.push(Move::new(src, dest, None, MoveType::Normal));
            }
            while right_captures > Bitboard::EMPTY {
                let dest = right_captures.pop_lsb();
                let src = dest.checked_shift(down_left).expect("Valid shift");
                moves.push(Move::new(src, dest, None, MoveType::Normal));
            }
        }

        // En Passant
        if board.can_en_passant() {
            if let Some(x) = get_en_passant(board, down_right) {
                moves.push(x)
            }
            if let Some(x) = get_en_passant(board, down_left) {
                moves.push(x)
            }
        }
    }

    moves
}

pub fn get_en_passant(board: &Board, dir: Direction) -> Option<Move> {
    let sq = board.en_passant_square?.checked_shift(dir)?;
    let pawn = sq.bitboard() & board.bitboard(board.to_move, Pawn);
    if pawn != Bitboard::EMPTY {
        let dest = board.en_passant_square?;
        let src = dest.checked_shift(dir)?;
        return Some(Move::new(src, dest, None, MoveType::EnPassant));
    }
    None
}

fn generate_promotions(dest: Square, d: Direction, moves: &mut MoveList) {
    for p in Promotion::iter() {
        moves.push(Move::new(dest.checked_shift(d).unwrap(), dest, Some(p), MoveType::Promotion));
    }
}

fn generate_bitboard_moves(board: &Board, piece_name: PieceName, gen_type: MGT) -> MoveList {
    let mut moves = MoveList::default();
    // Don't calculate any moves if no pieces of that type exist for the given color
    let occ_bitboard = board.bitboard(board.to_move, piece_name);
    for sq in occ_bitboard {
        let occupancies = board.occupancies();
        let attack_bitboard = match piece_name {
            King => MOVEGENERATOR.king_attacks(sq),
            Queen => {
                MOVEGENERATOR.magics.rook_attacks(occupancies, sq)
                    | MOVEGENERATOR.magics.bishop_attacks(occupancies, sq)
            }
            Rook => MOVEGENERATOR.magics.rook_attacks(occupancies, sq),
            Bishop => MOVEGENERATOR.magics.bishop_attacks(occupancies, sq),
            Knight => MOVEGENERATOR.knight_attacks(sq),
            Pawn => panic!(),
        };
        let enemies_and_vacancies = !board.color_occupancies(board.to_move);
        let attacks = match gen_type {
            MoveGenerationType::CapturesOnly => attack_bitboard & board.color_occupancies(!board.to_move),
            MoveGenerationType::QuietsOnly => attack_bitboard & !board.occupancies(),
            MoveGenerationType::All => attack_bitboard & enemies_and_vacancies,
        };
        // let attacks = attack_bitboard & enemies_and_vacancies;
        for dest in attacks {
            moves.push(Move::new(sq, dest, None, MoveType::Normal));
        }
    }
    moves
}

/// Returns all legal moves
pub fn generate_moves(board: &Board) -> MoveList {
    generate_psuedolegal_moves(board, MGT::All)
        .into_iter()
        .filter(|m| {
            let mut new_b = board.to_owned();
            new_b.make_move(m.m);
            !new_b.in_check(board.to_move)
        })
        .collect()
}
