use crate::board::Board;
use crate::fen::{self, build_board, parse_fen_from_buffer};
use crate::moves::from_lan;
use crate::search::*;
use crate::zobrist::add_to_triple_repetition_map;
#[allow(unused_imports)]
use rand::seq::SliceRandom;
use std::collections::HashMap;
use std::io;

/// Main loop that handles UCI communication with GUIs
pub fn main_loop() -> ! {
    let mut board = Board::new();
    let mut buffer = String::new();
    let mut triple_repetitions: HashMap<u64, u8> = HashMap::new();

    loop {
        buffer.clear();
        io::stdin().read_line(&mut buffer).unwrap();

        if buffer.starts_with("isready") {
            println!("readyok");
        } else if buffer.starts_with("debug on") {
            println!("info string debug on");
        } else if buffer.starts_with("ucinewgame") {
            board = build_board(fen::STARTING_FEN);
        } else if buffer.starts_with("position") {
            let vec: Vec<&str> = buffer.split_whitespace().collect();

            if buffer.contains("fen") {
                board = build_board(&parse_fen_from_buffer(&vec));

                if vec.len() > 9 {
                    parse_moves(&vec, &mut board, 9, &mut triple_repetitions);
                }
            } else if buffer.contains("startpos") {
                board = build_board(fen::STARTING_FEN);

                if vec.len() > 3 {
                    parse_moves(&vec, &mut board, 3, &mut triple_repetitions);
                }
            }
        } else if buffer.eq("d\n") {
            println!("{}\n", board);
        } else if buffer.starts_with("go") {
            if buffer.contains("perft") {
                let vec: Vec<char> = buffer.chars().collect();
                let depth = vec[9].to_digit(10).unwrap();
                perft(&board, depth as i32);
            } else {
                let m = search(&board, 3, &mut triple_repetitions);
                println!("bestmove {}", m.to_lan());
                board.make_move(&m);
            }
        } else if buffer.starts_with("stop") || buffer.starts_with("quit") {
            std::process::exit(0);
        } else if buffer.starts_with("uci") {
            println!("id name Jeff's Chess Engine");
            println!("id author Jeff Powell");
            println!("uciok");
        } else {
            println!("Command not handled: {}", buffer);
        }
    }
}

fn parse_moves(moves: &[&str], board: &mut Board, skip: usize, zobrist_map: &mut HashMap<u64, u8>) {
    for str in moves.iter().skip(skip) {
        let m = from_lan(str, board);
        board.make_move(&m);
        println!("{m}");
        println!("{board}");
        println!();
        add_to_triple_repetition_map(board, zobrist_map);
    }
}
