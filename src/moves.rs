use core::fmt;
use std::fmt::Display;

use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::{board::Board, pieces::Color, pieces::Piece, pieces::PieceName};

pub struct Move {
    pub starting_idx: i8,
    pub end_idx: i8,
    pub castle: Castle,
    pub promotion: bool,
    pub piece_moving: PieceName,
    pub capture: Option<Piece>,
}

impl Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut str = String::new();
        str += "Start: ";
        str += &self.starting_idx.to_string();
        str += " End: ";
        str += &self.end_idx.to_string();
        str += " Castle: ";
        match self.castle {
            Castle::None => str += "No Castle ",
            Castle::WhiteKingCastle => str += "White King castle ",
            Castle::WhiteQueenCastle => str += "white queen castle ",
            Castle::BlackKingCastle => str += "black king castle ",
            Castle::BlackQueenCastle => str += "black queen castle ",
        }
        match self.promotion {
            true => str += " Promotion: true ",
            false => str += " Promotion: false ",
        }
        match self.capture {
            None => {
                str += " Nothing Captured ";
            }
            Some(piece) => match piece.piece_name {
                PieceName::King => str += " Captured a King  ",
                PieceName::Queen => str += " Captured a Queen ",
                PieceName::Rook => str += " Captured a Rook ",
                PieceName::Bishop => str += " Captured a Bishop ",
                PieceName::Knight => str += " Captured a Knight ",
                PieceName::Pawn => str += " Captured a Pawn ",
            },
        }
        match self.piece_moving {
            PieceName::King => str += " King moving ",
            PieceName::Queen => str += " Queen moving ",
            PieceName::Bishop => str += " Bishop moving ",
            PieceName::Rook => str += " Rook moving ",
            PieceName::Knight => str += " Knight moving ",
            PieceName::Pawn => str += " Pawn moving ",
        }
        str += &self.to_lan();
        write!(f, "{}", str)
    }
}

impl Move {
    /// To Long Algebraic Notation
    pub fn to_lan(&self) -> String {
        let mut str = String::new();
        let arr = ["a", "b", "c", "d", "e", "f", "g", "h"];
        let y_origin = self.starting_idx / 8 + 1;
        let x_origin = self.starting_idx % 8;
        let y_end = self.end_idx / 8 + 1;
        let x_end = self.end_idx % 8;
        str += arr[x_origin as usize];
        str += &y_origin.to_string();
        str += arr[x_end as usize];
        str += &y_end.to_string();
        match self.promotion {
            true => str += "q",
            false => {}
        }
        str
    }
}

/// Method converts a lan move provided by UCI framework into a Move struct
pub fn from_lan(str: &str, board: &Board) -> Move {
    let vec: Vec<char> = str.chars().collect();

    // Using base 20 allows program to convert letters directly to numbers instead of matching
    // against letters or some other workaround
    let start_column = vec[0].to_digit(20).unwrap() - 10;
    let start_row = (vec[1].to_digit(10).unwrap() - 1) * 8;
    let starting_idx = (start_row + start_column) as i8;

    let end_column = vec[2].to_digit(20).unwrap() - 10;
    let end_row = (vec[3].to_digit(10).unwrap() - 1) * 8;
    let end_idx = (end_row + end_column) as i8;

    let mut promotion = false;
    let piece = board.board[starting_idx as usize].expect("Piece should be here");
    if piece.piece_name == PieceName::Pawn && is_promotion(&piece, end_idx) {
        promotion = true;
    }
    Move {
        starting_idx,
        end_idx,
        castle: Castle::None,
        promotion,
        piece_moving: board.board[starting_idx as usize].unwrap().piece_name,
        capture: board.board[end_idx as usize],
    }
}

pub enum Castle {
    None,
    WhiteKingCastle,
    WhiteQueenCastle,
    BlackKingCastle,
    BlackQueenCastle,
}

/// Cardinal directions from the point of view of white side
#[derive(EnumIter, Copy, Clone, Debug, PartialEq, Eq)]
#[repr(i8)]
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

/// Takes a direction and number of times that direction is being moved and converts to an x-y tuple
fn convert_idx_to_tuple(d: Direction, repetitions: i8) -> (i8, i8) {
    match d {
        Direction::North => (0, repetitions),
        Direction::NorthWest => (-repetitions, repetitions),
        Direction::West => (-repetitions, 0),
        Direction::SouthWest => (-repetitions, -repetitions),
        Direction::South => (0, -repetitions),
        Direction::SouthEast => (repetitions, -repetitions),
        Direction::East => (repetitions, 0),
        Direction::NorthEast => (repetitions, repetitions),
    }
}

