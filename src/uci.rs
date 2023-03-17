use crate::board::Board;
use crate::fen::{self, build_board, parse_fen_from_buffer};
use crate::moves::{from_lan, in_check};
use crate::pieces::Color;
use crate::search::*;
#[allow(unused_imports)]
use rand::seq::SliceRandom;
use std::fs::File;
use std::io::{self, Write};

pub fn main_loop() -> ! {
    let mut board = Board::new();
    let mut buffer = String::new();
    let mut file = File::create("log.txt").expect("File can't be created");

    loop {
        buffer.clear();
        io::stdin().read_line(&mut buffer).unwrap();

        writeln!(file, "UCI said: {}", buffer).expect("File not written to");

        if buffer.starts_with("isready") {
            println!("readyok");
            writeln!(file, "readyok").expect("File not written to");
        } else if buffer.starts_with("debug on") {
            println!("info string debug on");
        } else if buffer.starts_with("ucinewgame") {
            board = build_board(fen::STARTING_FEN);
        } else if buffer.starts_with("position") {
            let vec: Vec<&str> = buffer.split_whitespace().collect();

            if buffer.contains("fen") {
                board = build_board(&parse_fen_from_buffer(&vec));

                if vec.len() > 9 {
                    parse_moves(&vec, &mut board, 9);
                }
            } else if buffer.contains("startpos") {
                board = build_board(fen::STARTING_FEN);

                if vec.len() > 3 {
                    parse_moves(&vec, &mut board, 3);
                }
            }
        } else if buffer.eq("d\n") {
            println!("{}\n", board);
        } else if buffer.starts_with("go") {
            if buffer.contains("perft") {
                let vec: Vec<char> = buffer.chars().collect();
                let depth = vec[9].to_digit(10).unwrap();
                perft(&board, depth as i32);
            } else if buffer.contains("old") {
                let m = old_search(&board, 6);
                println!("bestmove {}", m.to_lan());
            } else {
                let m = search(&board, 8);
                println!("bestmove {}", m.to_lan());
                board.make_move(&m);
                writeln!(file, "{}", m.to_lan()).unwrap();
            }
        } else if buffer.starts_with("stop") || buffer.starts_with("quit") {
            std::process::exit(0);
        } else if buffer.starts_with("uci") {
            println!("id name Jeff's Chess Engine");
            println!("id author Jeff Powell");
            println!("uciok");

            writeln!(file, "id name Jeff's Chess Engine").expect("File not written to");
            writeln!(file, "id author Jeff Powell").expect("File not written to");
            writeln!(file, "uciok").expect("File not written to");
        } else {
            writeln!(file, "{}", buffer).unwrap();
            println!("Command not handled: {}", buffer);
        }
    }
}

fn parse_moves(moves: &[&str], board: &mut Board, skip: usize) {
    for str in moves.iter().skip(skip) {
        let m = from_lan(str, board);
        board.make_move(&m);
        if in_check(board, board.to_move) {
            match board.to_move {
                Color::White => {
                    board.white_king_castle = false;
                    board.white_queen_castle = false;
                }
                Color::Black => {
                    board.black_king_castle = false;
                    board.black_queen_castle = false;
                }
            }
        }
    }
}
