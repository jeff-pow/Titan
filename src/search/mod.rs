use rustc_hash::FxHashMap;

use crate::moves::moves::Move;
use crate::search::pvs::MAX_SEARCH_DEPTH;
use crate::{board::board::Board, engine::transposition::TableEntry};

use self::{game_time::GameTime, search_stats::SearchStats};

pub(crate) mod eval;
pub(crate) mod game_time;
pub(crate) mod pvs;
pub(crate) mod quiescence;
pub(crate) mod search_stats;

pub struct SearchInfo {
    pub board: Board,
    pub transpos_table: FxHashMap<u64, TableEntry>,
    pub search_stats: SearchStats,
    pub game_time: GameTime,
    pub search_type: SearchType,
    pub iter_max_depth: i8,
    pub killer_moves: [[Move; 2]; MAX_SEARCH_DEPTH as usize],
    pub sel_depth: i8,
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
            killer_moves: [[Move::NULL; 2]; MAX_SEARCH_DEPTH as usize],

            sel_depth: 0,
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
