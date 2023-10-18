use std::sync::{Arc, RwLock};

use rustc_hash::FxHashMap;

use crate::board::board::Board;
use crate::board::fen::{build_board, STARTING_FEN};
use crate::engine::transposition::{get_table, TableEntry};
use crate::moves::movegenerator::MoveGenerator;
use crate::moves::movelist::MAX_LEN;
use crate::moves::moves::Move;
use crate::search::pvs::MAX_SEARCH_DEPTH;

use self::killers::{empty_killers, KillerMoves};
use self::pvs::{LMR_THRESHOLD, MIN_LMR_DEPTH};
use self::{game_time::GameTime, search_stats::SearchStats};

pub(crate) mod game_time;
pub mod killers;
pub(crate) mod pvs;
pub(crate) mod quiescence;
pub(crate) mod search_stats;
pub mod see;

#[derive(Clone)]
pub struct SearchInfo {
    pub board: Board,
    pub transpos_table: Arc<RwLock<FxHashMap<u64, TableEntry>>>,
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
            transpos_table: Arc::new(RwLock::new(get_table())),
            search_stats: Default::default(),
            game_time: Default::default(),
            search_type: Default::default(),
            iter_max_depth: 0,
            nmp_plies: 0,
            max_depth: MAX_SEARCH_DEPTH,
            killer_moves: empty_killers(),
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

#[inline(always)]
pub fn reduction(depth: i32, moves_played: i32) -> i32 {
    if depth <= MIN_LMR_DEPTH || moves_played < LMR_THRESHOLD {
        return 1;
    }
    let depth = depth as f32;
    let ply = moves_played as f32;
    let ret = 1. + depth.ln() * ply.ln() / 2.;
    ret as i32
}

#[inline(always)]
pub fn store_pv(pvs: &mut Vec<Move>, node_pvs: &mut Vec<Move>, m: Move) {
    pvs.clear();
    pvs.push(m);
    pvs.append(node_pvs);
}
