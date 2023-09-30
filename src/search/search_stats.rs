use std::time::Instant;

#[derive(Clone)]
pub struct SearchStats {
    pub nodes_searched: u64,
    pub start: Instant,
}

impl Default for SearchStats {
    fn default() -> Self {
        Self {
            nodes_searched: Default::default(),
            start: Instant::now(),
        }
    }
}