/// Method ensures that two indexes can be added together. Bool determines if operation was
/// successful, and i8 contains the added index if boolean is true and nonsense value if false.
/// x_sum denotes letters and y_sum denotes number of a square on the board
fn check_index_addition(a: i8, b: (i8, i8)) -> (usize, bool) {
    let a_x_component = a % 8;
    let a_y_component = a / 8;
    let b_x_component = b.0;
    let b_y_component = b.1;
    let x_sum = a_x_component + b_x_component;
    let y_sum = a_y_component + b_y_component;
    if !(0..8).contains(&x_sum) || !(0..8).contains(&y_sum) {
        return (0, false);
    }
    let ret = (a + (b.1 * 8) + b.0) as usize;
    (ret, true)
}

/// Method returns a tuple with a bool stating if a piece is on that square and the color of the
/// piece if there is a piece
fn check_space_occupancy(board: &Board, potential_space: i8) -> (bool, Color) {
    match board.board[potential_space as usize] {
        None => return (false, Color::White),
        Some(_piece) => {
            let _p = board.board[potential_space as usize].unwrap();
        }
    }
    if board.board[potential_space as usize].is_none() {
        return (false, Color::White);
    }
    (true, board.board[potential_space as usize].unwrap().color)
}

/// Method checks the moves the other side could make in response to a move to determine if a check
/// would result. Removes moves if they are invalid. Checks for check :)
pub fn check_check(board: &Board, moves: &mut Vec<Move>) {
    let mut idx: i32 = 0;
    loop {
        if idx as usize >= moves.len() {
            break;
        }
        let mut new_board = board.clone();
        let _q = &moves[idx as usize];
        new_board.make_move(&moves[idx as usize]);
        let new_moves = generate_all_moves(&new_board);
        for new_m in new_moves {
            match board.to_move {
                Color::White => {
                    if new_m.end_idx == new_board.white_king_square {
                        moves.swap_remove(idx as usize);
                        idx -= 1;
                        break;
                    }
                }
                Color::Black => {
                    if new_m.end_idx == new_board.black_king_square {
                        moves.swap_remove(idx as usize);
                        idx -= 1;
                        break;
                    }
                }
            }
        }
        idx += 1;
    }
}

pub fn generate_all_moves(board: &Board) -> Vec<Move> {
    let mut moves: Vec<Move> = Vec::new();
    let color_to_move = board.to_move;
    for piece in board.board {
        match piece {
            None => continue,
            Some(piece) => {
                if piece.color == color_to_move {
                    let mut vec = generate_moves_for_piece(board, &piece);
                    moves.append(&mut vec);
                }
            }
        }
    }
    moves
}

fn generate_moves_for_piece(board: &Board, piece: &Piece) -> Vec<Move> {
    match piece.piece_name {
        PieceName::King => generate_king_moves(board, piece),
        PieceName::Queen => generate_queen_moves(board, piece),
        PieceName::Rook => generate_rook_moves(board, piece),
        PieceName::Bishop => generate_bishop_moves(board, piece),
        PieceName::Knight => generate_knight_moves(board, piece),
        PieceName::Pawn => generate_new_pawn_moves(board, piece),
    }
}

fn is_cardinal(direction: Direction) -> bool {
    matches!(direction, Direction::North | Direction::West | Direction::South | Direction::East)
}

