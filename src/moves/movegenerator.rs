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
    moves::{Move, MoveType},
};

pub const WHITE_KINGSIDE_SQUARES: Bitboard = Bitboard(0b1100000);
pub const WHITE_QUEENSIDE_SQUARES: Bitboard = Bitboard(0b1110);
pub const BLACK_KINGSIDE_SQUARES: Bitboard = Bitboard(0x6000000000000000);
pub const BLACK_QUEENSIDE_SQUARES: Bitboard = Bitboard(0xe00000000000000);

pub struct MoveGenerator {
    pub king_table: [Bitboard; 64],
    pub knight_table: [Bitboard; 64],
    pub pawn_table: [[Bitboard; 64]; 2],
    pub magics: Magics,
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
    pub fn knight_attacks(&self, square: Square) -> Bitboard {
        self.knight_table[square.0 as usize]
    }

    pub fn king_attacks(&self, square: Square) -> Bitboard {
        self.king_table[square.0 as usize]
    }

    pub fn pawn_attacks(&self, square: Square, attacker: Color) -> Bitboard {
        self.pawn_table[attacker as usize][square.idx()]
    }
}

/// Generates all moves with no respect to legality via leaving itself in check
pub fn generate_psuedolegal_moves(board: &Board) -> MoveList {
    let mut moves = MoveList::default();
    moves.append(&generate_bitboard_moves(board, PieceName::Knight));
    moves.append(&generate_bitboard_moves(board, PieceName::King));
    moves.append(&generate_bitboard_moves(board, PieceName::Queen));
    moves.append(&generate_bitboard_moves(board, PieceName::Rook));
    moves.append(&generate_bitboard_moves(board, PieceName::Bishop));
    moves.append(&generate_pawn_moves(board));
    moves.append(&generate_castling_moves(board));
    moves
}

fn generate_castling_moves(board: &Board) -> MoveList {
    let mut moves = MoveList::default();
    let (kingside_vacancies, queenside_vacancies) = match board.to_move {
        Color::White => (WHITE_KINGSIDE_SQUARES, WHITE_QUEENSIDE_SQUARES),
        Color::Black => (BLACK_KINGSIDE_SQUARES, BLACK_QUEENSIDE_SQUARES),
    };
    let (can_kingside, can_queenside) = match board.to_move {
        Color::White => (board.white_king_castle, board.white_queen_castle),
        Color::Black => (board.black_king_castle, board.black_queen_castle),
    };
    let (kingside_dest, queenside_dest) = match board.to_move {
        Color::White => (Square(6), Square(2)),
        Color::Black => (Square(62), Square(58)),
    };
    let king_sq = match board.to_move {
        Color::White => board.white_king_square,
        Color::Black => board.black_king_square,
    };
    'kingside: {
        if can_kingside && (kingside_vacancies & board.occupancies()) == Bitboard::EMPTY {
            let range = match board.to_move {
                Color::White => 4..=6,
                Color::Black => 60..=62,
            };
            for check_sq in range {
                if board.square_under_attack(board.to_move.opp(), Square(check_sq)) {
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
                if board.square_under_attack(board.to_move.opp(), Square(check_sq)) {
                    break 'queenside;
                }
            }
            moves.push(Move::new(king_sq, queenside_dest, None, MoveType::Castle));
        }
    }
    moves
}

fn generate_pawn_moves(board: &Board) -> MoveList {
    let mut moves = MoveList::default();
    let pawns = board.bitboards[board.to_move as usize][Pawn as usize];
    let vacancies = !board.occupancies();
    let enemies = board.color_occupancies(board.to_move.opp());
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

    // Promotions - captures and straight pushes
    if promotions != Bitboard::EMPTY {
        let no_capture_promotions = promotions.shift(up) & vacancies;
        let left_capture_promotions = promotions.shift(up_left) & enemies;
        let right_capture_promotions = promotions.shift(up_right) & enemies;
        for dest in no_capture_promotions {
            generate_promotions(dest, down, &mut moves);
        }
        for dest in left_capture_promotions {
            generate_promotions(dest, down_right, &mut moves);
        }
        for dest in right_capture_promotions {
            generate_promotions(dest, down_left, &mut moves);
        }
    }

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

    moves
}

