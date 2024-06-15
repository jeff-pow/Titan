use crate::board::Board;

#[derive(Clone)]
pub struct CorrectionTable {
    table: [[i32; TABLE_SIZE]; 2],
}

impl Default for CorrectionTable {
    fn default() -> Self {
        Self { table: [[0; TABLE_SIZE]; 2] }
    }
}

const GRAIN: i32 = 256;
const TABLE_SIZE: usize = 16384;
const MAX: i32 = 12288;
const WEIGHT_SCALE: i32 = 512;
impl CorrectionTable {
    pub(crate) fn corrected_eval(&self, board: &Board, raw_eval: i32) -> i32 {
        let entry = self.table[board.stm][(board.pawn_hash % TABLE_SIZE as u64) as usize];
        raw_eval + entry / GRAIN
    }

    pub(crate) fn update_table(&mut self, depth: i32, corrected_eval: i32, search_score: i32, board: &Board) {
        let entry = &mut self.table[board.stm][(board.pawn_hash % TABLE_SIZE as u64) as usize];
        let scaled_error = (search_score - corrected_eval) * GRAIN;
        let new_weight = (depth * depth + depth + 1).min(64);
        let updated_val = *entry * (WEIGHT_SCALE - new_weight) + scaled_error * new_weight;
        *entry = (updated_val / WEIGHT_SCALE).clamp(-MAX, MAX);
    }
}
