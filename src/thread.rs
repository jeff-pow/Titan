use std::{
    io,
    process::exit,
    sync::atomic::{AtomicBool, AtomicU64, Ordering},
    thread,
    time::{Duration, Instant},
};

use crate::{
    board::Board,
    chess_move::Move,
    eval::accumulator::{Accumulator, AccumulatorStack},
    history_table::{CaptureHistory, ContinuationHistory, CorrectionHistory, QuietHistory},
    search::{
        game_time::Clock,
        lmr_table::LmrTable,
        search::{start_search, Score, MAX_PLY},
        PVTable, SearchStack, SearchType,
    },
    transposition::TranspositionTable,
    uci::{parse_time, PRETTY_PRINT},
    utils::zeroed_box,
};

#[derive(Clone)]
pub struct ThreadData<'a> {
    pub ply: usize,
    pub min_nmp_ply: usize,
    /// Max depth reached by search (include qsearch)
    pub sel_depth: usize,
    pub iter_depth: i32,

    pub nodes_table: Box<[[u64; 64]; 64]>,
    pub nodes: AtomicCounter<'a>,
    pub stack: SearchStack,
    pub hash_history: Vec<u64>,
    pub accumulators: AccumulatorStack,
    pub pv: PVTable,

    pub quiet_hist: QuietHistory,
    pub capt_hist: CaptureHistory,
    pub cont_hist: ContinuationHistory,
    pub pawn_corr_hist: CorrectionHistory,

    pub search_start: Instant,
    thread_id: usize,
    pub search_types: Vec<SearchType>,
    halt: &'a AtomicBool,
    pub lmr: LmrTable,
}

impl<'a> ThreadData<'a> {
    pub(crate) fn new(
        halt: &'a AtomicBool,
        hash_history: Vec<u64>,
        thread_idx: usize,
        global_nodes: &'a AtomicU64,
    ) -> Self {
        Self {
            ply: 0,
            min_nmp_ply: 0,
            stack: SearchStack::default(),
            iter_depth: 0,
            sel_depth: 0,
            nodes: AtomicCounter::new(global_nodes),
            nodes_table: zeroed_box(),
            accumulators: AccumulatorStack::new(Accumulator::default()),
            quiet_hist: QuietHistory::default(),
            capt_hist: CaptureHistory::default(),
            cont_hist: ContinuationHistory::default(),
            pawn_corr_hist: CorrectionHistory::default(),
            halt,
            search_types: vec![SearchType::default()],
            hash_history,
            thread_id: thread_idx,
            lmr: LmrTable::default(),
            search_start: Instant::now(),
            pv: PVTable::default(),
        }
    }

    pub fn set_halt(&self, x: bool) {
        self.halt.store(x, Ordering::Relaxed);
    }

    pub fn halt(&self) -> bool {
        self.halt.load(Ordering::Relaxed)
    }

    pub(super) fn node_tm_stop(&self, game_time: Clock, depth: i32) -> bool {
        if depth > 7 {
            let Some(m) = self.pv.best_move() else { return false };
            let mut limit = game_time.rec_time.as_secs_f32();
            let frac = self.nodes_table[m.from()][m.to()] as f32 / self.nodes.local_count() as f32;
            limit *= frac.mul_add(-1.5, 2.0);
            if self.search_start.elapsed() >= Duration::from_secs_f32(limit) {
                return true;
            }
        }
        false
    }

    pub(super) fn soft_stop(&self, depth: i32, prev_score: i32) -> bool {
        self.search_types.iter().any(|&search_type| match search_type {
            SearchType::Depth(d) => depth >= d,
            SearchType::Time(time) => {
                self.main_thread() && self.node_tm_stop(time, depth) || time.soft_termination(self.search_start)
            }
            SearchType::Nodes(n) => self.nodes.global_count() >= n,
            SearchType::Infinite => self.halt.load(Ordering::Relaxed),
            SearchType::Mate(d) => {
                let dist = if prev_score.is_positive() {
                    (Score::CHECKMATE - prev_score + 1) / 2
                } else {
                    -(Score::CHECKMATE + prev_score) / 2
                };
                dist.abs() <= d.abs() || depth > MAX_PLY as i32
            }
            SearchType::MoveTime(time) => self.search_start.elapsed() > time,
        })
    }

