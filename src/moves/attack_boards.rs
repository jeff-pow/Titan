use crate::const_array;

use crate::types::bitboard::Bitboard;
use crate::types::pieces::Color;
use crate::types::square::Square;

const FILE_A_U64: u64 = 0x101010101010101;
const FILE_H_U64: u64 = 0x101010101010101 << 7;

pub const FILE_A: Bitboard = Bitboard(FILE_A_U64);
pub const FILE_B: Bitboard = Bitboard(FILE_A_U64 << 1);
pub const FILE_C: Bitboard = Bitboard(FILE_A_U64 << 2);
pub const FILE_D: Bitboard = Bitboard(FILE_A_U64 << 3);
pub const FILE_E: Bitboard = Bitboard(FILE_A_U64 << 4);
pub const FILE_F: Bitboard = Bitboard(FILE_A_U64 << 5);
pub const FILE_G: Bitboard = Bitboard(FILE_A_U64 << 6);
pub const FILE_H: Bitboard = Bitboard(FILE_A_U64 << 7);

pub const FILES: [Bitboard; 8] = const_array!(|f, 8| Bitboard(FILE_A_U64 << f));

const RANK1_U64: u64 = 0b11111111;

pub const RANK1: Bitboard = Bitboard(0b11111111);
pub const RANK2: Bitboard = Bitboard(RANK1_U64 << 8);
pub const RANK3: Bitboard = Bitboard(RANK1_U64 << 16);
pub const RANK4: Bitboard = Bitboard(RANK1_U64 << 24);
pub const RANK5: Bitboard = Bitboard(RANK1_U64 << 32);
pub const RANK6: Bitboard = Bitboard(RANK1_U64 << 40);
pub const RANK7: Bitboard = Bitboard(RANK1_U64 << 48);
pub const RANK8: Bitboard = Bitboard(RANK1_U64 << 56);

pub const RANKS: [Bitboard; 8] = const_array!(|p, 8| Bitboard(RANK1_U64 << (8 * p)));

pub fn knight_attacks(sq: Square) -> Bitboard {
    KNIGHT_ATTACKS[sq]
}

pub fn king_attacks(sq: Square) -> Bitboard {
    KING_ATTACKS[sq]
}

pub fn pawn_attacks(square: Square, attacker: Color) -> Bitboard {
    PAWN_ATTACKS[attacker][square]
}

pub const fn pawn_set_attacks(pawns: Bitboard, side: Color) -> Bitboard {
    let pawns = pawns.0;
    if side.idx() == Color::White.idx() {
        Bitboard((pawns & !FILE_A_U64) << 7 | (pawns & !FILE_H_U64) << 9)
    } else {
        Bitboard((pawns & !FILE_A_U64) >> 9 | (pawns & !FILE_H_U64) >> 7)
    }
}

pub const KING_ATTACKS: [Bitboard; 64] = const_array!(|sq, 64| {
    let sq = 1 << sq;
    // Create a bitboard out of the square
    let mut bb = sq;
    // Put in the bits above and below - These won't have any effect if they are outside of the range
    // of the board
    bb |= sq << 8 | sq >> 8;
    // Then literally shake your column of bits back and forth to get diagonals and horizontal moves
    bb |= (bb & !FILE_A_U64) >> 1 | (bb & !FILE_H_U64) << 1;
    // Remove the square the piece is currently on from possible attacks
    Bitboard(bb ^ sq)
});

pub const KNIGHT_ATTACKS: [Bitboard; 64] = const_array!(|sq, 64| {
    let sq = 1 << sq;
    let mut bb = sq;
    // Get squares two rows above and below current occupied square
    let vert = sq << 16 | sq >> 16;
    // Shake those bits back and forth as long as it wouldn't end up in another row
    bb |= (vert & !FILE_A_U64) >> 1 | (vert & !FILE_H_U64) << 1;
    // Get squares two columns to the left and right of current occupied square. Constants ensure you
    // won't go to a different row
    let horizontal = (sq & 0x3f3f3f3f3f3f3f3f) << 2 | (sq & 0xfcfcfcfcfcfcfcfc) >> 2;
    // Shake those bits back and forth - can't go out of bounds vertically
    bb |= horizontal << 8 | horizontal >> 8;
    // Remove current occupied square from final attack board
    Bitboard(bb ^ sq)
});

pub const PAWN_ATTACKS: [[Bitboard; 64]; 2] = [
    const_array!(|sq, 64| pawn_set_attacks(Bitboard(1 << sq), Color::White)),
    const_array!(|sq, 64| pawn_set_attacks(Bitboard(1 << sq), Color::Black)),
];

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
        moves::attack_boards::pawn_attacks,
        types::{pieces::Color, square::Square},
    };

    #[test]
    fn test_pawn_attacks() {
        let p_sq = Square(40);
        assert_eq!(pawn_attacks(p_sq, Color::Black), Square(33).bitboard());
        assert_eq!(pawn_attacks(p_sq, Color::White), Square(49).bitboard());

        let p_sq = Square(19);
        assert_eq!(
            pawn_attacks(p_sq, Color::Black),
            (Square(10).bitboard() | Square(12).bitboard())
        );
        assert_eq!(
            pawn_attacks(p_sq, Color::White),
            (Square(26).bitboard() | Square(28).bitboard())
        );
    }
}
