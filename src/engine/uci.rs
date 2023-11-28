use std::sync::atomic::Ordering;
use std::thread::{self, JoinHandle};
use std::{io, time::Duration};

use itertools::Itertools;

use crate::bench::bench;
use crate::board::fen::parse_fen_from_buffer;
use crate::board::zobrist::ZOBRIST;
use crate::engine::perft::perft;
use crate::moves::movegenerator::MG;
use crate::search::get_reduction;
use crate::search::search::search;
use crate::types::square::Square;
use crate::{
    board::{
        board::Board,
        fen::{self, build_board},
    },
    moves::moves::from_san,
    search::{game_time::GameTime, SearchInfo, SearchType},
    types::pieces::Color,
};

fn handle_go(buffer: &str, search_info: &mut SearchInfo) -> Option<JoinHandle<()>> {
    search_info.halt.store(false, Ordering::SeqCst);
    search_info.transpos_table.age_up();

    if buffer.contains("depth") {
        let mut iter = buffer.split_whitespace().skip(2);
        let depth = iter.next().unwrap().parse::<i32>().unwrap();
        search_info.max_depth = depth;
        search_info.search_type = SearchType::Depth;
    } else if buffer.contains("wtime") {
        search_info.search_type = SearchType::Time;
        search_info.game_time = parse_time(buffer, search_info);
    } else {
        search_info.search_type = SearchType::Infinite;
    }

    let mut s = search_info.clone();
    Some(thread::spawn(move || {
        println!("bestmove {}", search(&mut s, true).to_san());
        s.transpos_table.age_up();
    }))
}

/// Main loop that handles UCI communication with GUIs
pub fn main_loop() -> ! {
    let mut search_info = SearchInfo::default();
    let mut buffer = String::new();
    // Calling this code will allow the global static zobrist and movegenerator constants to be
    // initialized before the engine enters play, so it doesn't waste playing time initializing
    // constants. A large difference in STC
    let _ = ZOBRIST.turn_hash;
    let _ = MG.king_attacks(Square(0));
    let _ = get_reduction(0, 0);
    println!("option name Threads type spin default 1 min 1 max 1");
    println!("option name Hash type spin default 16 min 16 max 16");
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
            println!("{} cp", search_info.board.evaluate());
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
            // println!("{}", &search_info.board);
        } else if buffer.eq("dbg\n") {
            dbg!(&search_info.board);
            search_info.board.debug_bitboards();
        } else if buffer.starts_with("bench") {
            bench();
        } else if buffer.starts_with("clear") {
            search_info = SearchInfo::default();
            println!("Transposition table cleared");
        } else if buffer.starts_with("go") {
            handle = handle_go(&buffer, &mut search_info);
        } else if buffer.contains("perft") {
            let mut iter = buffer.split_whitespace().skip(1);
            let depth = iter.next().unwrap().parse::<i32>().unwrap();
            perft(&search_info.board, depth);
        } else if buffer.starts_with("stop") {
            search_info.halt.store(true, Ordering::SeqCst);
            if let Some(h) = handle.take() {
                let _ = h.join();
            }
            search_info.halt.store(false, Ordering::SeqCst);
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
        let m = from_san(str, board);
        let _ = board.make_move::<true>(m);
    }
}

fn parse_time(buff: &str, search_info: &mut SearchInfo) -> GameTime {
    let mut game_time = GameTime::default();
    let vec = buff.split_whitespace().skip(1).tuples::<(_, _)>();
    for entry in vec {
        match entry {
            ("wtime", wtime) => {
                search_info.search_type = SearchType::Time;
                game_time.time_remaining[Color::White] = Duration::from_millis(wtime.parse::<u64>().expect("Valid u64"))
            }
            ("btime", btime) => {
                search_info.search_type = SearchType::Time;
                game_time.time_remaining[Color::Black] = Duration::from_millis(btime.parse::<u64>().expect("Valid u64"))
            }
            ("winc", winc) => {
                search_info.search_type = SearchType::Time;
                game_time.time_inc[Color::White] = Duration::from_millis(winc.parse::<u64>().expect("Valid u64"))
            }
            ("binc", binc) => {
                search_info.search_type = SearchType::Time;
                game_time.time_inc[Color::Black] = Duration::from_millis(binc.parse::<u64>().expect("Valid u64"))
            }
            ("movestogo", moves) => game_time.movestogo = moves.parse::<i32>().expect("Valid i32"),
            _ => return game_time,
        }
    }
    game_time
}
