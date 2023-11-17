use strum::IntoEnumIterator;

use crate::{
    board::board::Board,
    moves::{movegenerator::MG, moves::Move},
    types::{
        bitboard::Bitboard,
        pieces::{Color, PieceName},
    },
};

impl Board {
    fn gain(&self, m: Move) -> i32 {
        if m.is_castle() {
            return 0;
        }
        if m.is_en_passant() {
            return PieceName::Pawn.value();
        }
        let mut score = if let Some(capture) = self.piece_at(m.dest_square()) {
            capture.value()
        } else {
            0
        };
        if let Some(p) = m.promotion() {
            score += p.value();
            score -= PieceName::Pawn.value()
        }
        score
    }

    fn next_attacker(&self, occupied: &mut Bitboard, attackers: Bitboard, side: Color) -> PieceName {
        for p in PieceName::iter() {
            let bb = attackers & self.bitboard(side, p);
            if bb != Bitboard::EMPTY {
                *occupied ^= bb.get_lsb().bitboard();
                return p;
            }
        }
        unreachable!()
    }

    /// Function that returns true if the side to move of the board would come out on top directly
    /// trading pieces, and false if they would come out behind in piece value
    // Based off implementation in Stormphrax, which got its implementation from Weiss
    pub fn see(&self, m: Move, threshold: i32) -> bool {
        let dest = m.dest_square();
        let src = m.origin_square();

        let mut val = self.gain(m) - threshold;
        if val < 0 {
            return false;
        }

        let piece_moving = self.piece_at(src).expect("There is a piece here");
        val -= piece_moving.value();
        if val >= 0 {
            return true;
        }

        let mut occupied = (self.occupancies() ^ src.bitboard()) | dest.bitboard();
        // if m.is_en_passant() {
        // occupied ^= Square(dest.0 ^ 8).bitboard();
        // }
        let mut attackers = self.attackers(dest, occupied) & occupied;

        let queens = self.bitboard(Color::White, PieceName::Queen) | self.bitboard(Color::Black, PieceName::Queen);
        let bishops =
            self.bitboard(Color::White, PieceName::Bishop) | self.bitboard(Color::Black, PieceName::Bishop) | queens;
        let rooks =
            self.bitboard(Color::White, PieceName::Rook) | self.bitboard(Color::Black, PieceName::Rook) | queens;

        let mut to_move = !self.to_move;

        loop {
            let my_attackers = attackers & self.color_occupancies(to_move);

            if my_attackers == Bitboard::EMPTY {
                break;
            }

            let next_piece = self.next_attacker(&mut occupied, my_attackers, to_move);

            if matches!(next_piece, PieceName::Pawn | PieceName::Bishop | PieceName::Queen) {
                attackers |= MG.bishop_attacks(dest, occupied) & bishops;
            }
            if matches!(next_piece, PieceName::Rook | PieceName::Queen) {
                attackers |= MG.rook_attacks(dest, occupied) & rooks;
            }

            attackers &= occupied;

            to_move = !to_move;
            val = -val - 1 - next_piece.value();
            if val >= 0 {
                if next_piece == PieceName::King && (attackers & self.color_occupancies(to_move) != Bitboard::EMPTY) {
                    to_move = !to_move;
                }
                break;
            }
        }

        to_move != self.to_move
    }
}
