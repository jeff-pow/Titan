use std::num::NonZeroI8;

use crate::{Board::Board, Pieces::Color, Pieces::PieceName, Pieces::Piece};

pub struct Move {
    pub starting_idx: u8,
    pub end_idx: u8,
    pub castle_starting_idx: u8,
    pub castle_ending_idx: u8,
}

// Cardinal directions from the point of view of white side
enum Direction {
    North = 8,
    NorthWest = 7,
    West = -1,
    SouthWest = -9,
    South = -8,
    SouthEast = -7,
    East = 1,
    NorthEast = 9,
}

fn generate_all_moves(board: &Board, color_to_move: Color) -> Vec<Move> {
    let mut moves: Vec<Move> = Vec::new();
    for piece in board.board {
        match piece {
            None => continue,
            Some(piece) => {
                if piece.color == color_to_move {
                    let mut vec = generate_moves_for_piece(board, &piece);
                    moves.append(&mut vec);
                }
                else {
                    continue;
                }
            }
        }
    }
    moves
}

fn generate_moves_for_piece(board: &Board, piece: &Piece) -> Vec<Move> {
    match piece.piece_name {
        PieceName::King => return generate_king_moves(board, piece),
        PieceName::Queen => return generate_queen_moves(board, piece),
        PieceName::Rook => return generate_rook_moves(board, piece),
        PieceName::Bishop => return generate_bishop_moves(board, piece),
        PieceName::Knight => return generate_knight_moves(board, piece),
        PieceName::Pawn => return generate_pawn_moves(board, piece),
    }
}

fn generate_king_moves(board: &Board, piece: &Piece) -> Vec<Move> {
    let mut moves: Vec<Move> = Vec::new();
    // Generate moves for castling
    match piece.color {
        Color::White => {
            if board.white_queen_castle && board.board[3] == None && board.board[2] == None && 
                board.board[1] == None {
                    moves.push(Move {
                        starting_idx: 4,
                        end_idx: 2,
                        castle_starting_idx: 0,
                        castle_ending_idx: 3,
                    });
            }
            if board.white_king_castle && board.board[5] == None && board.board[6] == None {
                moves.push(Move {
                    starting_idx: 4,
                    end_idx: 6,
                    castle_starting_idx: 7,
                    castle_ending_idx: 5,
                });
            }
        }
        Color::Black => {
            if board.black_queen_castle && board.board[57] == None && board.board[58] == None && 
                board.board[59] == None {
                    moves.push(Move {
                        starting_idx: 60,
                        end_idx: 58,
                        castle_starting_idx: 56,
                        castle_ending_idx: 59,
                    });
            }
            if board.black_king_castle && board.board[61] == None && board.board[62] == None {
                moves.push(Move {
                    starting_idx: 60, 
                    end_idx: 62,
                    castle_starting_idx: 63,
                    castle_ending_idx: 61,
                });
            }
        }
    }
    let current_idx = piece.current_square;
    for direction in Direction.iter() {
        
    }
    moves
}

fn generate_queen_moves(board: &Board, piece: &Piece) -> Vec<Move> {

    let mut moves: Vec<Move> = Vec::new();
    moves
}

fn generate_rook_moves(board: &Board, piece: &Piece) -> Vec<Move> {

    let mut moves: Vec<Move> = Vec::new();
    moves
}

fn generate_bishop_moves(board: &Board, piece: &Piece) -> Vec<Move> {

    let mut moves: Vec<Move> = Vec::new();
    moves
}

fn generate_knight_moves(board: &Board, piece: &Piece) -> Vec<Move> {

    let mut moves: Vec<Move> = Vec::new();
    moves
}

fn generate_pawn_moves(board: &Board, piece: &Piece) -> Vec<Move> {

    let mut moves: Vec<Move> = Vec::new();
    moves
}

