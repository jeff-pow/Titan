use std::process::exit;
use std::sync::atomic::{AtomicBool, Ordering};
use std::{io, time::Duration};

use itertools::Itertools;

use crate::bench::bench;
use crate::board::fen::{parse_fen_from_buffer, STARTING_FEN};
use crate::engine::perft::perft;
use crate::engine::transposition::{TranspositionTable, TARGET_TABLE_SIZE_MB};
use crate::eval::accumulator::Accumulator;
use crate::search::lmr_reductions;
use crate::search::thread::ThreadPool;
use crate::spsa::{parse_param, uci_print_tunable_params, SPSA_TUNE};
use crate::{
    board::{
        board::Board,
        fen::{self, build_board},
    },
    moves::moves::from_san,
    search::game_time::GameTime,
    types::pieces::Color,
};

pub const ENGINE_NAME: &str = "Titan";

/// Main loop that handles UCI communication with GUIs
pub fn main_loop() -> ! {
    let mut transpos_table = TranspositionTable::new(TARGET_TABLE_SIZE_MB);
    let mut board = build_board(STARTING_FEN);
    let mut msg: Option<String> = None;
    let mut hash_history = Vec::new();
    let halt = AtomicBool::new(false);
    let mut thread_pool = ThreadPool::new(&halt, Vec::new());
    lmr_reductions();
    println!("{ENGINE_NAME} by {}", env!("CARGO_PKG_AUTHORS"));

    loop {
        let input = if let Some(ref m) = msg {
            m.clone()
        } else {
            let mut buffer = String::new();
            io::stdin().read_line(&mut buffer).unwrap();
            buffer
        };

        msg = None;
        let input = input.split_whitespace().collect::<Vec<_>>();

        match *input.first().unwrap_or(&"Invalid command") {
            "isready" => println!("readyok"),
            "ucinewgame" => {
                transpos_table.clear();
                halt.store(false, Ordering::Relaxed);
                thread_pool = ThreadPool::new(&halt, Vec::new());
            }
            "eval" => println!("{} cp", board.evaluate(&board.clone().new_accumulator())),
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
                    hash_history.clone(),
                    &transpos_table,
                );
                transpos_table.age_up();
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
                    transpos_table = TranspositionTable::new(x.parse().unwrap())
                }
                ["setoption", "name", "Clear", "Hash"] => transpos_table.clear(),
                ["setoption", "name", "Threads", "value", x] => {
                    thread_pool.add_workers(x.parse().unwrap(), hash_history.clone())
                }
                _ => {
                    if SPSA_TUNE {
                        parse_param(&input)
                    }
                }
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
    if SPSA_TUNE {
        uci_print_tunable_params();
    }
    println!("uciok");
}

fn position_command(input: &[&str], board: &mut Board, hash_history: &mut Vec<u64>) {
    hash_history.clear();

    if input.contains(&"fen") {
        *board = build_board(&parse_fen_from_buffer(input));

        if input.len() > 9 {
            parse_moves(input, board, 9, hash_history);
        }
    } else if input.contains(&"startpos") {
        *board = build_board(fen::STARTING_FEN);

        if input.len() > 3 {
            parse_moves(input, board, 3, hash_history);
        }
    }
}

fn parse_moves(moves: &[&str], board: &mut Board, skip: usize, hash_history: &mut Vec<u64>) {
    for str in moves.iter().skip(skip) {
        let m = from_san(str, board);
        let _ = board.make_move::<true>(m, &mut Accumulator::default());
        hash_history.push(board.zobrist_hash);
    }
}

pub(crate) fn parse_time(buff: &[&str]) -> GameTime {
    let mut game_time = GameTime::default();
    let vec = buff.iter().skip(1).tuples::<(_, _)>();
    for entry in vec {
        match entry {
            (&"wtime", wtime) => {
                game_time.time_remaining[Color::White] =
                    Duration::from_millis(wtime.parse::<u64>().expect("Valid u64"))
            }
            (&"btime", btime) => {
                game_time.time_remaining[Color::Black] =
                    Duration::from_millis(btime.parse::<u64>().expect("Valid u64"))
            }
            (&"winc", winc) => {
                game_time.time_inc[Color::White] =
                    Duration::from_millis(winc.parse::<u64>().expect("Valid u64"))
            }
            (&"binc", binc) => {
                game_time.time_inc[Color::Black] =
                    Duration::from_millis(binc.parse::<u64>().expect("Valid u64"))
            }
            (&"movestogo", moves) => game_time.movestogo = moves.parse::<i32>().expect("Valid i32"),
            _ => return game_time,
        }
    }
    game_time
}
