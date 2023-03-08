use crate::board::Board;
use crate::fen::{self, build_board, parse_fen_from_buffer};
use crate::moves::{from_lan, generate_all_moves, check_check};
use crate::search::{search_moves, perft};
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
        else if buffer.starts_with("perft") {
            let vec: Vec<char> = buffer.chars().collect();
            let depth = vec[6].to_digit(10).unwrap();
            perft(&board, depth as i32);
        }
        else if buffer.eq("d") {
            println!("{}\n", board);
        }
        else if buffer.starts_with("go") {
            /*
            let mut moves = generate_all_moves(&board);
            check_check(&mut board, &mut moves);
            let m = moves.choose(&mut rand::thread_rng()).unwrap();
            */
            let m = search_moves(&board, 4);
            println!("bestmove {}", m.to_lan());
            board.make_move(&m);

            if debug {
                println!("info string MOVE CHOSEN: {}\n {}", m, board);
            }
            writeln!(file, "{}", m.to_lan()).unwrap();
        }
        else if buffer.starts_with("stop") {
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
            panic!("Command not handled: {}", buffer);
        }
    }
}

fn parse_moves(moves: &[&str], board: &mut Board, skip: usize, debug: bool) {
    for str in moves.iter().skip(skip) {
        let m = from_lan(str, board);
        board.make_move(&m);
        if debug {
            print!("info string making move {}\n {}", m, board);
            println!("{}", m);
            println!("{}", board);
            println!("------------------------");
        }
    }
}
