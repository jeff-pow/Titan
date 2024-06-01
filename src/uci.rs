use std::process::exit;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::{io, time::Duration};

use crate::bench::bench;
use crate::chess_move::Move;
use crate::fen::{parse_fen_from_buffer, STARTING_FEN};
use crate::perft::perft;
use crate::search::lmr_table::LmrTable;
use crate::thread::ThreadPool;
use crate::transposition::{TranspositionTable, TARGET_TABLE_SIZE_MB};
use crate::{board::Board, search::game_time::Clock, types::pieces::Color};

pub const ENGINE_NAME: &str = "Titan";
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Main loop that handles UCI communication with GUIs
pub fn main_loop() -> ! {
    let mut transpos_table = TranspositionTable::new(TARGET_TABLE_SIZE_MB);
    let mut board = Board::from_fen(STARTING_FEN);
    let consts = LmrTable::new();
    let mut msg: Option<String> = None;
    let mut hash_history = Vec::new();
    let halt = AtomicBool::new(false);
    let global_nodes = AtomicU64::new(0);
    let mut thread_pool = ThreadPool::new(&halt, Vec::new(), &consts, &global_nodes);
    println!("{ENGINE_NAME} v{VERSION} by {}", env!("CARGO_PKG_AUTHORS"));

    loop {
        let input = msg.as_ref().map_or_else(
            || {
                let mut buffer = String::new();
                let len_read = io::stdin().read_line(&mut buffer).unwrap();
                if len_read == 0 {
                    // Stdin closed, exit for openbench
                    exit(0);
                }
                buffer
            },
            Clone::clone,
        );

        msg = None;
        let input = input.split_whitespace().collect::<Vec<_>>();

        match *input.first().unwrap_or(&"Invalid command") {
            "isready" => println!("readyok"),
            "ucinewgame" => {
                transpos_table.clear();
                halt.store(false, Ordering::Relaxed);
                thread_pool = ThreadPool::new(&halt, Vec::new(), &consts, &global_nodes);
            }
            "eval" => {
                let acc = board.new_accumulator();
                println!("raw: {} cp, adjusted: {} cp", acc.raw_evaluate(board.stm), acc.scaled_evaluate(&board),)
            }
            "position" => position_command(&input, &mut board, &mut hash_history),
            "d" => {
                dbg!(&board);
            }
            "dbg" => {
                dbg!(&board);
                board.debug_bitboards();
            }
            "bench" => bench(),
            "clear" => {
                println!("Engine state cleared");
                thread_pool.reset();
                transpos_table.clear();
            }
            "go" => {
                thread_pool.handle_go(&input, &board, &halt, &mut msg, &hash_history, &transpos_table);
            }
            "perft" => {
                perft(&board, input[1].parse().unwrap());
            }
            "quit" => {
                exit(0);
            }
            "uci" => {
                uci_opts();
            }
            "setoption" => match input[..] {
                ["setoption", "name", "Hash", "value", x] => {
                    transpos_table = TranspositionTable::new(x.parse().unwrap());
                }
                ["setoption", "name", "Clear", "Hash"] => transpos_table.clear(),
                ["setoption", "name", "Threads", "value", x] => {
                    thread_pool.add_workers(x.parse().unwrap(), &hash_history, &consts, &global_nodes)
                }
                _ => println!("Option not recognized"),
            },
            _ => (),
        };
    }
}

fn uci_opts() {
    println!("id name {ENGINE_NAME} {VERSION}");
    println!("id author {}", env!("CARGO_PKG_AUTHORS"));
    println!("option name Threads type spin default 1 min 1 max 1");
    println!("option name Hash type spin default 16 min 1 max 8388608");
    println!("uciok");
}

fn position_command(input: &[&str], board: &mut Board, hash_history: &mut Vec<u64>) {
    hash_history.clear();

    if input.contains(&"fen") {
        *board = Board::from_fen(&parse_fen_from_buffer(input));

        if let Some(skip) = input.iter().position(|f| f == &"moves") {
            parse_moves(&input[skip + 1..], board, hash_history);
        }
    } else if input.contains(&"startpos") {
        *board = Board::from_fen(STARTING_FEN);

        if let Some(skip) = input.iter().position(|f| f == &"moves") {
            parse_moves(&input[skip + 1..], board, hash_history);
        }
    }
}

fn parse_moves(moves: &[&str], board: &mut Board, hash_history: &mut Vec<u64>) {
    for str in moves.iter() {
        let m = Move::from_san(str, board);
        let _ = board.make_move(m);
        hash_history.push(board.zobrist_hash);
    }
}

pub fn parse_time(buff: &[&str]) -> Clock {
    let mut game_time = Clock::default();
    let mut iter = buff.iter().skip(1);
    while let Some(uci_opt) = iter.next() {
        match *uci_opt {
            "wtime" => {
                let raw_time = iter.next().unwrap().parse::<i64>().expect("Valid i64").max(1);
                game_time.time_remaining[Color::White] = Duration::from_millis(raw_time as u64);
            }
            "btime" => {
                let raw_time = iter.next().unwrap().parse::<i64>().expect("Valid i64").max(1);
                game_time.time_remaining[Color::Black] = Duration::from_millis(raw_time as u64);
            }
            "winc" => {
                let raw_time = iter.next().unwrap().parse::<i64>().expect("Valid i64").max(1);
                game_time.time_inc[Color::White] = Duration::from_millis(raw_time as u64);
            }
            "binc" => {
                let raw_time = iter.next().unwrap().parse::<i64>().expect("Valid i64").max(1);
                game_time.time_inc[Color::Black] = Duration::from_millis(raw_time as u64);
            }
            "movestogo" => game_time.movestogo = iter.next().unwrap().parse::<i32>().expect("Valid i32"),
            _ => return game_time,
        }
    }
    game_time
}