    pub(super) fn hard_stop(&self) -> bool {
        self.search_types.iter().any(|&search_type| match search_type {
            SearchType::Mate(_) | SearchType::Depth(_) | SearchType::Infinite => self.halt.load(Ordering::Relaxed),
            SearchType::Time(time) => self.nodes.check_time() && time.hard_termination(self.search_start),
            SearchType::Nodes(n) => self.nodes.global_count() >= n,
            SearchType::MoveTime(time) => self.nodes.check_time() && self.search_start.elapsed() > time,
        })
    }

    pub(crate) fn update_histories(
        &mut self,
        best_move: Move,
        quiets_tried: &[Move],
        tacticals_tried: &[Move],
        board: &Board,
        depth: i32,
    ) {
        let bonus = (238 * depth).min(2095);
        let best_piece = board.piece_at(best_move.from());

        if best_move.is_tactical(board) {
            self.capt_hist.update(best_move, best_piece, board, bonus);
        } else {
            if depth > 3 || quiets_tried.len() > 1 {
                self.quiet_hist.update(best_move, best_piece, bonus);
                self.cont_hist.update(best_move, best_piece, &self.stack, self.ply - 1, bonus);
                self.cont_hist.update(best_move, best_piece, &self.stack, self.ply - 2, bonus);
            }
            // Only penalize quiets if best_move was quiet
            for m in quiets_tried {
                if *m == best_move {
                    continue;
                }
                let p = board.piece_at(m.from());
                self.quiet_hist.update(*m, p, -bonus);
                self.cont_hist.update(*m, p, &self.stack, self.ply - 1, -bonus);
                self.cont_hist.update(*m, p, &self.stack, self.ply - 2, -bonus);
            }
        }

        // Always penalize tacticals since they should always be good no matter what the position
        for m in tacticals_tried {
            if *m == best_move {
                continue;
            }
            let p = board.piece_at(m.from());
            self.capt_hist.update(*m, p, board, -bonus);
        }
    }

    pub(super) fn print_search_stats(&self, score: i32, tt: &TranspositionTable, depth: i32) {
        let nodes = self.nodes.global_count();
        let time_elapsed = self.search_start.elapsed().as_millis();
        let nps = (nodes as f64 / self.search_start.elapsed().as_secs_f64()) as i64;
        let hashfull = tt.permille_usage();
        let pv_line: String = self.pv.pv().map(|m| m.to_san()).collect::<Vec<String>>().join(" ");

        if PRETTY_PRINT.load(Ordering::Relaxed) {
            if depth == 1 {
                println!(
                    "{:<10} {:<9} {:<10} {:<8} {:<9} {:<9} PV",
                    "Time(ms)", "Depth", "Nodes", "kNPS", "Score", "Hashfull",
                );
                println!("{:-<10} {:-<9} {:-<10} {:-<8} {:-<9} {:-<9} {:-<20}", "", "", "", "", "", "", "");
            }
            let formatted_nps = (nps / 1000)
                .to_string()
                .as_bytes()
                .rchunks(3)
                .rev()
                .map(std::str::from_utf8)
                .collect::<Result<Vec<&str>, _>>()
                .unwrap()
                .join(",");
            println!(
                "{:<10} {:<9} {:<10} {:<8} {:<9} {:<9} {}",
                time_elapsed,
                format!("{depth}/{}", self.sel_depth),
                nodes,
                formatted_nps,
                {
                    if Score::mate_found(score) {
                        if score.is_positive() {
                            format!("mate {}", (Score::CHECKMATE - score + 1) / 2)
                        } else {
                            format!("mate {}", -(Score::CHECKMATE + score) / 2)
                        }
                    } else {
                        format!(
                            "{}{:.2}",
                            if score.is_positive() {
                                "+"
                            } else if score.is_negative() {
                                "-"
                            } else {
                                " "
                            },
                            f64::from(score) / 100.
                        )
                    }
                },
                format!("{}%", hashfull as f64 / 10.),
                pv_line,
            );
        } else {
            print!(
                "info time {} depth {} seldepth {} nodes {} nps {} score ",
                time_elapsed, depth, self.sel_depth, nodes, nps,
            );

            if Score::mate_found(score) {
                if score.is_positive() {
                    print!("mate {}", (Score::CHECKMATE - score + 1) / 2);
                } else {
                    print!("mate {}", (-(Score::CHECKMATE + score) / 2));
                }
            } else {
                print!("cp {score}");
            }

            print!(" hashfull {hashfull} pv {pv_line} ");
            println!();
        }
    }

