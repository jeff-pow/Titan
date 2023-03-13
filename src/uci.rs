use crate::board::Board;
use crate::fen::{self, build_board, parse_fen_from_buffer};
use crate::moves::{from_lan, in_check, generate_all_moves};
use crate::pieces::Color;
use crate::search::*;
#[allow(unused_imports)]
use rand::seq::SliceRandom;
use std::fs::File;
use std::io::{self, Write};

fn setup() {
    println!("Current options");
    println!();
    println!("id name Jeff's Chess Engine");
    println!("id author Jeff Powell");
    println!("uciok");
}

pub fn main_loop() -> ! {
    setup();
    let mut board = Board::new();
    let mut buffer = String::new();
    let mut file = File::create("log.txt").expect("File can't be created");
    let mut debug = false;

    loop {
        buffer.clear();
        io::stdin().read_line(&mut buffer).unwrap();

        writeln!(file, "UCI said: {}", buffer).expect("File not written to");

        if buffer.starts_with("isready") {
            println!("readyok");
            writeln!(file, "readyok").expect("File not written to");
        } 
        else if buffer.starts_with("debug on") {
            debug = true;
            println!("info string debug on");
        } 
        else if buffer.starts_with("ucinewgame") {
            board = build_board(fen::STARTING_FEN);
        } 
        else if buffer.starts_with("position") {
            let vec: Vec<&str> = buffer.split_whitespace().collect();

            if buffer.contains("fen") {
                board = build_board(&parse_fen_from_buffer(&vec));

                if debug {
                    println!("info string {}", board);
                }

                if vec.len() > 9 {
                    parse_moves(&vec, &mut board, 9, debug);
                }
            }
            else if buffer.contains("startpos") {
                board = build_board(fen::STARTING_FEN);

                if debug {
                    println!("info string\n {}", board);
                }

                if vec.len() > 3 {
                    parse_moves(&vec, &mut board, 3, debug);
                }

                if debug {
                    println!("info string\n {}", board);
                }
            }
        }
        else if buffer.eq("d\n") {
            println!("{}\n", board);
        }
        else if buffer.starts_with("go") {
            if buffer.contains("perft") {
                let vec: Vec<char> = buffer.chars().collect();
                let depth = vec[9].to_digit(10).unwrap();
                perft(&board, depth as i32);
            }
            else {
                let moves = generate_all_moves(&mut board);
                // let m = moves.choose(&mut rand::thread_rng()).unwrap();
                let m = search(&board, 6);
                println!("bestmove {}", m.to_lan());
                board.make_move(&m);

                if debug {
                    println!("info string MOVE CHOSEN: {}\n {}", m, board);
                }
                writeln!(file, "{}", m.to_lan()).unwrap();
            }
        }
        else if buffer.starts_with("stop") || buffer.starts_with("quit") {
            std::process::exit(0);
        }
        else if buffer.starts_with("uci") {
            println!("id name Jeff's Chess Engine");
            println!("id author Jeff Powell");
            println!("uciok");

            writeln!(file, "id name Jeff's Chess Engine").expect("File not written to");
            writeln!(file, "id author Jeff Powell").expect("File not written to");
            writeln!(file, "uciok").expect("File not written to");
        }
        else {
            writeln!(file, "{}", buffer).unwrap();
            println!("Command not handled: {}", buffer);
        }
    }
}

fn parse_moves(moves: &[&str], board: &mut Board, skip: usize, debug: bool) {
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
        if debug {
            print!("info string making move {}\n {}", m, board);
            println!("{}", m);
            println!("{}", board);
            println!("------------------------");
        }
    }
}
