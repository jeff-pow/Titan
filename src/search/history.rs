use crate::{moves::moves::Move, types::pieces::Color};

const MAX_HIST_VAL: i32 = i16::MAX as i32;

#[derive(Clone)]
pub struct MoveHistory {
    // Indexed [side][src sq][dest sq]
    search_history: [[[i32; 64]; 64]; 2],
}

impl MoveHistory {
    fn update_search_history(&mut self, m: Move, bonus: i32, side: Color) {
        let i = &mut self.search_history[side as usize][m.origin_square().idx()][m.dest_square().idx()];
        *i += bonus - *i * bonus.abs() / MAX_HIST_VAL;
    }

    pub fn update_history(&mut self, m: Move, bonus: i32, side: Color) {
        self.update_search_history(m, bonus, side);
    }

    pub fn get_history(&self, m: Move, side: Color) -> i32 {
        self.get_search_history(m, side)
    }

    fn get_search_history(&self, m: Move, side: Color) -> i32 {
        self.search_history[side as usize][m.origin_square().idx()][m.dest_square().idx()]
    }
}

impl Default for MoveHistory {
    fn default() -> Self {
        Self {
            search_history: [[[0; 64]; 64]; 2],
        }
    }
}
