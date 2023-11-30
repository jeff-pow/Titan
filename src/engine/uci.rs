use std::sync::atomic::{AtomicBool, Ordering};
use std::{io, time::Duration};

use itertools::Itertools;

use crate::bench::bench;
use crate::board::fen::{parse_fen_from_buffer, STARTING_FEN};
use crate::board::zobrist::ZOBRIST;
use crate::engine::perft::perft;
use crate::engine::transposition::{TranspositionTable, TARGET_TABLE_SIZE_MB};
use crate::moves::movegenerator::MG;
use crate::search::get_reduction;
use crate::search::thread::ThreadPool;
use crate::types::square::Square;
use crate::{
    board::{
        board::Board,
        fen::{self, build_board},
    },
    moves::moves::from_san,
    search::game_time::GameTime,
    types::pieces::Color,
};

/// Main loop that handles UCI communication with GUIs
pub fn main_loop() -> ! {
    let transpos_table = TranspositionTable::new(TARGET_TABLE_SIZE_MB);
    let mut board = build_board(STARTING_FEN);
    let mut buffer = String::new();
    let halt = AtomicBool::new(false);
    let mut thread_pool = ThreadPool::new(&board, &transpos_table, &halt);
    // Calling this code will allow the global static zobrist and movegenerator constants to be
    // initialized before the engine enters play, so it doesn't waste playing time initializing
    // constants. A large difference in STC
    let _ = ZOBRIST.turn_hash;
    let _ = MG.king_attacks(Square(0));
    let _ = get_reduction(0, 0);
    println!("option name Threads type spin default 1 min 1 max 1");
    println!("option name Hash type spin default 16 min 16 max 16");

    loop {
        buffer.clear();
        io::stdin().read_line(&mut buffer).unwrap();
        thread_pool.total_nodes.store(0, Ordering::Relaxed);

        if buffer.starts_with("isready") {
            println!("readyok");
        } else if buffer.starts_with("debug on") {
            println!("info string debug on");
        } else if buffer.starts_with("ucinewgame") {
            transpos_table.clear();
            halt.store(false, Ordering::Relaxed);
            thread_pool = ThreadPool::new(&board, &transpos_table, &halt);
        } else if buffer.starts_with("eval") {
            println!("{} cp", board.evaluate());
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
        } else if buffer.starts_with("bench") {
            bench();
        } else if buffer.starts_with("clear") {
            thread_pool.reset();
            println!("Transposition table cleared");
        } else if buffer.starts_with("go") {
            thread_pool.handle_go(&buffer, &board, &halt);
        } else if buffer.contains("perft") {
            let mut iter = buffer.split_whitespace().skip(1);
            let depth = iter.next().unwrap().parse::<i32>().unwrap();
            perft(&board, depth);
        } else if buffer.starts_with("quit") {
            std::process::exit(0);
        } else if buffer.starts_with("uci") {
            println!("id name Quintessence");
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

pub(crate) fn parse_time(buff: &str) -> GameTime {
    let mut game_time = GameTime::default();
    let vec = buff.split_whitespace().skip(1).tuples::<(_, _)>();
    for entry in vec {
        match entry {
            ("wtime", wtime) => {
                game_time.time_remaining[Color::White] = Duration::from_millis(wtime.parse::<u64>().expect("Valid u64"))
            }
            ("btime", btime) => {
                game_time.time_remaining[Color::Black] = Duration::from_millis(btime.parse::<u64>().expect("Valid u64"))
            }
            ("winc", winc) => {
                game_time.time_inc[Color::White] = Duration::from_millis(winc.parse::<u64>().expect("Valid u64"))
            }
            ("binc", binc) => {
                game_time.time_inc[Color::Black] = Duration::from_millis(binc.parse::<u64>().expect("Valid u64"))
            }
            ("movestogo", moves) => game_time.movestogo = moves.parse::<i32>().expect("Valid i32"),
            _ => return game_time,
        }
    }
    game_time
}
