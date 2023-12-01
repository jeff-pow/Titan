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
    history_heuristics::MoveHistory,
    search::{search, MAX_SEARCH_DEPTH},
    PlyEntry, SearchType,
};

#[derive(Clone)]
pub struct ThreadData<'a> {
    pub max_depth: i32,
    pub iter_max_depth: i32,
    pub transpos_table: &'a TranspositionTable,
    pub nodes_searched: u64,
    pub stack: [PlyEntry; MAX_SEARCH_DEPTH as usize],
    pub halt: &'a AtomicBool,
    pub current_line: Vec<Move>,
    pub sel_depth: i32,
    pub history: MoveHistory,
    pub root_color: Color,
    pub game_time: GameTime,
    pub search_type: SearchType,
}

impl<'a> ThreadData<'a> {
    pub(crate) fn new(transpos_table: &'a TranspositionTable, root_color: Color, halt: &'a AtomicBool) -> Self {
        Self {
            max_depth: MAX_SEARCH_DEPTH,
            iter_max_depth: 0,
            transpos_table,
            nodes_searched: 0,
            stack: [PlyEntry::default(); MAX_SEARCH_DEPTH as usize],
            current_line: Vec::with_capacity(MAX_SEARCH_DEPTH as usize),
            sel_depth: 0,
            history: MoveHistory::default(),
            root_color,
            game_time: GameTime::default(),
            halt,
            search_type: SearchType::default(),
        }
    }
}

pub struct ThreadPool<'a> {
    pub main_thread: ThreadData<'a>,
    pub halt: &'a AtomicBool,
    pub searching: AtomicBool,
    pub total_nodes: Arc<AtomicU64>,
}

impl<'a> ThreadPool<'a> {
    pub fn new(board: &Board, table: &'a TranspositionTable, halt: &'a AtomicBool) -> Self {
        Self {
            main_thread: ThreadData::new(table, board.to_move, halt),
            halt,
            searching: AtomicBool::new(false),
            total_nodes: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn reset(&mut self) {
        self.main_thread.transpos_table.clear();
        self.main_thread.history = MoveHistory::default();
        self.main_thread.nodes_searched = 0;
        self.halt.store(false, Ordering::Relaxed);
        self.searching.store(false, Ordering::Relaxed);
    }

    pub fn handle_go(&mut self, buffer: &str, board: &Board, halt: &AtomicBool, msg: &mut Option<String>) {
        self.halt.store(false, Ordering::SeqCst);

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
                println!("bestmove {}", search(&mut self.main_thread, true, *board).to_san());
                self.main_thread.transpos_table.age_up();
            });
            let mut s = String::new();
            io::stdin().read_line(&mut s).unwrap();
            match s.as_str().trim() {
                // "isready" => println!("readyok"),
                "quit" => std::process::exit(0),
                "stop" => halt.store(true, Ordering::Relaxed),
                _ => {
                    println!("huh thats weird: {}", s);
                    *msg = Some(s);
                }
            }
        });
    }
}
