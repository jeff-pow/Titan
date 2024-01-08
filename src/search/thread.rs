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
    search::search::{CHECKMATE, NEAR_CHECKMATE},
};

use super::{
    game_time::GameTime,
    history_table::HistoryTable,
    search::{search, MAX_SEARCH_DEPTH},
    SearchStack, SearchType, PV,
};

#[derive(Clone)]
pub(crate) struct ThreadData<'a> {
    pub ply: i32,
    pub max_depth: i32,
    pub iter_max_depth: i32,
    /// Max depth reached by a pv node
    pub sel_depth: i32,
    pub best_move: Move,

    pub nodes_searched: u64,
    pub global_nodes: Arc<AtomicU64>,
    pub stack: SearchStack,
    pub history: HistoryTable,
    pub hash_history: Vec<u64>,

    pub game_time: GameTime,
    pub search_type: SearchType,
    pub halt: &'a AtomicBool,
}

impl<'a> ThreadData<'a> {
    pub(crate) fn new(halt: &'a AtomicBool, hash_history: Vec<u64>) -> Self {
        Self {
            ply: 0,
            max_depth: MAX_SEARCH_DEPTH,
            iter_max_depth: 0,
            nodes_searched: 0,
            stack: SearchStack::default(),
            sel_depth: 0,
            best_move: Move::NULL,
            global_nodes: Arc::new(AtomicU64::new(0)),
            history: HistoryTable::default(),
            game_time: GameTime::default(),
            halt,
            search_type: SearchType::default(),
            hash_history,
        }
    }

    pub(super) fn print_search_stats(&self, eval: i32, pv: &PV, tt: &TranspositionTable) {
        let nodes = self.global_nodes.load(Ordering::Relaxed);
        print!(
            "info time {} depth {} seldepth {} nodes {} nps {} score ",
            self.game_time.search_start.elapsed().as_millis(),
            self.iter_max_depth,
            self.sel_depth,
            nodes,
            (nodes as f64 / self.game_time.search_start.elapsed().as_secs_f64()) as i64,
        );

        let score = eval;

        if score.abs() >= NEAR_CHECKMATE {
            if score.is_positive() {
                print!("mate {}", (CHECKMATE - score + 1) / 2);
            } else {
                print!("mate {}", (-(CHECKMATE + score) / 2));
            }
        } else {
            print!("cp {}", score);
        }

        print!(" hashfull {} pv ", tt.permille_usage());

        for m in pv.line.iter().take(pv.line.len()) {
            print!("{} ", m.to_san());
        }
        println!();
    }

    pub(super) fn is_repetition(&self, board: &Board) -> bool {
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
    pub workers: Vec<ThreadData<'a>>,
    pub halt: &'a AtomicBool,
    pub searching: AtomicBool,
    pub total_nodes: Arc<AtomicU64>,
}

impl<'a> ThreadPool<'a> {
    pub fn new(halt: &'a AtomicBool, hash_history: Vec<u64>) -> Self {
        Self {
            main_thread: ThreadData::new(halt, hash_history),
            workers: Vec::new(),
            halt,
            searching: AtomicBool::new(false),
            total_nodes: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn reset(&mut self) {
        self.main_thread.history = HistoryTable::default();
        self.main_thread.nodes_searched = 0;
        for t in self.workers.iter_mut() {
            t.history = HistoryTable::default();
            t.nodes_searched = 0;
        }
        self.halt.store(false, Ordering::Relaxed);
        self.searching.store(false, Ordering::Relaxed);
        self.total_nodes.store(0, Ordering::Relaxed);
    }

    /// This thread creates a number of workers equal to threads - 1. If 4 threads are requested,
    /// the main thread counts as one and then the remaining three are placed in the worker queue.
    pub fn add_workers(&mut self, threads: usize, hash_history: Vec<u64>) {
        self.workers.clear();
        for _ in 0..threads - 1 {
            self.workers.push(ThreadData::new(self.halt, hash_history.clone()));
        }
    }

    pub fn handle_go(
        &mut self,
        buffer: &[&str],
        board: &Board,
        halt: &AtomicBool,
        msg: &mut Option<String>,
        hash_history: Vec<u64>,
        tt: &TranspositionTable,
    ) {
        self.halt.store(false, Ordering::Relaxed);
        self.total_nodes.store(0, Ordering::Relaxed);
        self.main_thread.global_nodes = self.total_nodes.clone();
        self.main_thread.hash_history = hash_history.clone();
        for t in self.workers.iter_mut() {
            t.hash_history = hash_history.clone();
            t.global_nodes = self.total_nodes.clone();
        }

        if buffer.contains(&"depth") {
            let mut iter = buffer.iter().skip(2);
            let depth = iter.next().unwrap().parse::<i32>().unwrap();
            self.main_thread.max_depth = depth;
            for t in self.workers.iter_mut() {
                t.max_depth = depth;
            }
            self.main_thread.search_type = SearchType::Depth;
            for t in self.workers.iter_mut() {
                t.search_type = SearchType::Depth;
            }
        } else if buffer.contains(&"wtime") {
            self.main_thread.search_type = SearchType::Time;
            for t in self.workers.iter_mut() {
                t.search_type = SearchType::Time;
            }

            let mut clock = parse_time(buffer);
            clock.recommended_time(board.to_move);

            self.main_thread.game_time = clock;
            for t in self.workers.iter_mut() {
                t.game_time = clock;
            }
        } else {
            self.main_thread.search_type = SearchType::Infinite;
            for t in self.workers.iter_mut() {
                t.search_type = SearchType::Infinite;
            }
        }

        thread::scope(|s| {
            s.spawn(|| {
                search(&mut self.main_thread, true, *board, tt);
                self.halt.store(true, Ordering::Relaxed);
                println!("bestmove {}", self.main_thread.best_move.to_san());
            });
            for t in &mut self.workers {
                s.spawn(|| {
                    search(t, false, *board, tt);
                });
            }

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
