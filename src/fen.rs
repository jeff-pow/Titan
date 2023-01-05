/** File takes a string in Forsyth-Edwards notation and constructs a board state */
use crate::pieces::{Piece, Color, PieceName};
use crate::board::Board;

pub const STARTING_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

pub const TEST_FEN: &str = "rnbqkbnr/pp1ppppp/8/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R b KQkq - 1 2";

pub fn build_board(fen_string: &str) -> Board {
    let mut board = Board::new();
    let mut row = 7;
    let pieces: Vec<&str> = fen_string.split(['/', ' ']).collect();
    // FEN strings have 13 entries
    let mut iter = pieces.iter();
    let mut start = 7;
    let end = 0;
    let step: i32 = -1;
    while start >= end { // Loop handles reading board part of fen string
        let entry = iter.next().unwrap();
        let mut idx: usize = 0;
        for c in entry.chars() {
            if c.is_ascii_digit() {
                idx += c.to_digit(10).unwrap() as usize;
                continue;
            }
            else { match c {
                'K' => board.board[row * 8 + idx] = Some(Piece::new(Color::White, PieceName::King, (row * 8 + idx) as u8)),
                'Q' => board.board[row * 8 + idx] = Some(Piece::new(Color::White, PieceName::Queen, (row * 8 + idx) as u8)),
                'R' => board.board[row * 8 + idx] = Some(Piece::new(Color::White, PieceName::Rook, (row * 8 + idx) as u8)),
                'N' => board.board[row * 8 + idx] = Some(Piece::new(Color::White, PieceName::Knight, (row * 8 + idx) as u8)),
                'B' => board.board[row * 8 + idx] = Some(Piece::new(Color::White, PieceName::Bishop, (row * 8 + idx) as u8)),
                'P' => board.board[row * 8 + idx] = Some(Piece::new(Color::White, PieceName::Pawn, (row * 8 + idx) as u8)),
                'k' => board.board[row * 8 + idx] = Some(Piece::new(Color::Black, PieceName::King, (row * 8 + idx) as u8)),
                'q' => board.board[row * 8 + idx] = Some(Piece::new(Color::Black, PieceName::Queen, (row * 8 + idx) as u8)),
                'r' => board.board[row * 8 + idx] = Some(Piece::new(Color::Black, PieceName::Rook, (row * 8 + idx) as u8)),
                'b' => board.board[row * 8 + idx] = Some(Piece::new(Color::Black, PieceName::Bishop, (row * 8 + idx) as u8)),
                'n' => board.board[row * 8 + idx] = Some(Piece::new(Color::Black, PieceName::Knight, (row * 8 + idx) as u8)),
                'p' => board.board[row * 8 + idx] = Some(Piece::new(Color::Black, PieceName::Pawn, (row * 8 + idx) as u8)),
                _ => panic!("Unrecognized char {}, board could not be made", c),
            } }
            idx += 1;
        }
        start += step;
        if row > 0 {
            row -= 1;
        }
    }
    // 9th iteration: find who's turn it is to move
    let to_move = match iter.next().unwrap().chars().next().unwrap() {
        'w' => Color::White,
        'b' => Color::Black,
        _ => panic!("invalid turn"),
    };
    // 10th bucket find who can still castle
    // Order of array is white king castle, white queen castle, black king castle, black queen castle
    let mut castles = [false; 4];
    for c in iter.next().unwrap().chars() {
        match c {
            'K' => castles[0] = true,
            'Q' => castles[1] = true,
            'k' => castles[2] = true,
            'q' => castles[3] = true,
            _ => panic!("Unrecognized castle character: {}", c),
        }
    }
    // En passant square: not yet implemented
    iter.next();
    // Half move clock: not yet implemented
    iter.next();
    // Full number of moves in the game: starts from 1 and incremented after black's first move
    iter.next();
    assert_eq!(iter.next(), None);
    board
}
