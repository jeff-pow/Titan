use std::array;

#[derive(Clone)]
pub struct LmrTable {
    pub lmr_table: [[i32; 64]; 64],
}

impl LmrTable {
    pub(crate) fn base_reduction(&self, depth: i32, moves_played: i32) -> i32 {
        self.lmr_table[(depth as usize).min(63)][(moves_played as usize).min(63)]
    }
}

impl Default for LmrTable {
    fn default() -> Self {
        Self {
            lmr_table: array::from_fn(|depth| {
                array::from_fn(|moves| (0.89 + (depth as f32).ln() * (moves as f32).ln() / 1.99) as i32)
            }),
        }
    }
}
