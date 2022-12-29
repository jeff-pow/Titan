/** File takes a string in Forsyth-Edwards notation and constructs a board state */
use crate::pieces::{Piece, Color, PieceName};
use crate::board::Board;

pub const STARTING_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

pub fn build_board(fen_string: &str) -> Board {
    let mut board = Board::new();
    let mut row = 7;
    let pieces: Vec<&str> = fen_string.split(['/', ' ']).collect();
    // FEN strings have 13 entries
    let mut iter = pieces.iter();
    let mut start = 7;
    let end = 0;
    let step = -1;
    while start >= end { // Loop handles reading board part of fen string
        let entry = iter.next().unwrap();
        for c in entry.chars() {
            let mut idx: usize = 0;
            if c.is_ascii_digit() {
                idx += c.to_digit(10).unwrap() as usize;
            }
            else { match c {
                'K' => board.board[row * 8 + idx] = Some(Piece::new(Color::White, PieceName::King, (row * 8 + idx) as u8)),
                    'Q' => board.board[row * 8 + idx] = Some(Piece::new(Color::White, PieceName::Queen, (row * 8 + idx) as u8)),
                    'R' => board.board[row * 8 + idx] = Some(Piece::new(Color::White, PieceName::Rook, (row * 8 + idx) as u8)),
                    'B' => board.board[row * 8 + idx] = Some(Piece::new(Color::White, PieceName::Bishop, (row * 8 + idx) as u8)),
                    'N' => board.board[row * 8 + idx] = Some(Piece::new(Color::White, PieceName::Knight, (row * 8 + idx) as u8)),
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
    }
    let to_move = match iter.next().unwrap() {
        "w" => Color::White,
        "b" => Color::Black,
        _ => panic!("invalid turn");
    };

    board
}
