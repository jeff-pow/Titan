use crate::const_array;

use crate::types::bitboard::Bitboard;
use crate::types::pieces::{Color, PieceName};
use crate::types::square::Square;

use super::magics::{bishop_attacks, queen_attacks, rook_attacks};

const FILE_A_U64: u64 = 0x101010101010101;
const FILE_H_U64: u64 = 0x101010101010101 << 7;

const RANK1_U64: u64 = 0b11111111;

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
    if side.idx() == Color::White.idx() {
        Bitboard((pawns & !FILE_A_U64) << 7 | (pawns & !FILE_H_U64) << 9)
    } else {
        Bitboard((pawns & !FILE_A_U64) >> 9 | (pawns & !FILE_H_U64) >> 7)
    }
}

impl PieceName {
    pub(crate) fn attacks(self, side: Color, sq: Square, occupancies: Bitboard) -> Bitboard {
        match self {
            PieceName::Pawn => pawn_attacks(sq, side),
            PieceName::Knight => knight_attacks(sq),
            PieceName::Bishop => bishop_attacks(sq, occupancies),
            PieceName::Rook => rook_attacks(sq, occupancies),
            PieceName::Queen => queen_attacks(sq, occupancies),
            PieceName::King => king_attacks(sq),
            PieceName::None => unreachable!("Illegal piece type: None"),
        }
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

// pub const BETWEEN_SQUARES: [[Bitboard; 64]; 64] = {
//     let mut arr = [[Bitboard::EMPTY; 64]; 64];
//     let mut src = 0;
//     while src < 64 {
//         let mut dest = src + 1;
//         while dest < 64 {
//             if Square(src).rank() == Square(dest).rank() {
//                 // dest > src, so we always want to shift in a smaller direction,
//                 // from dest towards src
//                 let mut i = Square(dest).shift(Direction::West);
//                 while i.0 > src && i.is_valid() {
//                     arr[src as usize][dest as usize].0 |= i.bitboard().0;
//                     i = i.shift(Direction::West);
//                 }
//             } else if Square(src).file() == Square(dest).file() {
//                 let mut i = Square(dest).shift(Direction::South);
//                 while i.0 > src && i.is_valid() {
//                     arr[src as usize][dest as usize].0 |= i.bitboard().0;
//                     i = i.shift(Direction::South);
//                 }
//             } else if (dest - src) % Direction::NorthWest as u32 == 0
//                 && Square(dest).file() < Square(src).file()
//             {
//                 let mut i = Square(dest).shift(Direction::SouthEast);
//
//                 while i.0 > src && i.is_valid() {
//                     arr[src as usize][dest as usize].0 |= i.bitboard().0;
//                     i = i.shift(Direction::SouthEast);
//                 }
//             } else if (dest - src) % Direction::NorthEast as u32 == 0
//                 && Square(dest).file() > Square(src).file()
//             {
//                 let mut i = Square(dest).shift(Direction::SouthWest);
//
//                 while i.0 > src && i.is_valid() {
//                     arr[src as usize][dest as usize].0 |= i.bitboard().0;
//                     i = i.shift(Direction::SouthWest);
//                 }
//             }
//             dest += 1;
//         }
//         src += 1;
//     }
//
//     // Copy top half of the triangle over to the bottom half
//     let mut src = 0;
//     while src < 64 {
//         let mut dest = 0;
//         while dest < src {
//             arr[src][dest] = arr[dest][src];
//             dest += 1;
//         }
//         src += 1;
//     }
//     arr
// };

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
