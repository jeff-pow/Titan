use core::fmt;
use std::fmt::Display;

use strum::IntoEnumIterator;
use strum_macros::EnumIter;


use crate::{board::Board, pieces::Color, pieces::Piece, pieces::PieceName};

#[derive(Clone, Copy)]
pub struct Move {
    pub starting_idx: i8,
    pub end_idx: i8,
    pub castle: Castle,
    pub promotion: Promotion,
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
        str += " Promotion: ";
        match self.promotion {
            Promotion::Queen => str += "Queen ",
            Promotion::Rook => str += "Rook ",
            Promotion::Bishop => str += "Bishop ",
            Promotion::Knight => str += "Knight ",
            Promotion::None => str += "None ",
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
    pub fn to_lan(self) -> String {
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
            Promotion::Queen => str += "q",
            Promotion::Rook => str += "r",
            Promotion::Bishop => str += "b",
            Promotion::Knight => str += "k",
            Promotion::None => (),
        }
        str
    }

    /// Constructor for new moves - Mostly a placeholder for initializing variables that will
    /// certainly be changed at some other point during the runtime of the function
    pub fn new() -> Self {
        Move { starting_idx: 0, end_idx: 0, castle: Castle::None, promotion: Promotion::None, piece_moving: PieceName::King, capture: None }
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

    let mut promotion = Promotion::None;
    if vec.len() > 4 {
        promotion = match vec[4] {
            'q' => Promotion::Queen,
            'r' => Promotion::Rook,
            'b' => Promotion::Bishop,
            'n' => Promotion::Knight,
            _ => panic!(),
        };
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

#[derive(Clone, Copy, EnumIter, PartialEq)]
pub enum Promotion {
    Queen,
    Rook,
    Bishop,
    Knight,
    None
}

#[derive(Clone, Copy, PartialEq)]
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
fn check_index_addition(a: i8, b: (i8, i8)) -> Option<i8> {
    let a_x_component = a % 8;
    let a_y_component = a / 8;
    let b_x_component = b.0;
    let b_y_component = b.1;
    let x_sum = a_x_component + b_x_component;
    let y_sum = a_y_component + b_y_component;
    if !(0..8).contains(&x_sum) || !(0..8).contains(&y_sum) {
        return None;
    }
    let ret = a + (b.1 * 8) + b.0;
    Some(ret)
}

/// Method returns None if no piece is located at the potential space, and Some(Color) if a piece
/// is found
fn check_space_occupancy(board: &Board, potential_space: i8) -> Option<Color> {
    match board.board[potential_space as usize] {
        None => return None,
        Some(_piece) => {
            let _p = board.board[potential_space as usize].unwrap();
        }
    }
    board.board[potential_space as usize]?;
    Option::from(board.board[potential_space as usize].expect("There should be a piece here").color)
}

fn check_diagonal_squares(board: &Board, d: Direction, c: Color) -> bool {
    let king_square = match c {
        Color::White => board.white_king_square,
        Color::Black => board.black_king_square,
    };
    for i in 1..8 {
        let square = check_index_addition(king_square, convert_idx_to_tuple(d, i));
        if square.is_none() {
            // Edge of the board has been reached, no further squares to search in that direction
            return false;
        }
        let square = square.unwrap();
        match board.board[square as usize] {
            // Check further squares if there is no piece at the square
            None => continue,
            Some(piece) => {
                // King is not in check from a friendly piece, who also blocks further pieces from
                // placing king in check
                if piece.color == c {
                    return false;
                }
                return match piece.piece_name {
                    PieceName::King => true,
                    PieceName::Queen => true,
                    PieceName::Rook => false,
                    PieceName::Bishop => true,
                    PieceName::Knight => false,
                    PieceName::Pawn => {
                        if i == 1 {
                            return check_pawn_attack(d, c)
                        }
                        false
                    },
                }
            }
        }
    }
    unreachable!()
}

fn check_cardinal_squares(board: &Board, d: Direction, c: Color) -> bool {
    let king_square = match c {
        Color::White => board.white_king_square,
        Color::Black => board.black_king_square,
    };
    for i in 1..8 {
        let square = check_index_addition(king_square, convert_idx_to_tuple(d, i));
        if square.is_none() {
            // Edge of the board has been reached, no further squares to search in that direction
            return false;
        }
        let square = square.unwrap();
        match board.board[square as usize] {
            // Check further squares if there is no piece at the square
            None => continue,
            Some(piece) => {
                // King is not in check from a friendly piece, who also blocks further pieces from
                // placing king in check
                if piece.color == c {
                    return false;
                }
                return match piece.piece_name {
                    PieceName::King => true,
                    PieceName::Queen => true,
                    PieceName::Rook => true,
                    PieceName::Bishop => false,
                    PieceName::Knight => false,
                    PieceName::Pawn => {
                        if i == 1 {
                            return check_pawn_attack(d, c)
                        }
                        false
                    },
                }
            }
        }
    }
    unreachable!()
}

fn check_pawn_attack(d: Direction, c: Color) -> bool {
    let oc = match c {
        Color::White => Color::Black,
        Color::Black => Color::White,
    };
    match oc {
        Color::White => {
            if d == Direction::NorthWest || d == Direction::NorthEast {
                return true;
            }
            false
        }
        Color::Black => {
            if d == Direction::SouthWest || d == Direction::SouthEast {
                return true;
            }
            false
        }
    }
}

fn check_knight_moves(board: &Board, c: Color) -> bool {
    let king_square = match c {
        Color::White => board.white_king_square,
        Color::Black => board.black_king_square,
    };
    let oc = match c {
        Color::White => Color::Black,
        Color::Black => Color::White,
    };
    for m in KnightMovement::iter() {
        if let Some(square) = check_index_addition(king_square, knight_move_to_tuple(m)) {
            if let Some(piece) = board.board[square as usize] {
                if piece.piece_name == PieceName::Knight && piece.color == oc {
                    return true; 
                }
            }
        }
    }
    false
}

/// Returns true if the color provided in the parameter is in check and false otherwise
pub fn in_check(board: &Board, color: Color) -> bool {
    // Generate the squares the other side is attacking
    for d in Direction::iter() {
        if is_cardinal(d) {
            if check_cardinal_squares(board, d, color) {
                return true;
            }
        }
        else if check_diagonal_squares(board, d, color) {
            return true;
        }
    }
    // Only need to check knight moves once
    if check_knight_moves(board, color) {
        return true;
    }
    false
}

/// Method checks the moves the other side could make in response to a move to determine if a check
/// would result. Removes moves if they are invalid. Checks for check :)
fn check_for_check(board: &mut Board, moves: &mut Vec<Move>) {
    moves.retain(|m| {
        let mut new_b = board.clone();
        new_b.make_move(m);
        if m.piece_moving == PieceName::King {
            let i = 0;
        }
        !in_check(&new_b, board.to_move)
    })
}

/// Generates a list of moves available to a size at any given time. Filters out moves that would
/// place that side in check as well (illegal moves). Returns only fully legal moves for a position
pub fn generate_all_moves(board: &mut Board) -> Vec<Move> {
    let mut moves: Vec<Move> = Vec::new();
    match board.to_move {
        Color::White => {
            for p in board.white_pieces.borrow().iter() {
                moves.append(&mut generate_moves_for_piece(board, p));
            }
        }
        Color::Black => {
            for p in board.black_pieces.borrow().iter() {
                moves.append(&mut generate_moves_for_piece(board, p));
            }
        }
    }
    check_for_check(board, &mut moves);
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

fn directional_move(direction: Direction, piece: &Piece, board: &Board, start: usize, end: usize) -> Vec<Move> {
    let mut moves: Vec<Move> = Vec::new();
    // Loops multiple times to add moves if provided as a parameter
    for i in start..end {
        // Function contains a bool determining if direction points to a valid square based off the
        // current square, and if true the usize is the new index being looked at
        let new_square_indices = convert_idx_to_tuple(direction, i as i8);
        let idx = check_index_addition(piece.current_square, new_square_indices);
        if idx.is_none() {
            break;
        }
        let idx = idx.unwrap();
        // Method returns a tuple containing a bool determining if potential square contains a
        // piece of any kind, and if true color contains the color of the new piece
        let occupancy = check_space_occupancy(board, idx);
        // Handle special case of enpassant
        if piece.piece_name == PieceName::Pawn && idx == board.en_passant_square {
            moves.push( Move {
                starting_idx: piece.current_square,
                end_idx: idx,
                castle: Castle::None,
                promotion: Promotion::None,
                piece_moving: piece.piece_name,
                capture: board.board[idx as usize],
            });
        }
        if occupancy.is_none() && (piece.piece_name != PieceName::Pawn || (piece.piece_name == PieceName::Pawn && is_cardinal(direction))) {
            // If position not occupied, add the move
            if !is_promotion(piece, idx) {
                moves.push(Move { starting_idx: piece.current_square,
                    end_idx: idx,
                    castle: Castle::None,
                    promotion: Promotion::None,
                    piece_moving: piece.piece_name,
                    capture: board.board[idx as usize],
                });
            }
            else {
                for p in Promotion::iter() {
                    if p == Promotion::None {
                        continue;
                    }
                    moves.push(Move { starting_idx: piece.current_square,
                        end_idx: idx,
                        castle: Castle::None,
                        promotion: p,
                        piece_moving: piece.piece_name,
                        capture: board.board[idx as usize],
                    });
                }
            }
        }
        // Otherwise square is occupied
        else {
            if occupancy.is_some() && piece.color == occupancy.unwrap() {
                // If color of other piece is the same as current piece, you can't move there
                break;
            }
            if piece.piece_name == PieceName::Pawn && (is_cardinal(direction) || occupancy.is_none()) {
                // Can't capture if the piece is a pawn and direction is non-diagonal
                break;
            }
            // Otherwise you can capture that piece
            if !is_promotion(piece, idx) {
                moves.push(Move {
                    starting_idx: piece.current_square,
                    end_idx: idx,
                    castle: Castle::None,
                    promotion: Promotion::None,
                    capture: board.board[idx as usize],
                    piece_moving: piece.piece_name,
                });
            }
            else {
                for p in Promotion::iter() {
                    if p == Promotion::None {
                        continue;
                    }
                    moves.push(Move {
                        starting_idx: piece.current_square,
                        end_idx: idx,
                        castle: Castle::None,
                        promotion: p,
                        piece_moving: piece.piece_name,
                        capture: board.board[idx as usize],
                    });
                }
            }
            break;
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
                    promotion: Promotion::None,
                    capture: None,
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
                    promotion: Promotion::None,
                    capture: None,
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
                    promotion: Promotion::None,
                    capture: None,
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
                    promotion: Promotion::None,
                    capture: None,
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
        let idx = check_index_addition(piece.current_square, movement_directions);
        if idx.is_none() {
            continue;
        }
        let idx = idx.unwrap();
        let occupancy = check_space_occupancy(board, idx);
        if let Some(color) = occupancy {
            if piece.color == color {
                continue;
            }
            moves.push(Move {
                starting_idx: piece.current_square,
                end_idx: idx,
                castle: Castle::None,
                promotion: Promotion::None,
                capture: board.board[idx as usize],
                piece_moving: piece.piece_name,
            });
        }
        else {
            moves.push(Move {
                starting_idx: piece.current_square,
                end_idx: idx,
                castle: Castle::None,
                promotion: Promotion::None,
                capture: board.board[idx as usize],
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
        assert_eq!(Some(23), check_index_addition(31, (0, -1)));
        assert_eq!(Some(23), check_index_addition(31, convert_idx_to_tuple(Direction::South, 1)));
        assert_eq!(None, check_index_addition(13, (2, -2)));
    }
}
