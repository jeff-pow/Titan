use std::time::{Duration, Instant};

use strum::IntoEnumIterator;

use crate::types::pieces::Color;

const TIME_FRACTION: f64 = 0.60;

const GUI_DELAY: Duration = Duration::from_millis(25);

#[derive(Copy, Clone, Debug)]
pub struct GameTime {
    /// Time increase for each side
    pub time_inc: [Duration; 2],
    /// Time remaining for each side
    pub time_remaining: [Duration; 2],
    /// Moves until the next time control
    pub movestogo: i32,
    /// Recommended time
    pub rec_time: Duration,
    /// Max time the side may spend on the move
    pub max_time: Duration,
}

impl GameTime {
    /// If the function returns true and the search has not yet started, it means the side to play
    /// is out of time and any move should be played to avoid from dying.
    /// Otherwise returns false if the program should have time to finish another level of iterative
    /// deepening
    pub fn soft_termination(&self, search_start: Instant) -> bool {
        search_start.elapsed() > self.rec_time
    }

    pub fn hard_termination(&self, search_start: Instant) -> bool {
        search_start.elapsed() > self.max_time
    }

    /// Returns a recommended amount of time to spend on a given search.
    /// Returns None if player is out of time and should play absolutely anything to keep
    /// themselves alive
    pub fn recommended_time(&mut self, side: Color) {
            let clock = self.time_remaining[side] - GUI_DELAY;
            // If engine has less than GUI_DELAY ms to make a move, play anything to keep itself alive
            let time = clock / 20 + self.time_inc[side] * 1 / 2;
            self.rec_time = time.mul_f64(TIME_FRACTION);
            self.max_time = (time * 2).min(self.time_remaining[side]);
    }
}

impl Default for GameTime {
    fn default() -> Self {
        Self {
            time_inc: Default::default(),
            time_remaining: Default::default(),
            movestogo: Default::default(),
            rec_time: Duration::MAX,
            max_time: Duration::MAX,
        }
    }
}
