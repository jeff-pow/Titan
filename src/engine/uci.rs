use std::{io, time::Duration};

use itertools::Itertools;

use crate::{
    board::{
        board::Board,
        fen::{self, build_board, parse_fen_from_buffer},
    },
    moves::moves::from_lan,
    search::{
        alpha_beta::{perft, AlphaBetaSearch},
        game_time::GameTime,
    },
    types::pieces::Color,
};

use super::transposition::add_to_history;

/// Main loop that handles UCI communication with GUIs
pub fn main_loop() -> ! {
    let mut board = build_board(fen::STARTING_FEN);
    let mut buffer = String::new();
    let mut searcher = AlphaBetaSearch::new();

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
                    parse_moves(&vec, &mut board, 9);
                }
            } else if buffer.contains("startpos") {
                board = build_board(fen::STARTING_FEN);

                if vec.len() > 3 {
                    parse_moves(&vec, &mut board, 3);
                }
            }
        } else if buffer.eq("d\n") {
            dbg!(&board);
        } else if buffer.eq("dbg\n") {
            dbg!(&board);
            board.debug_bitboards();
        } else if buffer.starts_with("go") {
            searcher.game_time = parse_time(&buffer);
            if buffer.contains("perft") {
                let vec: Vec<char> = buffer.chars().collect();
                let depth = vec[9].to_digit(10).unwrap();
                perft(&board, depth as i32);
            } else if buffer.contains("depth") {
                let vec: Vec<char> = buffer.chars().collect();
                let depth = vec[9].to_digit(10).unwrap();
                searcher.max_depth = Some(depth as i32);
                println!("bestmove {}", searcher.search(&board).to_lan());
            } else {
                let m = searcher.search(&board);
                println!("bestmove {}", m.to_lan());
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

fn parse_moves(moves: &[&str], board: &mut Board, skip: usize) {
    for str in moves.iter().skip(skip) {
        let m = from_lan(str, board);
        board.make_move(&m);
        add_to_history(board);
    }
}

fn parse_time(buff: &str) -> GameTime {
    let mut game_time = GameTime::default();
    let vec = buff.split_whitespace().skip(1).tuples::<(_, _)>();
    for entry in vec {
        match entry {
            ("wtime", wtime) => {
                game_time.time_remaining[Color::White as usize] =
                    Duration::from_millis(wtime.parse::<u64>().expect("Valid u64"))
            }
            ("btime", btime) => {
                game_time.time_remaining[Color::Black as usize] =
                    Duration::from_millis(btime.parse::<u64>().expect("Valid u64"))
            }
            ("winc", winc) => {
                game_time.time_inc[Color::White as usize] =
                    Duration::from_millis(winc.parse::<u64>().expect("Valid u64"))
            }
            ("binc", binc) => {
                game_time.time_inc[Color::Black as usize] =
                    Duration::from_millis(binc.parse::<u64>().expect("Valid u64"))
            }
            ("movestogo", moves) => game_time.movestogo = moves.parse::<i32>().expect("Valid i32"),
            _ => return game_time,
        }
    }
    dbg!(game_time);
    game_time
}