    pub const fn main_thread(&self) -> bool {
        self.thread_id == 0
    }
}

pub struct ThreadPool<'a> {
    pub threads: Vec<ThreadData<'a>>,
}

impl<'a> ThreadPool<'a> {
    pub fn new(halt: &'a AtomicBool, hash_history: Vec<u64>, global_nodes: &'a AtomicU64) -> Self {
        Self { threads: vec![ThreadData::new(halt, hash_history, 0, global_nodes)] }
    }

    /// This thread creates a number of workers equal to threads - 1. If 4 threads are requested,
    /// the main thread counts as one and then the remaining three are placed in the worker queue.
    pub fn add_workers(&mut self, threads: usize) {
        // Might as well use whatever history values the main thread has if any.
        self.threads = vec![self.threads[0].clone(); threads];
        for (idx, t) in self.threads.iter_mut().enumerate() {
            t.thread_id = idx;
        }
    }

    pub fn reset(&mut self, halt: &'a AtomicBool, global_nodes: &'a AtomicU64) {
        let len = self.threads.len();
        self.threads.clear();
        for i in 0..len {
            self.threads.push(ThreadData::new(halt, vec![], i, global_nodes));
        }
    }

    pub fn handle_go(
        &mut self,
        buffer: &[&str],
        board: &Board,
        halt: &AtomicBool,
        msg: &mut Option<String>,
        hash_history: &[u64],
        tt: &TranspositionTable,
    ) {
        halt.store(false, Ordering::Relaxed);

        for t in &mut self.threads {
            hash_history.clone_into(&mut t.hash_history);
            t.search_types = vec![SearchType::Infinite];
            t.nodes.reset();
        }

        let mut iter = buffer.iter().skip(1).peekable();
        while let Some(&limit) = iter.next() {
            match limit {
                "depth" => {
                    if let Some(depth_str) = iter.next()
                        && let Ok(depth) = depth_str.parse()
                    {
                        for t in &mut self.threads {
                            t.search_types.push(SearchType::Depth(depth));
                        }
                    }
                }
                "nodes" => {
                    if let Some(nodes_str) = iter.next()
                        && let Ok(nodes) = nodes_str.parse()
                    {
                        for t in &mut self.threads {
                            t.search_types.push(SearchType::Nodes(nodes));
                        }
                    }
                }
                "wtime" | "btime" | "winc" | "binc" | "movestogo" => {
                    let mut clock = parse_time(buffer);
                    clock.recommended_time(board.stm());
                    for t in &mut self.threads {
                        t.search_types.push(SearchType::Infinite);
                    }
                    self.threads[0].search_types.push(SearchType::Time(clock));
                    while iter.peek().is_some_and(|t| matches!(**t, "wtime" | "btime" | "winc" | "binc" | "movestogo"))
                    {
                        iter.next();
                    }
                }
                "mate" => {
                    if let Some(ply_str) = iter.next()
                        && let Ok(ply) = ply_str.parse()
                    {
                        for t in &mut self.threads {
                            t.search_types.push(SearchType::Mate(ply));
                        }
                    }
                }
                "movetime" => {
                    if let Some(time_str) = iter.next()
                        && let Ok(ms) = time_str.parse()
                    {
                        for t in &mut self.threads {
                            t.search_types.push(SearchType::MoveTime(Duration::from_millis(ms)));
                        }
                    }
                }
                _ => {}
            }
        }

        thread::scope(|s| {
            for t in &mut self.threads {
                s.spawn(|| {
                    start_search(t, t.main_thread(), *board, tt);
                    halt.store(true, Ordering::Relaxed);
                    if t.main_thread() {
                        println!("bestmove {}", t.pv.best_move().unwrap().to_san());
                    }
                });
            }

            let mut s = String::new();
            let len_read = io::stdin().read_line(&mut s).unwrap();
            if len_read == 0 {
                // Stdin closed, exit for openbench
                exit(0);
            }
            match s.as_str().trim() {
                "isready" => println!("readyok"),
                "quit" => exit(0),
                "stop" => halt.store(true, Ordering::Relaxed),
                _ => {
                    *msg = Some(s);
                }
            }
        });
        tt.age_up();
    }
}

