use crate::{movelist::MAX_LEN, search::search::MAX_SEARCH_DEPTH};

type LmrReductions = [[i32; MAX_LEN + 1]; (MAX_SEARCH_DEPTH + 1) as usize];

pub struct LmrTable {
    pub lmr_table: LmrReductions,
}

impl LmrTable {
    pub fn new() -> Self {
        let mut a = Self { lmr_table: [[0; MAX_LEN + 1]; MAX_SEARCH_DEPTH as usize + 1] };
        a.init_lmr();
        a
    }

    fn init_lmr(&mut self) {
        for depth in 0..=MAX_SEARCH_DEPTH {
            for moves_played in 0..=MAX_LEN {
                let reduction = (0.88 + (depth as f32).ln() * (moves_played as f32).ln() / 1.88) as i32;
                self.lmr_table[depth as usize][moves_played] = reduction;
            }
        }
        self.lmr_table[0][0] = 0;
        self.lmr_table[1][0] = 0;
        self.lmr_table[0][1] = 0;
    }

    pub(crate) fn base_reduction(&self, depth: i32, moves_played: i32) -> i32 {
        self.lmr_table[depth.min(MAX_SEARCH_DEPTH) as usize][(moves_played as usize).min(MAX_LEN)]
    }
}
