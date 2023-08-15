use rustc_hash::FxHashMap;

use crate::{board::board::Board, engine::transposition::TableEntry};

use self::{game_time::GameTime, search_stats::SearchStats};

pub(crate) mod alpha_beta;
pub(crate) mod eval;
pub(crate) mod game_time;
pub(crate) mod quiescence;
pub(crate) mod search_stats;

#[derive(Default)]
pub struct SearchInfo {
    pub board: Board,
    pub transpos_table: FxHashMap<u64, TableEntry>,
    pub search_stats: SearchStats,
    pub game_time: GameTime,
    pub search_type: SearchType,
    pub depth: i8,
}

#[derive(Clone, Copy, Default, PartialEq)]
pub enum SearchType {
    Depth, // User has requested a search until a particular depth
    #[default]
    Time, // Search determines how much time to allow itself
}
