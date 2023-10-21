use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self};
use std::{io, time::Duration};

use itertools::Itertools;

use crate::board::fen::parse_fen_from_buffer;
use crate::eval::eval::evaluate;
use crate::search::killers::empty_killers;
use crate::search::pvs::{search, MAX_SEARCH_DEPTH};
use crate::{
    board::{
        board::Board,
        fen::{self, build_board},
    },
    moves::moves::from_lan,
    search::{game_time::GameTime, SearchInfo, SearchType},
    types::pieces::Color,
};

use super::perft::multi_threaded_perft;

/// Main loop that handles UCI communication with GUIs
pub fn main_loop() -> ! {
    let mut search_info = SearchInfo::default();
    let mut buffer = String::new();
    println!("Ready to go!");
    let halt = Arc::new(AtomicBool::new(false));
    let mut handle = None;

    loop {
        buffer.clear();
        io::stdin().read_line(&mut buffer).unwrap();
        search_info.search_stats.nodes_searched = 0;

        if buffer.starts_with("isready") {
            println!("readyok");
        } else if buffer.starts_with("debug on") {
            println!("info string debug on");
        } else if buffer.starts_with("ucinewgame") {
            search_info = SearchInfo::default();
        } else if buffer.starts_with("eval") {
            println!("{} cp", evaluate(&search_info.board));
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
        } else if buffer.starts_with("clear") {
            search_info.transpos_table.write().unwrap().clear();
            search_info.killer_moves = empty_killers();
            println!("Transposition table cleared");
        } else if buffer.starts_with("go") {
            halt.store(false, Ordering::SeqCst);
            if buffer.contains("depth") {
                let mut iter = buffer.split_whitespace().skip(2);
                let depth = iter.next().unwrap().parse::<i32>().unwrap();
                search_info.max_depth = depth;
                search_info.search_type = SearchType::Depth;
                let mut s = search_info.clone();
                let h = halt.clone();
                handle = Some(thread::spawn(move || {
                    println!("bestmove {}", search(&mut s, depth, h).to_lan());
                }));
            } else if buffer.contains("perft") {
                let mut iter = buffer.split_whitespace().skip(2);
                let depth = iter.next().unwrap().parse::<i32>().unwrap();
                multi_threaded_perft(search_info.board.to_owned(), depth);
            } else if buffer.contains("wtime") {
                search_info.search_type = SearchType::Time;
                search_info.game_time = parse_time(&buffer, &mut search_info);
                let mut s = search_info.clone();
                let h = halt.clone();
                handle = Some(thread::spawn(move || {
                    println!("bestmove {}", search(&mut s, MAX_SEARCH_DEPTH, h).to_lan());
                }));
            } else {
                search_info.search_type = SearchType::Infinite;
                let mut s = search_info.clone();
                let h = halt.clone();
                handle = Some(thread::spawn(move || {
                    search(&mut s, MAX_SEARCH_DEPTH, h);
                }));
            }
        } else if buffer.starts_with("stop") {
            halt.store(true, Ordering::SeqCst);
            if let Some(h) = handle.take() {
                let _ = h.join();
            }
            halt.store(false, Ordering::SeqCst);
        } else if buffer.starts_with("quit") {
            std::process::exit(0);
        } else if buffer.starts_with("uci") {
            println!("id name Kraken");
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
        board.make_move(m);
    }
}

fn parse_time(buff: &str, search_info: &mut SearchInfo) -> GameTime {
    let mut game_time = GameTime::default();
    let vec = buff.split_whitespace().skip(1).tuples::<(_, _)>();
    for entry in vec {
        match entry {
            ("wtime", wtime) => {
                search_info.search_type = SearchType::Time;
                game_time.time_remaining[Color::White.idx()] =
                    Duration::from_millis(wtime.parse::<u64>().expect("Valid u64"))
            }
            ("btime", btime) => {
                search_info.search_type = SearchType::Time;
                game_time.time_remaining[Color::Black.idx()] =
                    Duration::from_millis(btime.parse::<u64>().expect("Valid u64"))
            }
            ("winc", winc) => {
                search_info.search_type = SearchType::Time;
                game_time.time_inc[Color::White.idx()] = Duration::from_millis(winc.parse::<u64>().expect("Valid u64"))
            }
            ("binc", binc) => {
                search_info.search_type = SearchType::Time;
                game_time.time_inc[Color::Black.idx()] = Duration::from_millis(binc.parse::<u64>().expect("Valid u64"))
            }
            ("movestogo", moves) => game_time.movestogo = moves.parse::<i32>().expect("Valid i32"),
            _ => return game_time,
        }
    }
    game_time
}
