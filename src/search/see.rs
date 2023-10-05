use strum::IntoEnumIterator;

use crate::{
    board::board::Board,
    moves::moves::Move,
    types::{
        bitboard::Bitboard,
        pieces::{Color, PieceName},
    },
};

pub fn see(board: &Board, m: &Move, threshold: i32) -> bool {
    let to = m.dest_square();
    let from = m.origin_square();

    let mut val = if let Some(piece) = board.piece_at(to) {
        piece.value() - threshold
    } else {
        -threshold
    };
    if val < 0 {
        return false;
    }

    let piece_moving = board.piece_at(from).expect("There is a piece here");
    val -= piece_moving.value();
    if val >= 0 {
        return true;
    }

    let mut occupied = (board.occupancies() ^ from.bitboard()) | to.bitboard();
    let mut attackers = board.attackers(to, occupied) & occupied;

    let queens = board.bitboard(Color::White, PieceName::Queen) | board.bitboard(Color::Black, PieceName::Queen);
    let bishops =
        board.bitboard(Color::White, PieceName::Bishop) | board.bitboard(Color::Black, PieceName::Bishop) | queens;
    let rooks = board.bitboard(Color::White, PieceName::Rook) | board.bitboard(Color::Black, PieceName::Rook) | queens;

    let mut to_move = board.to_move.opp();

    loop {
        attackers &= occupied;

        let my_attackers = attackers & board.color_occupancies(to_move);

        if my_attackers == Bitboard::EMPTY {
            break;
        }

        let mut most_valuable_piece = PieceName::Pawn;
        for p in PieceName::iter().rev() {
            most_valuable_piece = p;
            if my_attackers & (board.bitboard(Color::White, p) | board.bitboard(Color::Black, p)) == Bitboard::EMPTY {
                break;
            }
        }

        to_move = to_move.opp();
        val = -val - 1 - most_valuable_piece.value();
        if val >= 0 {
            if most_valuable_piece == PieceName::King
                && (attackers & board.color_occupancies(to_move) != Bitboard::EMPTY)
            {
                to_move = to_move.opp();
            }
            break;
        }
        occupied ^= Bitboard(
            1 << (my_attackers
                & (board.bitboard(Color::White, most_valuable_piece)
                    | board.bitboard(Color::Black, most_valuable_piece)))
            .0
            .trailing_zeros(),
        );
        if most_valuable_piece == PieceName::Pawn
            || most_valuable_piece == PieceName::Bishop
            || most_valuable_piece == PieceName::Queen
        {
            attackers |= board.mg.magics.bishop_attacks(occupied, to) & bishops;
        }
        if most_valuable_piece == PieceName::Rook || most_valuable_piece == PieceName::Queen {
            attackers |= board.mg.magics.rook_attacks(occupied, to) & rooks;
        }
    }

    to_move != board.to_move
}
