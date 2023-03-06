mod moves;
mod pieces;
mod uci;
mod search;

use std::process::exit;
use fen::build_board;
use pieces::Piece;
use search::time_move_search;
use crate::moves::{check_check, generate_all_moves};

mod board;
mod fen;

fn main() {
    let board = build_board(fen::STARTING_FEN);
    time_move_search(&board, 6);
    //print_moves();
    //uci::main_loop();
}

#[allow(dead_code)]
fn print_moves() {
    let mut board = fen::build_board("r1b1k3/pp1pQp2/n7/2p5/4P3/q7/PPPP1PPP/RNB1KBNR b KQq - 0 8");
    println!("{}", board);
    //let board = fen::build_board(fen::STARTING_FEN);
    let mut moves = generate_all_moves(&board);
    let i = moves.len();
    check_check(&mut board, &mut moves);
    for m in moves.iter() {
        println!("{}", m);
        let mut cloned_board = board.clone();
        cloned_board.make_move(m);
        println!("{}", cloned_board);
        println!("---------------------------------------------------------");
    }
    println!("{} moves possible pre check", i);
    println!("{} moves possible post check", moves.len());
    exit(0);
}