pub fn get_en_passant(board: &Board, dir: Direction) -> Option<Move> {
    let sq = board.en_passant_square.checked_shift(dir)?;
    let pawn = sq.bitboard() & board.bitboards[board.to_move as usize][Pawn as usize];
    if pawn != Bitboard::EMPTY {
        let dest = board.en_passant_square;
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

fn generate_bitboard_moves(board: &Board, piece_name: PieceName) -> MoveList {
    let mut moves = MoveList::default();
    // Don't calculate any moves if no pieces of that type exist for the given color
    let occ_bitboard = board.bitboards[board.to_move as usize][piece_name as usize];
    for sq in occ_bitboard {
        let occupancies = board.occupancies();
        let attack_bitboard = match piece_name {
            King => board.mg.king_attacks(sq),
            Queen => board.mg.magics.rook_attacks(occupancies, sq) | board.mg.magics.bishop_attacks(occupancies, sq),
            Rook => board.mg.magics.rook_attacks(occupancies, sq),
            Bishop => board.mg.magics.bishop_attacks(occupancies, sq),
            Knight => board.mg.knight_attacks(sq),
            Pawn => panic!(),
        };
        let enemies_and_vacancies = !board.color_occupancies(board.to_move);
        let attacks = attack_bitboard & enemies_and_vacancies;
        for dest in attacks {
            moves.push(Move::new(sq, dest, None, MoveType::Normal));
        }
    }
    moves
}

/// Filters out moves that are silent for quiescence search
pub fn generate_psuedolegal_captures(board: &Board) -> MoveList {
    let moves = generate_psuedolegal_moves(board);
    moves
        .iter()
        .filter(|m| board.occupancies().square_is_occupied(m.dest_square()))
        .collect::<MoveList>()
}

/// Returns all legal moves
pub fn generate_moves(board: &Board) -> MoveList {
    generate_psuedolegal_moves(board)
        .iter()
        .filter(|m| {
            let mut new_b = board.to_owned();
            new_b.make_move(m);
            !new_b.side_in_check(board.to_move)
        })
        .collect()
}

#[cfg(test)]
mod movegen_tests {
    // Positions and expected values from https://www.chessprogramming.org/Perft_Results
    use crate::{
        board::fen::{self, build_board},
        engine::perft::{multi_threaded_perft, perft},
    };

    #[test]
    fn test_starting_pos() {
        let board = build_board(fen::STARTING_FEN);
        assert_eq!(119_060_324, multi_threaded_perft(board, 6));
    }

    #[test]
    fn test_position_2() {
        let board = build_board("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -");
        assert_eq!(193_690_690, multi_threaded_perft(board, 5));
    }

    #[test]
    fn test_position_3() {
        let board = build_board("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - -");
        assert_eq!(11_030_083, multi_threaded_perft(board, 6));
    }

    #[test]
    fn test_position_4() {
        let board = build_board("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1");
        assert_eq!(706_045_033, multi_threaded_perft(board, 6));
    }

    #[test]
    fn test_position_5() {
        let board = build_board("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8");
        assert_eq!(89_941_194, multi_threaded_perft(board, 5));
    }

    #[test]
    fn test_position_6() {
        let board = build_board("r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10");
        assert_eq!(164_075_551, perft(board, 5));
    }

    #[test]
    fn test_multithread() {
        let board = build_board("r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10");
        assert_eq!(164_075_551, multi_threaded_perft(board, 5));
    }

    // http://www.rocechess.ch/perft.html
    #[test]
    fn test_position_7() {
        let board = build_board("n1n5/PPPk4/8/8/8/8/4Kppp/5N1N b - - 0 1");
        assert_eq!(71_179_139, multi_threaded_perft(board, 6));
    }
}
