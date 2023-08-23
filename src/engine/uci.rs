use std::{io, time::Duration};

use itertools::Itertools;

use crate::board::fen::parse_fen_from_buffer;
use crate::{
    board::{
        board::Board,
        fen::{self, build_board},
    },
    moves::moves::from_lan,
    search::pvs,
    search::{game_time::GameTime, SearchInfo, SearchType},
    types::pieces::Color,
};

use super::perft::perft;
use super::transposition::get_table;

/// Main loop that handles UCI communication with GUIs
pub fn main_loop() -> ! {
    let mut search_info = SearchInfo::default();
    let mut buffer = String::new();
    search_info.transpos_table = get_table();
    search_info.board = build_board(fen::STARTING_FEN);

    loop {
        buffer.clear();
        io::stdin().read_line(&mut buffer).unwrap();
        search_info.search_stats.nodes_searched = 0;

        if buffer.starts_with("isready") {
            println!("readyok");
        } else if buffer.starts_with("debug on") {
            println!("info string debug on");
        } else if buffer.starts_with("ucinewgame") {
            search_info.board = build_board(fen::STARTING_FEN);
        } else if buffer.starts_with("position") {
            let vec: Vec<&str> = buffer.split_whitespace().collect();

            if buffer.contains("fen") {
                search_info.board = build_board(&parse_fen_from_buffer(&vec));

                if vec.len() > 9 {
                    parse_moves(&vec, &mut search_info.board, 9);
                }
            } else if buffer.contains("startpos") {
                search_info.board = build_board(fen::STARTING_FEN);

                if vec.len() > 3 {
                    parse_moves(&vec, &mut search_info.board, 3);
                }
            }
        } else if buffer.eq("d\n") {
            dbg!(&search_info.board);
        } else if buffer.eq("dbg\n") {
            dbg!(&search_info.board);
            search_info.board.debug_bitboards();
        } else if buffer.starts_with("go") {
            search_info.game_time = parse_time(&buffer);
            if buffer.contains("perft") {
                let mut iter = buffer.split_whitespace();
                iter.next();
                iter.next();
                let depth = iter.next().unwrap().parse::<i8>().unwrap();
                perft(search_info.board.to_owned(), depth as i32);
            } else if buffer.contains("depth") {
                let mut iter = buffer.split_whitespace();
                iter.next();
                iter.next();
                let depth = iter.next().unwrap().parse::<i8>().unwrap();
                search_info.iter_max_depth = depth;
                search_info.search_type = SearchType::Depth;
                println!("bestmove {}", pvs::search(&mut search_info).to_lan());
            } else {
                search_info.search_type = SearchType::Time;
                let m = pvs::search(&mut search_info);
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
    game_time
}
