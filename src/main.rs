mod moves;
mod pieces;
mod uci;
mod search;

use std::process::exit;
use board::Board;
use fen::build_board;
use pieces::Piece;
use search::{search_moves, perft};
use crate::{moves::{check_check, generate_all_moves}, search::time_move_search};

mod board;
mod fen;

fn main() {
    let board = build_board("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8");
    time_move_search(&board, 6);
}

#[allow(dead_code)]
fn print_moves(board: &Board) {
    println!("{}", board);
    let mut board = *board;
    let mut moves = generate_all_moves(&board);
    let i = moves.len();
    check_check(&mut board, &mut moves);
    for m in moves.iter() {
        println!("{}", m);
        let mut cloned_board = board;
        cloned_board.make_move(m);
        println!("{}", cloned_board);
        println!("---------------------------------------------------------");
    }
    println!("{} moves possible pre check", i);
    println!("{} moves possible post check", moves.len());
    exit(0);
}
