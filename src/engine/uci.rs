use std::io;

use itertools::Itertools;

use crate::{
    board::{
        board::Board,
        fen::{self, build_board, parse_fen_from_buffer},
        zobrist::add_to_history,
    },
    moves::moves::from_lan,
    search::alpha_beta::{perft, Search},
};

/// Main loop that handles UCI communication with GUIs
pub fn main_loop() -> ! {
    let mut board = fen::build_board(fen::STARTING_FEN);
    let mut buffer = String::new();
    let mut searcher = Search::new();

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
                    parse_moves(&vec, &mut board, 9, &mut searcher.history);
                }
            } else if buffer.contains("startpos") {
                board = build_board(fen::STARTING_FEN);

                if vec.len() > 3 {
                    parse_moves(&vec, &mut board, 3, &mut searcher.history);
                }
            }
        } else if buffer.eq("d\n") {
            dbg!(board);
        } else if buffer.eq("dbg\n") {
            dbg!(board);
            board.debug_bitboards();
        } else if buffer.starts_with("go") {
            parse_go_buffer(&mut board, &buffer);
            if buffer.contains("perft") {
                let vec: Vec<char> = buffer.chars().collect();
                let depth = vec[9].to_digit(10).unwrap();
                perft(&board, depth as i32);
            } else {
                let m = searcher.search(&board, 6);
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

fn parse_moves(moves: &[&str], board: &mut Board, skip: usize, history: &mut Vec<u64>) {
    for str in moves.iter().skip(skip) {
        let m = from_lan(str, board);
        board.make_move(&m);
        add_to_history(board, history);
    }
}

fn parse_go_buffer(board: &mut Board, buff: &str) {
    let mut vec = buff.split_whitespace().skip(1).tuples::<(_, _)>();
    while let Some(entry) = vec.next() {
        match entry {
            ("winc", _) => {
                board.
            }
        }
    }
}
