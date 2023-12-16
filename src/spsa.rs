use crate::{
    moves::movelist::MAX_LEN,
    search::{reduction, search::MAX_SEARCH_DEPTH, LMR_REDUC},
};

fn update_lmr() {
    for depth in 0..MAX_SEARCH_DEPTH + 1 {
        for moves_played in 0..MAX_LEN + 1 {
            unsafe {
                LMR_REDUC[depth as usize][moves_played] = reduction(depth, moves_played as i32);
            }
        }
    }
}
