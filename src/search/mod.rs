use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use lazy_static::lazy_static;

use crate::board::board::Board;
use crate::board::fen::{build_board, STARTING_FEN};
use crate::engine::transposition::TranspositionTable;
use crate::moves::movelist::MAX_LEN;
use crate::moves::moves::Move;

use self::history_heuristics::MoveHistory;
use self::search::MAX_SEARCH_DEPTH;
use self::{game_time::GameTime, search_stats::SearchStats};

pub(crate) mod game_time;
pub mod history_heuristics;
pub mod killers;
pub(crate) mod quiescence;
pub mod search;
pub(crate) mod search_stats;
pub mod see;

// Tunable Constants
/// Initial aspiration window value
pub const INIT_ASP: i32 = 10;
/// Begin LMR if more than this many moves have been searched
pub const LMR_THRESHOLD: i32 = 2;
pub const MIN_LMR_DEPTH: i32 = 2;
pub const MAX_LMP_DEPTH: i32 = 6;
pub const LMP_CONST: i32 = 3;
pub const RFP_MULTIPLIER: i32 = 70;
pub const MAX_RFP_DEPTH: i32 = 9;
pub const MIN_NMP_DEPTH: i32 = 3;
pub const MIN_IIR_DEPTH: i32 = 4;

#[derive(Clone)]
pub struct SearchInfo {
    pub board: Board,
    pub transpos_table: TranspositionTable,
    pub search_stats: SearchStats,
    pub game_time: GameTime,
    pub search_type: SearchType,
    pub iter_max_depth: i32,
    pub max_depth: i32,
    pub nmp_plies: i32,
    pub sel_depth: i32,
    pub history: MoveHistory,
    pub halt: Arc<AtomicBool>,
    pub current_line: Vec<Move>,
    stack: [PlyEntry; MAX_SEARCH_DEPTH as usize],
}

impl Default for SearchInfo {
    fn default() -> Self {
        Self {
            board: build_board(STARTING_FEN),
            transpos_table: TranspositionTable::default(),
            search_stats: Default::default(),
            game_time: Default::default(),
            search_type: Default::default(),
            iter_max_depth: 0,
            nmp_plies: 0,
            max_depth: MAX_SEARCH_DEPTH,
            sel_depth: 0,
            history: MoveHistory::default(),
            halt: Arc::new(AtomicBool::from(false)),
            current_line: Vec::new(),
            stack: [PlyEntry::default(); MAX_SEARCH_DEPTH as usize],
        }
    }
}

#[derive(Clone, Copy, Default)]
struct PlyEntry {
    pub killers: [Move; 2],
    pub played_move: Move,
    pub eval: i32,
}

#[derive(Clone, Copy, Default, PartialEq)]
pub enum SearchType {
    Depth, // User has requested a search until a particular depth
    Time,  // Search determines how much time to allow itself
    #[default]
    Infinite, // Search forever
}

lazy_static! {
    pub static ref LMR_REDUCTIONS: LmrReductions = lmr_reductions();
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

pub fn get_reduction(depth: i32, moves_played: i32) -> i32 {
    LMR_REDUCTIONS[depth as usize][moves_played as usize]
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
