use crate::{
    moves::movelist::MAX_LEN,
    search::search::MAX_SEARCH_DEPTH,
    spsa::{lmr_base, lmr_div},
};

type LmrReductions = [[i32; MAX_LEN + 1]; (MAX_SEARCH_DEPTH + 1) as usize];

pub struct LmrTable {
    lmr_table: LmrReductions,
    ln_table: [f64; MAX_LEN + 1],
}

impl LmrTable {
    pub fn new() -> Self {
        let mut ln_table = [0.0; MAX_LEN + 1];
        for x in 0..MAX_LEN + 1 {
            ln_table[x] = (x as f64).ln();
        }
        let mut a = Self { lmr_table: [[0; MAX_LEN + 1]; MAX_SEARCH_DEPTH as usize + 1], ln_table };
        a.init_lmr();
        a
    }

    fn init_lmr(&mut self) {
        for depth in 0..=MAX_SEARCH_DEPTH {
            for moves_played in 0..=MAX_LEN {
                let reduction =
                    (0.88 + (depth as f32).ln() * (moves_played as f32).ln() / 1.88) as i32;
                self.lmr_table[depth as usize][moves_played] = reduction;
            }
        }
        self.lmr_table[0][0] = 0;
        self.lmr_table[1][0] = 0;
        self.lmr_table[0][1] = 0;
    }

    pub(crate) fn base_reduction(&self, depth: i32, moves_played: i32) -> i32 {
        // self.lmr_table[depth.min(MAX_SEARCH_DEPTH) as usize][(moves_played as usize).min(MAX_LEN)]
        if depth == 0 || moves_played == 0 {
            return 0;
        }
        ((lmr_base() as f64 / 100.)
            + self.ln_table[depth.min(MAX_SEARCH_DEPTH) as usize]
                * self.ln_table[(moves_played as usize).min(MAX_LEN)]
                / (lmr_div() as f64 / 100.)) as i32
    }
}
