use core::fmt;
use std::fmt::Display;

use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::attack_boards::{gen_pawn_attack_board, AttackBoards, RANK2, RANK7};
use crate::pieces::PieceName::Pawn;
use crate::{board::Board, pieces::Color, pieces::PieceName};

#[derive(Clone, Copy, Debug)]
pub struct Move {
    pub starting_idx: i8,
    pub end_idx: i8,
    pub castle: Castle,
    pub promotion: Promotion,
    pub piece_moving: PieceName,
    pub capture: Option<PieceName>,
    pub en_passant: EnPassant,
}

impl Move {
    #[allow(dead_code)]
    pub fn is_quiet(&self) -> bool {
        self.capture.is_some()
    }

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
            Promotion::Knight => str += "n",
            Promotion::None => (),
        }
        str
    }

    /// Constructor for new moves - Mostly a placeholder for initializing variables that will
    /// certainly be changed at some other point during the runtime of the function
    pub fn invalid() -> Self {
        Move {
            starting_idx: -1,
            end_idx: -1,
            castle: Castle::None,
            promotion: Promotion::None,
            piece_moving: PieceName::King,
            capture: None,
            en_passant: EnPassant::None,
        }
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
    let piece_moving = board
        .piece_on_square(starting_idx as usize)
        .expect("There should be a piece here...");
    let castle = match piece_moving {
        PieceName::King => {
            if i8::abs_diff(starting_idx, end_idx) != 2 {
                Castle::None
            } else if end_idx == 2 {
                Castle::WhiteQueenCastle
            } else if end_idx == 6 {
                Castle::WhiteKingCastle
            } else if end_idx == 58 {
                Castle::BlackQueenCastle
            } else if end_idx == 62 {
                Castle::BlackKingCastle
            } else {
                unreachable!()
            }
        }
        _ => Castle::None,
    };
    let capture = board.piece_on_square(end_idx as usize);
    Move {
        starting_idx,
        end_idx,
        castle,
        promotion,
        piece_moving,
        capture,
        en_passant: EnPassant::None,
        // BUG: WHAT WAS I THINKING HERE
    }
}

#[derive(Clone, Copy, Debug, EnumIter, PartialEq)]
pub enum Promotion {
    Queen,
    Rook,
    Bishop,
    Knight,
    None,
}

#[derive(Clone, Copy, Debug, EnumIter, PartialEq)]
pub enum EnPassant {
    NW,
    NE,
    SW,
    SE,
    None,
}

#[derive(Clone, Copy, Debug, PartialEq)]
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

#[inline]
pub fn coordinates(idx: usize) -> (usize, usize) {
    (idx % 8, idx / 8)
}

pub fn generate_psuedolegal_moves(board: &Board, bb: &AttackBoards) -> Vec<Move> {
    let mut moves = Vec::new();
    moves.append(&mut generate_bitboard_moves(board, bb, PieceName::Knight));
    moves.append(&mut generate_bitboard_moves(board, bb, PieceName::King));
    moves.append(&mut generate_pawn_moves(board));
    moves
}

