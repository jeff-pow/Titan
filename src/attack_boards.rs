use crate::const_array;

use crate::magics::{bishop_attacks, rook_attacks};
use crate::types::bitboard::Bitboard;
use crate::types::pieces::Color;
use crate::types::square::Square;

const FILE_A_U64: u64 = 0x0101_0101_0101_0101;
const FILE_H_U64: u64 = 0x0101_0101_0101_0101 << 7;

const RANK1_U64: u64 = 0b1111_1111;

/// Vertical
pub const FILES: [Bitboard; 8] = const_array!(|f, 8| Bitboard(FILE_A_U64 << f));
/// Horizontal
pub const RANKS: [Bitboard; 8] = const_array!(|r, 8| Bitboard(RANK1_U64 << (8 * r)));

pub fn knight_attacks(sq: Square) -> Bitboard {
    KNIGHT_ATTACKS[sq]
}

pub fn king_attacks(sq: Square) -> Bitboard {
    KING_ATTACKS[sq]
}

pub fn pawn_attacks(sq: Square, attacker: Color) -> Bitboard {
    PAWN_ATTACKS[attacker][sq]
}

pub const fn pawn_set_attacks(pawns: Bitboard, side: Color) -> Bitboard {
    let pawns = pawns.0;
    if matches!(side, Color::White) {
        Bitboard((pawns & !FILE_A_U64) << 7 | (pawns & !FILE_H_U64) << 9)
    } else {
        Bitboard((pawns & !FILE_A_U64) >> 9 | (pawns & !FILE_H_U64) >> 7)
    }
}

pub const KING_ATTACKS: [Bitboard; 64] = const_array!(|sq, 64| {
    let sq_bb = 1 << sq;
    // Create a bitboard out of the square
    let mut bb = sq_bb;
    // Put in the bits above and below - These won't have any effect if they are outside of the range
    // of the board
    bb |= sq_bb << 8 | sq_bb >> 8;
    // Then literally shake your column of bits back and forth to get diagonals and horizontal moves
    bb |= (bb & !FILE_A_U64) >> 1 | (bb & !FILE_H_U64) << 1;
    // Remove the square the piece is currently on from possible attacks
    Bitboard(bb ^ sq_bb)
});

pub const KNIGHT_ATTACKS: [Bitboard; 64] = const_array!(|sq, 64| {
    let sq_bb = 1 << sq;
    let mut bb = sq_bb;
    // Get squares two rows above and below current occupied square
    let vert = sq_bb << 16 | sq_bb >> 16;
    // Shake those bits back and forth as long as it wouldn't end up in another row
    bb |= (vert & !FILE_A_U64) >> 1 | (vert & !FILE_H_U64) << 1;
    // Get squares two columns to the left and right of current occupied square. Constants ensure you
    // won't go to a different row
    let horizontal = (sq_bb & 0x3f3f_3f3f_3f3f_3f3f) << 2 | (sq_bb & 0xfcfc_fcfc_fcfc_fcfc) >> 2;
    // Shake those bits back and forth - can't go out of bounds vertically
    bb |= horizontal << 8 | horizontal >> 8;
    // Remove current occupied square from final attack board
    Bitboard(bb ^ sq_bb)
});

pub const PAWN_ATTACKS: [[Bitboard; 64]; 2] = [
    const_array!(|sq, 64| pawn_set_attacks(Bitboard(1 << sq), Color::White)),
    const_array!(|sq, 64| pawn_set_attacks(Bitboard(1 << sq), Color::Black)),
];

static BETWEEN: [[Bitboard; 64]; 64] = const_array!(|i, 64| const_array!(|j, 64| {
    let i = Square(i as u32);
    let j = Square(j as u32);

    if rook_attacks(i, Bitboard::EMPTY).contains(j) {
        rook_attacks(i, j.bitboard()).and(rook_attacks(j, i.bitboard()))
    } else if bishop_attacks(i, Bitboard::EMPTY).contains(j) {
        bishop_attacks(i, j.bitboard()).and(bishop_attacks(j, i.bitboard()))
    } else {
        Bitboard::EMPTY
    }
}));

pub fn between(sq1: Square, sq2: Square) -> Bitboard {
    BETWEEN[sq1][sq2]
}

/// Indexed by [king square][pinned piece]
static PINNED_MOVES: [[Bitboard; 64]; 64] = const_array!(|king, 64| const_array!(|pinned, 64| {
    let king = Square(king as u32);
    let pinned = Square(pinned as u32);

    if bishop_attacks(pinned, Bitboard::EMPTY).contains(king) {
        bishop_attacks(king, Bitboard::EMPTY).and(bishop_attacks(pinned, king.bitboard()))
    } else if rook_attacks(pinned, Bitboard::EMPTY).contains(king) {
        rook_attacks(king, Bitboard::EMPTY).and(rook_attacks(pinned, king.bitboard()))
    } else {
        Bitboard::EMPTY
    }
}));

pub fn pinned_moves(king: Square, pinned: Square) -> Bitboard {
    PINNED_MOVES[king][pinned]
}

#[macro_export]
/// Credit for this macro goes to akimbo
macro_rules! const_array {
    (| $i:ident, $size:literal | $($r:tt)+) => {{
        let mut $i = 0;
        let mut res = [{$($r)+}; $size];
        while $i < $size - 1 {
            $i += 1;
            res[$i] = {$($r)+};
        }
        res
    }}
}

#[cfg(test)]
mod test_attack_boards {
    use crate::{
        attack_boards::pawn_attacks,
        types::{pieces::Color, square::Square},
    };

    #[test]
    fn test_pawn_attacks() {
        let p_sq = Square(40);
        assert_eq!(pawn_attacks(p_sq, Color::Black), Square(33).bitboard());
        assert_eq!(pawn_attacks(p_sq, Color::White), Square(49).bitboard());

        let p_sq = Square(19);
        assert_eq!(pawn_attacks(p_sq, Color::Black), (Square(10).bitboard() | Square(12).bitboard()));
        assert_eq!(pawn_attacks(p_sq, Color::White), (Square(26).bitboard() | Square(28).bitboard()));
    }
}