fn directional_move(
    direction: Direction,
    piece: &Piece,
    board: &Board,
    start: usize,
    end: usize,
) -> Vec<Move> {
    let mut moves: Vec<Move> = Vec::new();
    // Loops multiple times to add moves if provided as a parameter
    for i in start..end {
        // Function contains a bool determining if direction points to a valid square based off the
        // current square, and if true the usize is the new index being looked at
        let new_square_indices = convert_idx_to_tuple(direction, i as i8);
        let (idx, square_validity) = check_index_addition(piece.current_square, new_square_indices);
        if !square_validity {
            break;
        }
        // Method returns a tuple containing a bool determining if potential square contains a
        // piece of any kind, and if true color contains the color of the new piece
        let occupancy = check_space_occupancy(board, idx as i8);
        if !occupancy.0 && (piece.piece_name != PieceName::Pawn || (piece.piece_name == PieceName::Pawn && is_cardinal(direction))) {
            // If position not occupied, add the move
            moves.push(Move {
                starting_idx: piece.current_square,
                end_idx: idx as i8,
                castle: Castle::None,
                promotion: is_promotion(piece, idx as i8),
                capture: board.board[idx],
                piece_moving: piece.piece_name,
            });
        }
        // Otherwise square is occupied
        else {
            if piece.color == occupancy.1 {
                // If color of other piece is the same as current piece, you can't move there
                break;
            }
            if piece.piece_name == PieceName::Pawn && (is_cardinal(direction) || !occupancy.0) {
                // Can't capture if the piece is a pawn and direction is non-diagonal
                break;
            }
            // Otherwise you can capture that piece
            moves.push(Move {
                starting_idx: piece.current_square,
                end_idx: idx as i8,
                castle: Castle::None,
                promotion: is_promotion(piece, idx as i8),
                capture: board.board[idx],
                piece_moving: piece.piece_name,
            });
        }
    }
    moves
}

fn generate_king_moves(board: &Board, piece: &Piece) -> Vec<Move> {
    let mut moves: Vec<Move> = Vec::new();
    // Generate moves for castling
    match piece.color {
        Color::White => {
            if board.white_queen_castle
                && piece.current_square == 4
                && board.board[3].is_none()
                && board.board[2].is_none()
                && board.board[1].is_none()
            {
                moves.push(Move {
                    starting_idx: 4,
                    end_idx: 2,
                    castle: Castle::WhiteQueenCastle,
                    promotion: false,
                    capture: board.board[2],
                    piece_moving: piece.piece_name,
                });
            }
            if board.white_king_castle
                && piece.current_square == 4
                && board.board[5].is_none()
                && board.board[6].is_none()
            {
                moves.push(Move {
                    starting_idx: 4,
                    end_idx: 6,
                    castle: Castle::WhiteKingCastle,
                    promotion: false,
                    capture: board.board[4],
                    piece_moving: piece.piece_name,
                });
            }
        }
        Color::Black => {
            if board.black_queen_castle
                && piece.current_square == 60
                && board.board[57].is_none()
                && board.board[58].is_none()
                && board.board[59].is_none()
            {
                moves.push(Move {
                    starting_idx: 60,
                    end_idx: 58,
                    castle: Castle::BlackQueenCastle,
                    promotion: false,
                    capture: board.board[58],
                    piece_moving: piece.piece_name,
                });
            }
            if board.black_king_castle
                && piece.current_square == 60
                && board.board[61].is_none()
                && board.board[62].is_none()
            {
                moves.push(Move {
                    starting_idx: 60,
                    end_idx: 62,
                    castle: Castle::BlackKingCastle,
                    promotion: false,
                    capture: board.board[62],
                    piece_moving: piece.piece_name,
                });
            }
        }
    }
    // Generate the normal directional moves for a King
    for direction in Direction::iter() {
        moves.append(&mut directional_move(direction, piece, board, 1, 2));
    }
    moves
}

fn generate_queen_moves(board: &Board, piece: &Piece) -> Vec<Move> {
    let mut moves: Vec<Move> = Vec::new();
    for direction in Direction::iter() {
        moves.append(&mut directional_move(direction, piece, board, 1, 8));
    }
    moves
}

fn generate_rook_moves(board: &Board, piece: &Piece) -> Vec<Move> {
    let mut moves: Vec<Move> = Vec::new();
    for direction in Direction::iter() {
        match direction {
            // Filter out the four diagonals
            Direction::NorthWest => continue,
            Direction::NorthEast => continue,
            Direction::SouthWest => continue,
            Direction::SouthEast => continue,
            // Continue generating move if move is in a cardinal direction
            _ => (),
        }
        moves.append(&mut directional_move(direction, piece, board, 1, 8));
    }
    moves
}

fn generate_bishop_moves(board: &Board, piece: &Piece) -> Vec<Move> {
    let mut moves: Vec<Move> = Vec::new();
    for direction in Direction::iter() {
        match direction {
            // Filter out the four main cardinal directions
            Direction::North => continue,
            Direction::South => continue,
            Direction::East => continue,
            Direction::West => continue,
            // Continue generating move if move is diagonal
            _ => (),
        }
        moves.append(&mut directional_move(direction, piece, board, 1, 8));
    }
    moves
}