// This method uses attacks and calculates starting position from whether or not the move was a double
// push or not because it knows the calling method filtered out the pawns that couldn't move
fn push_pawn_moves(
    moves: &mut Vec<Move>,
    double_push: bool,
    mut attacks: u64,
    color: Color,
    board: &Board,
) {
    let mut idx = 0;
    match color {
        Color::White => {
            while attacks != 0 {
                if attacks & 1 != 0 {
                    let capture = None;
                    if double_push && bit_is_on(board.occupancy(), idx as usize - 8) {
                        attacks >>= 1;
                        idx += 1;
                        continue;
                    }
                    let starting_idx =
                        if double_push && !bit_is_on(board.occupancy(), idx as usize - 8) {
                            idx - 16
                        } else {
                            idx - 8
                        };
                    moves.push(Move {
                        end_idx: idx,
                        starting_idx,
                        castle: Castle::None,
                        promotion: Promotion::None,
                        piece_moving: Pawn,
                        capture,
                        en_passant: EnPassant::None,
                    })
                }
                attacks >>= 1;
                idx += 1;
            }
        }
        Color::Black => {
            while attacks != 0 {
                if attacks & 1 != 0 {
                    let capture = None;
                    if double_push && bit_is_on(board.occupancy(), idx as usize + 8) {
                        attacks >>= 1;
                        idx += 1;
                        continue;
                    }
                    let starting_idx =
                        if double_push && !bit_is_on(board.occupancy(), idx as usize + 8) {
                            idx + 16
                        } else {
                            idx + 8
                        };
                    moves.push(Move {
                        end_idx: idx,
                        starting_idx,
                        castle: Castle::None,
                        promotion: Promotion::None,
                        piece_moving: Pawn,
                        capture,
                        en_passant: EnPassant::None,
                    })
                }
                }
                attacks >>= 1;
                idx += 1;
            }
        }
    }
}
fn generate_pawn_moves(board: &Board) -> Vec<Move> {
    let mut moves = Vec::new();
    let pawn_attacks = gen_pawn_attack_board(board);
    let pawns = board.board[board.to_move as usize][PieceName::Pawn as usize];
    match board.to_move {
        Color::White => {
            // Bitwise and the pawns with the second row
            let double_push_endings = ((pawns & RANK2) << 16) & !board.occupancy();
            push_pawn_moves(&mut moves, true, double_push_endings, Color::White, board);

            let single_push_endings = ((pawns & !RANK7) << 8) & !board.occupancy();
            // Handle the promotions separately by anding with the second to last row
            push_pawn_moves(&mut moves, false, single_push_endings, Color::White, board);

            // Don't do promotions if there aren't any pawns in the seventh column...
            if pawns & RANK7 != 0 {
                let mut straight_promotions = ((pawns & RANK7) << 8) & !board.occupancy();
                let mut idx = 0;
                while straight_promotions != 0 {
                    if straight_promotions & 1 != 0 {
                        let capture = None;
                        for p in Promotion::iter() {
                            if p == Promotion::None {
                                continue;
                            }
                            moves.push(Move {
                                starting_idx: idx - 8,
                                end_idx: idx,
                                castle: Castle::None,
                                promotion: p,
                                piece_moving: Pawn,
                                capture,
                                en_passant: EnPassant::None,
                            });
                        }
                    }
                    straight_promotions >>= 1;
                    idx += 1;
                }
            }

            let captures = pawn_attacks & !board.color_occupancy(board.to_move);
        }
        Color::Black => {
            // Bitwise and the pawns with the second to last row
            let _double_push = ((pawns & 0xff000000000000) >> 16) & !board.occupancy();

            let _single_push = (pawns >> 8) & !board.occupancy();

            let _captures = pawn_attacks & !board.color_occupancy(board.to_move);
        }
    }
    moves
}

fn generate_bitboard_moves(board: &Board, bb: &AttackBoards, piece_name: PieceName) -> Vec<Move> {
    if board.board[board.to_move as usize][piece_name as usize] == 0 {
        return Vec::new();
    }
    let mut moves = Vec::new();
    for square in 0..63 {
        if board.square_contains_piece(piece_name, board.to_move, square) {
            let occupancy = !board.color_occupancy(board.to_move);
            let attack_bitboard = match piece_name {
                PieceName::King => bb.king[square],
                PieceName::Queen => panic!(),
                PieceName::Rook => todo!(),
                PieceName::Bishop => todo!(),
                PieceName::Knight => bb.knight[square],
                PieceName::Pawn => panic!(),
            };
            let attacks = attack_bitboard & occupancy;
            push_moves(
                board,
                piece_name,
                &mut moves,
                attacks,
                square,
                EnPassant::None,
            );
        }
    }
    moves
}

fn push_moves(
    board: &Board,
    piece_name: PieceName,
    moves: &mut Vec<Move>,
    mut attacks: u64,
    square: usize,
    en_passant: EnPassant,
) {
    let mut idx = 0;
    while attacks != 0 {
        if attacks & 1 != 0 {
            let capture = board.piece_on_square(idx as usize);
            moves.push(Move {
                starting_idx: square as i8,
                end_idx: idx,
                castle: Castle::None,
                promotion: Promotion::None,
                piece_moving: piece_name,
                capture,
                en_passant,
            })
        }
        attacks >>= 1;
        idx += 1;
    }
}

fn bit_is_on(bb: u64, idx: usize) -> bool {
    bb & (1 << idx) != 0
}

pub fn generate_legal_moves(board: &Board, bb: &AttackBoards) -> Vec<Move> {
    let psuedolegal = generate_psuedolegal_moves(board, bb);

    psuedolegal
        .into_iter()
        .filter(|m| {
            let mut new_b = *board;
            new_b.make_move(m, bb);
            !new_b.under_attack(bb, board.to_move)
        })
        .collect::<Vec<Move>>()
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
            Some(piece_name) => match piece_name {
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