#[derive(Clone)]
pub struct AtomicCounter<'a> {
    global_nodes: &'a AtomicU64,
    local_nodes: u64,
    batch: u64,
}

const UPDATE_FREQ: u64 = 1024;

impl<'a> AtomicCounter<'a> {
    const fn new(global_nodes: &'a AtomicU64) -> Self {
        Self { global_nodes, local_nodes: 0, batch: 0 }
    }

    pub(crate) fn global_count(&self) -> u64 {
        self.global_nodes.load(Ordering::Relaxed) + self.batch
    }

    pub(crate) const fn local_count(&self) -> u64 {
        self.local_nodes + self.batch
    }

    pub(crate) fn increment(&mut self) {
        self.batch += 1;
        if self.batch > UPDATE_FREQ {
            self.local_nodes += self.batch;
            self.global_nodes.fetch_add(self.batch, Ordering::Relaxed);
            self.batch = 0;
        }
    }

    pub(crate) fn reset(&mut self) {
        self.batch = 0;
        self.local_nodes = 0;
        self.global_nodes.store(0, Ordering::Relaxed);
    }

    pub(crate) const fn check_time(&self) -> bool {
        self.batch == 0
    }
}

#[cfg(test)]
mod search_tests {
    use super::ThreadData;
    use crate::{
        board::Board,
        search::{search::start_search, SearchType},
        transposition::{TranspositionTable, TARGET_TABLE_SIZE_MB},
    };
    use std::sync::atomic::{AtomicBool, AtomicU64};

    #[test]
    fn go_nodes() {
        let transpos_table = TranspositionTable::new(TARGET_TABLE_SIZE_MB);
        let halt = AtomicBool::new(false);
        let global_nodes = AtomicU64::new(0);

        let mut thread = ThreadData::new(&halt, Vec::new(), 0, &global_nodes);

        thread.search_types.push(SearchType::Nodes(12345));

        start_search(&mut thread, false, Board::default(), &transpos_table);

        assert_eq!(thread.nodes.local_count(), thread.nodes.global_count());
        assert_eq!(12345, thread.nodes.global_count());
    }

    #[test]
    fn go_mate() {
        let tt = TranspositionTable::new(TARGET_TABLE_SIZE_MB);
        let halt = AtomicBool::new(false);
        let global_nodes = AtomicU64::new(0);

        let mut thread = ThreadData::new(&halt, Vec::new(), 0, &global_nodes);

        thread.search_types.push(SearchType::Mate(2));
        thread.search_types.push(SearchType::Nodes(1_000_000));

        start_search(&mut thread, false, Board::from_fen("4k1K1/3n4/2N5/4N3/8/8/8/8 w - - 0 1"), &tt);

        assert_eq!("e5g4", thread.pv.best_move().unwrap().to_san());
        let pv = thread.pv.pv().collect::<Vec<_>>();
        assert_eq!("g4f6", pv[2].to_san());
    }
}
