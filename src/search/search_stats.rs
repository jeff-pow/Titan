use std::time::Instant;

pub struct SearchStats {
    nodes_searched: u64,
    start: Instant,
}

impl Default for SearchStats {
    fn default() -> Self {
        Self { nodes_searched: Default::default(), start: Instant::now() }
    }
}
