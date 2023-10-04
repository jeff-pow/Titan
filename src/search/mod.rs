use rustc_hash::FxHashMap;

use crate::board::fen::{build_board, STARTING_FEN};
use crate::engine::transposition::get_table;
use crate::moves::movegenerator::MoveGenerator;
use crate::moves::movelist::MAX_LEN;
use crate::moves::moves::Move;
use crate::search::pvs::MAX_SEARCH_DEPTH;
use crate::{board::board::Board, engine::transposition::TableEntry};

use self::killers::{KillerMoves, NUM_KILLER_MOVES};
use self::{game_time::GameTime, search_stats::SearchStats};

mod alpha_beta;
pub(crate) mod game_time;
pub mod killers;
pub(crate) mod pvs;
pub(crate) mod quiescence;
pub(crate) mod search_stats;

#[derive(Clone)]
pub struct SearchInfo {
    pub board: Board,
    // pub transpos_table: FxHashMap<u64, TableEntry>,
    pub search_stats: SearchStats,
    pub game_time: GameTime,
    pub search_type: SearchType,
    pub iter_max_depth: i32,
    pub max_depth: i32,
    pub nmp_plies: i32,
    pub killer_moves: KillerMoves,
    pub sel_depth: i32,
    pub mg: MoveGenerator,
    pub lmr_reductions: LmrReductions,
}

impl Default for SearchInfo {
    fn default() -> Self {
        Self {
            board: build_board(STARTING_FEN),
            // transpos_table: get_table(),
            search_stats: Default::default(),
            game_time: Default::default(),
            search_type: Default::default(),
            iter_max_depth: 0,
            nmp_plies: 0,
            max_depth: MAX_SEARCH_DEPTH,
            killer_moves: [[Move::NULL; NUM_KILLER_MOVES]; MAX_SEARCH_DEPTH as usize],
            sel_depth: 0,
            mg: MoveGenerator::default(),
            lmr_reductions: lmr_reductions(),
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

/// Begin LMR if more than this many moves have been searched
const REDUCTION_THRESHOLD: i32 = 2;
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

pub fn get_reduction(search_info: &SearchInfo, depth: i32, moves_played: i32) -> i32 {
    search_info.lmr_reductions[depth as usize][moves_played as usize]
}

#[inline(always)]
pub fn reduction(depth: i32, moves_played: i32) -> i32 {
    if depth == 0 || moves_played < REDUCTION_THRESHOLD {
        return 0;
    }
    let depth = depth as f32;
    let ply = moves_played as f32;
    let ret = 1. + depth.ln() * ply.ln() / 2.;
    ret as i32
}
