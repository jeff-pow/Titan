type LmrReductions = [[i32; 64]; 64];
pub struct LmrTable {
    pub lmr_table: LmrReductions,
}

impl LmrTable {
    pub fn new() -> Self {
        let mut a = Self { lmr_table: [[0; 64]; 64] };
        a.init_lmr();
        a
    }

    fn init_lmr(&mut self) {
        for depth in 0..64 {
            for moves_played in 0..64 {
                let reduction = (0.89 + (depth as f32).ln() * (moves_played as f32).ln() / 1.99) as i32;
                self.lmr_table[depth][moves_played] = reduction;
            }
        }
        self.lmr_table[0][0] = 0;
        self.lmr_table[1][0] = 0;
        self.lmr_table[0][1] = 0;
    }

    pub(crate) fn base_reduction(&self, depth: i32, moves_played: i32) -> i32 {
        self.lmr_table[(depth as usize).min(63)][(moves_played as usize).min(63)]
    }
}