/// Movement chords are defined by a combination of three cardinal directions - ex West West North
#[derive(EnumIter, Copy, Clone, Debug, PartialEq, Eq)]
#[allow(clippy::upper_case_acronyms)]
enum KnightMovement {
    WWN = 6,
    WNN = 15,
    ENN = 17,
    EEN = 10,
    EES = -6,
    ESS = -15,
    WSS = -17,
    WWS = -10,
}

/// Converts a direction of moves into a tuple of x,y movement
fn knight_move_to_tuple(k: KnightMovement) -> (i8, i8) {
    match k {
        KnightMovement::WWN => (-2, 1),
        KnightMovement::WNN => (-1, 2),
        KnightMovement::ENN => (1, 2),
        KnightMovement::EEN => (2, 1),
        KnightMovement::EES => (2, -1),
        KnightMovement::ESS => (1, -2),
        KnightMovement::WSS => (-1, -2),
        KnightMovement::WWS => (-2, -1),
    }
}

fn generate_knight_moves(board: &Board, piece: &Piece) -> Vec<Move> {
    let mut moves: Vec<Move> = Vec::new();
    for direction in KnightMovement::iter() {
        let movement_directions = knight_move_to_tuple(direction);
        let square_validity = check_index_addition(piece.current_square, movement_directions);
        if !square_validity.1 {
            continue;
        }
        let (occupied, potential_color) = check_space_occupancy(board, square_validity.0 as i8);
        if !occupied {
            moves.push(Move {
                starting_idx: piece.current_square,
                end_idx: square_validity.0 as i8,
                castle: Castle::None,
                promotion: false,
                capture: board.board[square_validity.0],
                piece_moving: piece.piece_name,
            });
        } else {
            if piece.color == potential_color {
                continue;
            }
            moves.push(Move {
                starting_idx: piece.current_square,
                end_idx: square_validity.0 as i8,
                castle: Castle::None,
                promotion: false,
                capture: board.board[square_validity.0],
                piece_moving: piece.piece_name,
            });
        }
    }
    moves
}

fn is_promotion(piece: &Piece, end_idx: i8) -> bool {
    if piece.piece_name == PieceName::Pawn {
        return match piece.color {
            Color::White => end_idx > 55 && end_idx < 64,
            Color::Black => end_idx > -1 && end_idx < 8,
        }
    }
    false
}

