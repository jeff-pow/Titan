use std::process::exit;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::{io, time::Duration};

use itertools::Itertools;

use crate::bench::bench;
use crate::board::fen::{parse_fen_from_buffer, STARTING_FEN};
use crate::engine::perft::perft;
use crate::engine::transposition::{TranspositionTable, TARGET_TABLE_SIZE_MB};
use crate::moves::moves::Move;
use crate::search::lmr_table::LmrTable;
use crate::search::thread::ThreadPool;
use crate::{
    board::{
        board::Board,
        fen::{self},
    },
    search::game_time::Clock,
    types::pieces::Color,
};

pub const ENGINE_NAME: &str = "Titan";

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
    println!("{ENGINE_NAME} by {}", env!("CARGO_PKG_AUTHORS"));

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
            std::clone::Clone::clone,
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
            "eval" => println!(
                "raw: {} cp, adjusted: {} cp",
                board.raw_evaluate(&board.clone().new_accumulator()),
                board.evaluate(&board.clone().new_accumulator()),
            ),
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
                thread_pool.handle_go(
                    &input,
                    &board,
                    &halt,
                    &mut msg,
                    &hash_history,
                    &transpos_table,
                );
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
                ["setoption", "name", "Threads", "value", x] => thread_pool.add_workers(
                    x.parse().unwrap(),
                    &hash_history,
                    &consts,
                    &global_nodes,
                ),
                _ => println!("Option not recognized"),
            },
            _ => (),
        };
    }
}

fn uci_opts() {
    println!("id name {ENGINE_NAME}");
    println!("id author {}", env!("CARGO_PKG_AUTHORS"));
    println!("option name Threads type spin default 1 min 1 max 64");
    println!("option name Hash type spin default 16 min 1 max 8388608");
    println!("uciok");
}

fn position_command(input: &[&str], board: &mut Board, hash_history: &mut Vec<u64>) {
    hash_history.clear();

    if input.contains(&"fen") {
        *board = Board::from_fen(&parse_fen_from_buffer(input));

        if input.len() > 9 {
            parse_moves(input, board, 9, hash_history);
        }
    } else if input.contains(&"startpos") {
        *board = Board::from_fen(fen::STARTING_FEN);

        if input.len() > 3 {
            parse_moves(input, board, 3, hash_history);
        }
    }
}

fn parse_moves(moves: &[&str], board: &mut Board, skip: usize, hash_history: &mut Vec<u64>) {
    for str in moves.iter().skip(skip) {
        let m = Move::from_san(str, board);
        let _ = board.make_move::<false>(m);
        hash_history.push(board.zobrist_hash);
    }
}

pub fn parse_time(buff: &[&str]) -> Clock {
    let mut game_time = Clock::default();
    let vec = buff.iter().skip(1).tuples::<(_, _)>();
    for entry in vec {
        match entry {
            (&"wtime", wtime) => {
                game_time.time_remaining[Color::White] =
                    Duration::from_millis(wtime.parse::<u64>().expect("Valid u64"));
            }
            (&"btime", btime) => {
                game_time.time_remaining[Color::Black] =
                    Duration::from_millis(btime.parse::<u64>().expect("Valid u64"));
            }
            (&"winc", winc) => {
                game_time.time_inc[Color::White] =
                    Duration::from_millis(winc.parse::<u64>().expect("Valid u64"));
            }
            (&"binc", binc) => {
                game_time.time_inc[Color::Black] =
                    Duration::from_millis(binc.parse::<u64>().expect("Valid u64"));
            }
            (&"movestogo", moves) => game_time.movestogo = moves.parse::<i32>().expect("Valid i32"),
            _ => return game_time,
        }
    }
    game_time
}
