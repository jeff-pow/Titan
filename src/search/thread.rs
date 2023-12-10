use std::{
    io,
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc,
    },
    thread::{self},
};

use crate::{
    board::board::Board,
    engine::{transposition::TranspositionTable, uci::parse_time},
    moves::moves::Move,
    types::pieces::Color,
};

use super::{
    game_time::GameTime,
    history_heuristics::HistoryTable,
    search::{search, MAX_SEARCH_DEPTH},
    SearchStack, SearchType,
};

#[derive(Clone)]
pub(crate) struct ThreadData<'a> {
    pub ply: i32,
    pub max_depth: i32,
    pub iter_max_depth: i32,
    pub nodes_searched: u64,
    pub stack: SearchStack,
    pub halt: &'a AtomicBool,
    pub sel_depth: i32,
    pub history: HistoryTable,
    pub root_color: Color,
    pub game_time: GameTime,
    pub search_type: SearchType,
    pub hash_history: Vec<u64>,
}

impl<'a> ThreadData<'a> {
    pub(crate) fn new(root_color: Color, halt: &'a AtomicBool, hash_history: Vec<u64>) -> Self {
        Self {
            ply: 0,
            max_depth: MAX_SEARCH_DEPTH,
            iter_max_depth: 0,
            nodes_searched: 0,
            stack: SearchStack::default(),
            sel_depth: 0,
            history: HistoryTable::default(),
            root_color,
            game_time: GameTime::default(),
            halt,
            search_type: SearchType::default(),
            hash_history,
        }
    }

    pub fn print_search_stats(&self, eval: i32, pv: &[Move]) {
        print!(
            "info time {} seldepth {} depth {} nodes {} nps {} score cp {} pv ",
            self.game_time.search_start.elapsed().as_millis(),
            self.sel_depth,
            self.iter_max_depth,
            self.nodes_searched,
            (self.nodes_searched as f64 / self.game_time.search_start.elapsed().as_secs_f64()) as i64,
            eval
        );
        for m in pv {
            print!("{} ", m.to_san());
        }
        println!();
    }

    pub(crate) fn is_repetition(&self, board: &Board) -> bool {
        if self.hash_history.len() < 6 {
            return false;
        }

        let mut reps = 2;
        for &hash in self.hash_history.iter().rev().take(board.half_moves + 1) {
            reps -= u32::from(hash == board.zobrist_hash);
            if reps == 0 {
                return true;
            }
        }
        false
    }
}

pub struct ThreadPool<'a> {
    pub main_thread: ThreadData<'a>,
    pub halt: &'a AtomicBool,
    pub searching: AtomicBool,
    pub total_nodes: Arc<AtomicU64>,
}

impl<'a> ThreadPool<'a> {
    pub fn new(board: &Board, halt: &'a AtomicBool, hash_history: Vec<u64>) -> Self {
        Self {
            main_thread: ThreadData::new(board.to_move, halt, hash_history),
            halt,
            searching: AtomicBool::new(false),
            total_nodes: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn reset(&mut self) {
        self.main_thread.history = HistoryTable::default();
        self.main_thread.nodes_searched = 0;
        self.halt.store(false, Ordering::Relaxed);
        self.searching.store(false, Ordering::Relaxed);
    }

    pub fn handle_go(
        &mut self,
        buffer: &str,
        board: &Board,
        halt: &AtomicBool,
        msg: &mut Option<String>,
        hash_history: Vec<u64>,
        tt: &TranspositionTable,
    ) {
        self.halt.store(false, Ordering::SeqCst);
        self.main_thread.hash_history = hash_history;

        if buffer.contains("depth") {
            let mut iter = buffer.split_whitespace().skip(2);
            let depth = iter.next().unwrap().parse::<i32>().unwrap();
            self.main_thread.max_depth = depth;
            self.main_thread.search_type = SearchType::Depth;
        } else if buffer.contains("wtime") {
            self.main_thread.search_type = SearchType::Time;
            self.main_thread.game_time = parse_time(buffer);
            self.main_thread.game_time.recommended_time(board.to_move);
        } else {
            self.main_thread.search_type = SearchType::Infinite;
        }

        thread::scope(|s| {
            s.spawn(move || {
                println!("bestmove {}", search(&mut self.main_thread, true, *board, tt).to_san());
            });
            let mut s = String::new();
            io::stdin().read_line(&mut s).unwrap();
            match s.as_str().trim() {
                "isready" => println!("readyok"),
                "quit" => std::process::exit(0),
                "stop" => halt.store(true, Ordering::Relaxed),
                _ => {
                    *msg = Some(s);
                }
            }
        });
    }
}
