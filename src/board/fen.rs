use crate::{
    moves::moves::Castle,
    types::{
        pieces::{Color, PieceName},
        square::Square,
    },
};

use super::board::Board;

/// Fen string for the starting position of a board
pub const STARTING_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

/// Takes in a string in fen notation and returns a board state
pub fn build_board(fen_string: &str) -> Board {
    let mut board = Board::default();
    let mut row = 7;
    let pieces: Vec<&str> = fen_string.split(['/', ' ']).collect();
    // FEN strings have 13 entries (if each slash and each space delimit an entry)
    let mut iter = pieces.iter();
    let mut start = 7;
    let end = 0;
    let step: i32 = -1;
    while start >= end {
        // Loop handles reading board part of fen string
        let entry = iter.next().unwrap();
        let mut idx: usize = 0;
        for c in entry.chars() {
            if c.is_ascii_digit() {
                idx += c.to_digit(10).unwrap() as usize;
                continue;
            }
            let square = row * 8 + idx;
            let square = Square(square as u8);
            match c {
                'K' => {
                    board.place_piece(PieceName::King, Color::White, square);
                }
                'Q' => {
                    board.place_piece(PieceName::Queen, Color::White, square);
                }
                'R' => {
                    board.place_piece(PieceName::Rook, Color::White, square);
                }
                'N' => {
                    board.place_piece(PieceName::Knight, Color::White, square);
                }
                'B' => {
                    board.place_piece(PieceName::Bishop, Color::White, square);
                }
                'P' => {
                    board.place_piece(PieceName::Pawn, Color::White, square);
                }
                'k' => {
                    board.place_piece(PieceName::King, Color::Black, square);
                }
                'q' => {
                    board.place_piece(PieceName::Queen, Color::Black, square);
                }
                'r' => {
                    board.place_piece(PieceName::Rook, Color::Black, square);
                }
                'b' => {
                    board.place_piece(PieceName::Bishop, Color::Black, square);
                }
                'n' => {
                    board.place_piece(PieceName::Knight, Color::Black, square);
                }
                'p' => {
                    board.place_piece(PieceName::Pawn, Color::Black, square);
                }
                _ => panic!("Unrecognized char {}, board could not be made", c),
            }
            idx += 1;
        }
        start += step;
        row = row.saturating_sub(1);
    }
    // 9th element: find who's turn it is to move
    board.to_move = match iter.next().unwrap().chars().next().unwrap() {
        'w' => Color::White,
        'b' => Color::Black,
        _ => panic!("invalid turn"),
    };

    // 10th bucket find who can still castle
    // Order of array is white king castle, white queen castle, black king castle, black queen castle
    board.castling_rights = parse_castling(iter.next().unwrap());

    let en_passant_letters: Vec<char> = iter.next().unwrap().chars().collect();
    let en_passant_idx = find_en_passant_square(en_passant_letters);
    if let Some(idx) = en_passant_idx {
        board.en_passant_square = Some(Square(idx))
    }

    let half_moves = iter.next();
    if let Some(half_moves) = half_moves {
        if let Ok(half_moves) = half_moves.parse() {
            board.half_moves = half_moves;
        }
    }

    // Full number of moves in the game: starts from 1 and incremented after black's first move
    let full_moves = iter.next();
    if let Some(full_moves) = full_moves {
        if let Ok(full_moves) = full_moves.parse() {
            board.num_moves = full_moves;
        }
    }
    assert_eq!(iter.next(), None);
    board.zobrist_hash = board.generate_hash();
    board.refresh_accumulators();
    board
}

fn parse_castling(buf: &&str) -> u8 {
    let rights = buf.chars().fold(0, |x, ch| {
        x | match ch {
            'K' => Castle::WhiteKing as u8,
            'Q' => Castle::WhiteQueen as u8,
            'k' => Castle::BlackKing as u8,
            'q' => Castle::BlackQueen as u8,
            _ => 0,
        }
    });
    rights
}

fn find_en_passant_square(vec: Vec<char>) -> Option<u8> {
    if vec[0] == '-' {
        return None;
    }
    // Using base 20 allows program to convert letters directly to numbers instead of matching
    // against letters or some other workaround
    let column = vec[0].to_digit(20).unwrap() - 10;
    let row = (vec[1].to_digit(10).unwrap() - 1) * 8;
    Some((row + column) as u8)
}
#[allow(clippy::ptr_arg)]
pub fn parse_fen_from_buffer(buf: &[&str]) -> String {
    let mut vec = buf.to_owned();
    vec.remove(0);
    vec.remove(0);
    for _ in 6..vec.len() {
        vec.pop();
    }
    vec.join(" ")
}

#[cfg(test)]
mod fen_tests {
    use crate::{
        board::fen::{find_en_passant_square, parse_castling},
        moves::moves::Castle,
    };

    #[test]
    fn test_en_passant_square() {
        assert_eq!(Some(0), find_en_passant_square(vec!['a', '1']));
        assert_eq!(Some(9), find_en_passant_square(vec!['b', '2']));
        assert_eq!(Some(18), find_en_passant_square(vec!['c', '3']));
        assert_eq!(Some(27), find_en_passant_square(vec!['d', '4']));
        assert_eq!(Some(36), find_en_passant_square(vec!['e', '5']));
        assert_eq!(Some(45), find_en_passant_square(vec!['f', '6']));
        assert_eq!(Some(54), find_en_passant_square(vec!['g', '7']));
        assert_eq!(Some(63), find_en_passant_square(vec!['h', '8']));
        assert_eq!(Some(62), find_en_passant_square(vec!['g', '8']));
    }

    #[test]
    fn test_parse_castling_white_king() {
        let input = "K";
        let result = parse_castling(&input);
        assert_eq!(result, Castle::WhiteKing as u8);
    }

    #[test]
    fn test_parse_castling_white_queen() {
        let input = "Q";
        let result = parse_castling(&input);
        assert_eq!(result, Castle::WhiteQueen as u8);
    }

    #[test]
    fn test_parse_castling_black_king() {
        let input = "k";
        let result = parse_castling(&input);
        assert_eq!(result, Castle::BlackKing as u8);
    }

    #[test]
    fn test_parse_castling_black_queen() {
        let input = "q";
        let result = parse_castling(&input);
        assert_eq!(result, Castle::BlackQueen as u8);
    }

    #[test]
    fn test_parse_castling_invalid() {
        let input = "X";
        let result = parse_castling(&input);
        assert_eq!(result, 0); // Expecting 0 for invalid input
    }

    #[test]
    fn test_parse_multiple_castlings() {
        let input = "KQkq";
        let result = parse_castling(&input);
        // You need to define the expected result based on the combination of castling rights.
        // For example, if all castling rights are allowed (KQkq), you can set the expected result to a specific value.
        let expected_result =
            Castle::WhiteKing as u8 | Castle::WhiteQueen as u8 | Castle::BlackKing as u8 | Castle::BlackQueen as u8;
        assert_eq!(result, expected_result);
    }

    #[test]
    fn test_parse_partial_castlings() {
        let input = "Kk";
        let result = parse_castling(&input);
        // Define the expected result for the combination of castling rights in the input.
        let expected_result = Castle::WhiteKing as u8 | Castle::BlackKing as u8;
        assert_eq!(result, expected_result);
    }
}