fn generate_old_pawn_moves(board: &Board, piece: &Piece) -> Vec<Move> {
    let mut moves: Vec<Move> = Vec::new();
    match piece.color {
        Color::White => {
            // Determines if one square in front of piece is occupied
            let (n_occupied, _) =
                check_space_occupancy(board, piece.current_square + Direction::North as i8);
            // Determines if two squares in front of piece is occupied
            if piece.current_square > 7 && piece.current_square < 16 {
                let (nn_occupied, _) =
                    check_space_occupancy(board, piece.current_square + 2 * Direction::North as i8);
                if !n_occupied && !nn_occupied {
                    let end_idx = piece.current_square + 2 * Direction::North as i8;
                    // Handles moving two spaces forward if pawn has not moved yet
                    moves.push(Move {
                        starting_idx: piece.current_square,
                        end_idx,
                        castle: Castle::None,
                        promotion: is_promotion(piece, end_idx),
                        capture: board.board[end_idx as usize],
                        piece_moving: piece.piece_name,
                    });
                }
            }
            if !n_occupied {
                // Can still move one space forward on the first square
                let end_idx = piece.current_square + Direction::North as i8;
                moves.push(Move {
                    starting_idx: piece.current_square,
                    end_idx,
                    castle: Castle::None,
                    promotion: is_promotion(piece, end_idx),
                    capture: board.board[end_idx as usize],
                    piece_moving: piece.piece_name,
                });
            }
            let (nw_occupied, potential_color) =
                check_space_occupancy(board, piece.current_square + Direction::NorthWest as i8);
            if nw_occupied && piece.color != potential_color {
                // Capturing to the northwest
                let end_idx = piece.current_square + Direction::NorthWest as i8;
                moves.push(Move {
                    starting_idx: piece.current_square,
                    end_idx,
                    castle: Castle::None,
                    promotion: is_promotion(piece, end_idx),
                    piece_moving: piece.piece_name,
                    capture: board.board[end_idx as usize],
                });
            }
            let (ne_occupied, potential_color) =
                check_space_occupancy(board, piece.current_square + Direction::NorthEast as i8);
            if ne_occupied && piece.color != potential_color {
                // Capturing to the northeast
                let end_idx = piece.current_square + Direction::NorthEast as i8;
                moves.push(Move {
                    starting_idx: piece.current_square,
                    end_idx: piece.current_square + Direction::NorthEast as i8,
                    castle: Castle::None,
                    promotion: is_promotion(piece, end_idx),
                    capture: board.board[end_idx as usize],
                    piece_moving: piece.piece_name,
                });
            }
        }
        Color::Black => {
            // First square to the south
            let (s_occupied, _) =
                check_space_occupancy(board, piece.current_square + Direction::South as i8);
            // Second square to the south
            let (ss_occupied, _) =
                check_space_occupancy(board, piece.current_square + 2 * Direction::South as i8);
            if piece.current_square > 47 && piece.current_square < 56 && !s_occupied && !ss_occupied
            {
                let end_idx = piece.current_square + 2 * Direction::South as i8;
                moves.push(Move {
                    starting_idx: piece.current_square,
                    end_idx: piece.current_square + 2 * Direction::South as i8,
                    castle: Castle::None,
                    promotion: is_promotion(piece, end_idx),
                    capture: board.board[end_idx as usize],
                    piece_moving: piece.piece_name,
                });
            }
            if !s_occupied {
                let end_idx = piece.current_square + Direction::South as i8;
                moves.push(Move {
                    starting_idx: piece.current_square,
                    end_idx: piece.current_square + Direction::South as i8,
                    castle: Castle::None,
                    promotion: is_promotion(piece, end_idx),
                    capture: board.board[end_idx as usize],
                    piece_moving: piece.piece_name,
                });
            }
            let (se_occupied, potential_color) =
                check_space_occupancy(board, piece.current_square + Direction::SouthEast as i8);
            if se_occupied && piece.color != potential_color {
                let end_idx = piece.current_square + Direction::SouthEast as i8;
                moves.push(Move {
                    starting_idx: piece.current_square,
                    end_idx: piece.current_square + Direction::SouthEast as i8,
                    castle: Castle::None,
                    promotion: is_promotion(piece, end_idx),
                    capture: board.board[end_idx as usize],
                    piece_moving: piece.piece_name,
                });
            }
            let (sw_occupied, potential_color) =
                check_space_occupancy(board, piece.current_square + Direction::SouthWest as i8);
            if sw_occupied && piece.color != potential_color {
                let end_idx = piece.current_square + Direction::SouthWest as i8;
                moves.push(Move {
                    starting_idx: piece.current_square,
                    end_idx: piece.current_square + Direction::SouthWest as i8,
                    castle: Castle::None,
                    promotion: is_promotion(piece, end_idx),
                    capture: board.board[end_idx as usize],
                    piece_moving: piece.piece_name,
                });
            }
        }
    }
    moves
}

fn generate_new_pawn_moves(board: &Board, piece: &Piece) -> Vec<Move> {
    let mut moves: Vec<Move> = Vec::new();
    match piece.color {
        Color::White => {
            if piece.current_square > 7 && piece.current_square < 16 {
                moves.append(&mut directional_move(Direction::North, piece, board, 1, 3));
            }
            else {
                moves.append(&mut directional_move(Direction::North, piece, board, 1, 2));
            }
            moves.append(&mut directional_move(Direction::NorthEast, piece, board, 1, 2));
            moves.append(&mut directional_move(Direction::NorthWest, piece, board, 1, 2));
        }
        Color::Black => {
            // First square to the south
            if piece.current_square > 47 && piece.current_square < 56 {
                moves.append(&mut directional_move(Direction::South, piece, board, 1, 3));
            }
            else {
                moves.append(&mut directional_move(Direction::South, piece, board, 1, 2));
            }
            moves.append(&mut directional_move(Direction::SouthEast, piece, board, 1, 2));
            moves.append(&mut directional_move(Direction::SouthWest, piece, board, 1, 2));
        }
    }
    moves
}

#[cfg(test)]
mod moves_tests {
    use crate::moves::{convert_idx_to_tuple, Direction};
    use super::check_index_addition;

    #[test]
    fn test_check_index_addition() {
        assert_eq!((23, true), check_index_addition(31, (0, -1)));
        assert_eq!((23, true), check_index_addition(31, convert_idx_to_tuple(Direction::South, 1)));
        assert_eq!((0, false), check_index_addition(13, (2, -2)));
    }
}
