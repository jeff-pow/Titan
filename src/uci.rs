use crate::board::Board;

use crate::fen::{self, build_board, parse_fen_from_buffer};
use crate::moves::{from_lan, generate_all_moves};
use rand::seq::SliceRandom;
use std::io;

pub fn main_loop() -> ! {
    let mut board = Board::new();
    let mut buffer = String::new();
    let mut debug = false;

    loop {
        buffer.clear();
        io::stdin().read_line(&mut buffer).unwrap();
        if buffer.starts_with("uci") {
            println!("id name Jeff's Chess Engine");
            println!("id author Jeff Powell");
            println!("uciok");
        } else if buffer.starts_with("isready") {
            println!("readyok");
        } else if buffer.starts_with("debug on") {
            debug = true;
        } else if buffer.starts_with("ucinewgame") {
            board = Board::new();
        } else if buffer.starts_with("position") {
            let vec: Vec<&str> = buffer.split_whitespace().collect();
            if buffer.contains("fen") {
                board = build_board(&parse_fen_from_buffer(&vec));
                if debug {
                    board.print();
                    println!();
                }
                if vec.len() > 8 {
                    parse_moves(&vec, &mut board, 8, debug);
                }
            } else if buffer.contains("startpos") {
                board = build_board(fen::STARTING_FEN);
                if debug {
                    board.print();
                }
                if vec.len() > 2 {
                    parse_moves(&vec, &mut board, 2, debug);
                }
                if debug {
                    board.print();
                    println!();
                }
            }
        } else if buffer.starts_with("go") {
            let moves = generate_all_moves(&board);
            let m = moves.choose(&mut rand::thread_rng()).unwrap();
            println!("{}", m.to_lan());
        }
    }
}

fn parse_moves(moves: &[&str], board: &mut Board, skip: usize, debug: bool) {
    for str in moves.iter().skip(skip) {
        let m = from_lan(str, board);
        board.make_move(&m);
        if debug {
            m.print();
            board.print();
            println!("------------------------");
        }
    }
}
