use rustc_hash::FxHashMap;

use crate::moves::movegenerator::MoveGenerator;
use crate::moves::moves::Move;
use crate::search::pvs::MAX_SEARCH_DEPTH;
use crate::{board::board::Board, engine::transposition::TableEntry};

use self::killers::{KillerMoves, NUM_KILLER_MOVES};
use self::{game_time::GameTime, search_stats::SearchStats};

pub(crate) mod eval;
pub(crate) mod game_time;
pub mod killers;
pub mod mtdf;
pub(crate) mod pvs;
pub(crate) mod quiescence;
pub(crate) mod search_stats;
mod alpha_beta;

pub struct SearchInfo {
    pub board: Board,
    pub transpos_table: FxHashMap<u64, TableEntry>,
    pub search_stats: SearchStats,
    pub game_time: GameTime,
    pub search_type: SearchType,
    pub iter_max_depth: i8,
    pub max_depth: i8,
    pub killer_moves: KillerMoves,
    pub sel_depth: i8,
    pub mg: MoveGenerator,
}

impl Default for SearchInfo {
    fn default() -> Self {
        Self {
            board: Default::default(),
            transpos_table: Default::default(),
            search_stats: Default::default(),
            game_time: Default::default(),
            search_type: Default::default(),
            iter_max_depth: 0,
            max_depth: MAX_SEARCH_DEPTH,
            killer_moves: [[Move::NULL; NUM_KILLER_MOVES]; MAX_SEARCH_DEPTH as usize],
            sel_depth: 0,
            mg: MoveGenerator::default(),
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
