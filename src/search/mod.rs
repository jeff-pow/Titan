use std::sync::atomic::AtomicBool;
use std::sync::{Arc, RwLock};

use crate::board::board::Board;
use crate::board::fen::{build_board, STARTING_FEN};
use crate::engine::transposition::TranspositionTable;
use crate::moves::movelist::MAX_LEN;
use crate::moves::moves::Move;

use self::history_heuristics::MoveHistory;
use self::killers::{empty_killers, KillerMoves};
use self::search::{LMR_THRESHOLD, MAX_SEARCH_DEPTH, MIN_LMR_DEPTH};
use self::{game_time::GameTime, search_stats::SearchStats};

pub(crate) mod game_time;
pub mod history_heuristics;
pub mod killers;
pub(crate) mod quiescence;
pub mod search;
pub(crate) mod search_stats;
pub mod see;

#[derive(Clone)]
pub struct SearchInfo {
    pub board: Board,
    pub transpos_table: Arc<RwLock<TranspositionTable>>,
    pub search_stats: SearchStats,
    pub game_time: GameTime,
    pub search_type: SearchType,
    pub iter_max_depth: i32,
    pub max_depth: i32,
    pub nmp_plies: i32,
    pub killer_moves: KillerMoves,
    pub sel_depth: i32,
    pub lmr_reductions: LmrReductions,
    pub history: MoveHistory,
    pub halt: Arc<AtomicBool>,
    pub current_line: Vec<Move>,
}

impl Default for SearchInfo {
    fn default() -> Self {
        let table = TranspositionTable::default();
        Self {
            board: build_board(STARTING_FEN),
            transpos_table: Arc::new(RwLock::new(table)),
            search_stats: Default::default(),
            game_time: Default::default(),
            search_type: Default::default(),
            iter_max_depth: 0,
            nmp_plies: 0,
            max_depth: MAX_SEARCH_DEPTH,
            killer_moves: empty_killers(),
            sel_depth: 0,
            lmr_reductions: lmr_reductions(),
            history: MoveHistory::default(),
            halt: Arc::new(AtomicBool::from(false)),
            current_line: Vec::new(),
        }
    }
}

#[derive(Clone, Copy, Default, PartialEq)]
pub enum SearchType {
    Depth, // User has requested a search until a particular depth
    Time,  // Search determines how much time to allow itself
    #[default]
    Infinite, // Search forever
}

type LmrReductions = [[i32; MAX_LEN + 1]; (MAX_SEARCH_DEPTH + 1) as usize];
fn lmr_reductions() -> LmrReductions {
    let mut arr = [[0; MAX_LEN + 1]; (MAX_SEARCH_DEPTH + 1) as usize];
    for depth in 0..MAX_SEARCH_DEPTH + 1 {
        for moves_played in 0..MAX_LEN + 1 {
            arr[depth as usize][moves_played] = reduction(depth, moves_played as i32);
        }
    }
    arr
}

pub fn get_reduction(info: &SearchInfo, depth: i32, moves_played: i32) -> i32 {
    info.lmr_reductions[depth as usize][moves_played as usize]
}

pub fn reduction(depth: i32, moves_played: i32) -> i32 {
    if depth <= MIN_LMR_DEPTH || moves_played < LMR_THRESHOLD {
        return 1;
    }
    let depth = depth as f32;
    let ply = moves_played as f32;
    let ret = 1. + depth.ln() * ply.ln() / 2.;
    ret as i32
}

pub fn store_pv(pvs: &mut Vec<Move>, node_pvs: &mut Vec<Move>, m: Move) {
    pvs.clear();
    pvs.push(m);
    pvs.append(node_pvs);
}
