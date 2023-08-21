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
    attack_boards::{king_attacks, knight_attacks, RANK2, RANK3, RANK6, RANK7},
    magics::{bishop_attacks, rook_attacks},
    movelist::MoveList,
    moves::{Move, MoveType},
};

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
        Color::White => (Bitboard(0b1100000), Bitboard(0b1110)),
        Color::Black => (Bitboard(0x6000000000000000), Bitboard(0xe00000000000000)),
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
    let pawns = board.board[board.to_move as usize][Pawn as usize];
    let vacancies = !board.occupancies();
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
    let enemies = board.color_occupancies(board.to_move.opp());

    // Single and double pawn pushes w/o captures
    let mut push_one = vacancies & non_promotions.shift(up);
    let mut push_two = vacancies & (push_one & rank3_bb).shift(up);
    while push_one != Bitboard::EMPTY {
        let dest = push_one.pop_lsb();
        let src = dest.checked_shift(down).expect("Valid shift");
        moves.push(Move::new(src, dest, None, MoveType::Normal));
    }
    while push_two != Bitboard::EMPTY {
        let dest = push_two.pop_lsb();
        let src = dest
            .checked_shift(down)
            .expect("Valid shift")
            .checked_shift(down)
            .expect("Valid shift");
        moves.push(Move::new(src, dest, None, MoveType::Normal));
    }

    // Promotions - captures and straight pushes
    if promotions != Bitboard::EMPTY {
        let mut no_capture_promotions = promotions.shift(up) & vacancies;
        let mut left_capture_promotions = promotions.shift(up_left) & enemies;
        let mut right_capture_promotions = promotions.shift(up_right) & enemies;
        while no_capture_promotions != Bitboard::EMPTY {
            generate_promotions(no_capture_promotions.pop_lsb(), down, &mut moves);
        }
        while left_capture_promotions != Bitboard::EMPTY {
            generate_promotions(left_capture_promotions.pop_lsb(), down_right, &mut moves);
        }
        while right_capture_promotions != Bitboard::EMPTY {
            generate_promotions(right_capture_promotions.pop_lsb(), down_left, &mut moves);
        }
    }

    if non_promotions != Bitboard::EMPTY {
        // Captures
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

fn get_en_passant(board: &Board, dir: Direction) -> Option<Move> {
    let sq = board.en_passant_square.checked_shift(dir)?;
    let pawn = sq.bitboard() & board.board[board.to_move as usize][Pawn as usize];
    if pawn != Bitboard::EMPTY {
        let dest = board.en_passant_square;
        let src = dest.checked_shift(dir)?;
        return Some(Move::new(src, dest, None, MoveType::EnPassant));
    }
    None
}

fn generate_promotions(dest: Square, d: Direction, moves: &mut MoveList) {
    for p in Promotion::iter() {
        moves.push(Move::new(
            dest.checked_shift(d).unwrap(),
            dest,
            Some(p),
            MoveType::Promotion,
        ));
    }
}

fn generate_bitboard_moves(board: &Board, piece_name: PieceName) -> MoveList {
    let mut moves = MoveList::default();
    // Don't calculate any moves if no pieces of that type exist for the given color
    if board.board[board.to_move as usize][piece_name as usize] == Bitboard::EMPTY {
        return moves;
    }
    for square in Square::iter() {
        if board.square_contains_piece(piece_name, board.to_move, square) {
            let occupancies = board.occupancies();
            let attack_bitboard = match piece_name {
                King => king_attacks(square),
                Queen => Bitboard(
                    rook_attacks(occupancies.0, square.0) | bishop_attacks(occupancies.0, square.0),
                ),
                Rook => Bitboard(rook_attacks(occupancies.0, square.0)),
                Bishop => Bitboard(bishop_attacks(occupancies.0, square.0)),
                Knight => knight_attacks(square),
                Pawn => panic!(),
            };
            // Tells the program that out of the selected attack squares, the piece can move to
            // empty ones or ones where an enemy piece is
            let enemies_and_vacancies = !board.color_occupancies(board.to_move);
            let attacks = attack_bitboard & enemies_and_vacancies;
            push_moves(&mut moves, attacks, square);
        }
    }
    moves
}

fn push_moves(moves: &mut MoveList, mut attacks: Bitboard, sq: Square) {
    let mut idx = 0;
    while attacks != Bitboard::EMPTY {
        if attacks & Bitboard(1) != Bitboard::EMPTY {
            moves.push(Move::new(sq, Square(idx), None, MoveType::Normal));
        }
        attacks = attacks >> Bitboard(1);
        idx += 1;
    }
}

/// Filters out moves that are silent for quiescence search
pub fn generate_psuedolegal_captures(board: &Board) -> MoveList {
    let legal_moves = generate_psuedolegal_moves(board);
    legal_moves
        .iter()
        .filter(|m| board.occupancies().square_is_occupied(m.dest_square()))
        .collect::<MoveList>()
}

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
        init,
    };

    #[test]
    fn test_starting_pos() {
        init();
        let board = build_board(fen::STARTING_FEN);
        assert_eq!(119_060_324, multi_threaded_perft(board, 6));
    }

    #[test]
    fn test_position_2() {
        init();
        let board = build_board("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -");
        assert_eq!(193_690_690, multi_threaded_perft(board, 5));
    }

    #[test]
    fn test_position_3() {
        init();
        let board = build_board("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - -");
        assert_eq!(11_030_083, multi_threaded_perft(board, 6));
    }

    #[test]
    fn test_position_4() {
        init();
        let board = build_board("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1");
        assert_eq!(706_045_033, multi_threaded_perft(board, 6));
    }

    #[test]
    fn test_position_5() {
        init();
        let board = build_board("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8");
        assert_eq!(89_941_194, multi_threaded_perft(board, 5));
    }

    #[test]
    fn test_position_6() {
        init();
        let board =
            build_board("r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10");
        assert_eq!(164_075_551, perft(board, 5));
    }

    #[test]
    fn test_multithread() {
        init();
        let board =
            build_board("r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10");
        assert_eq!(164_075_551, multi_threaded_perft(board, 5));
    }

    // http://www.rocechess.ch/perft.html
    #[test]
    fn test_position_7() {
        init();
        let board = build_board("n1n5/PPPk4/8/8/8/8/4Kppp/5N1N b - - 0 1");
        assert_eq!(71_179_139, multi_threaded_perft(board, 6));
    }
}
