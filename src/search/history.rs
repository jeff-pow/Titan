use crate::{moves::moves::Move, types::pieces::Color};

pub fn update_history(history: &mut [[[i64; 64]; 64]; 2], m: Move, bonus: i64, side: Color) {
    let i = &mut history[side.idx()][m.origin_square().idx()][m.dest_square().idx()];
    *i += bonus - *i * bonus.abs() / i64::from(i16::MAX);
}
