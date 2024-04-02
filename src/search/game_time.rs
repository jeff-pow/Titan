use std::time::{Duration, Instant};

use crate::types::pieces::Color;

const TIME_FRACTION: f64 = 0.67;

const GUI_DELAY: Duration = Duration::from_millis(25);

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Clock {
    /// Time increase for each side
    pub time_inc: [Duration; 2],
    /// Time remaining for each side
    pub time_remaining: [Duration; 2],
    /// Moves until the next time control
    pub movestogo: i32,
    /// Recommended time for search
    pub rec_time: Duration,
    /// Max time allowable for this search
    pub max_time: Duration,
}

impl Clock {
    /// Returns true if engine is unlikely to finish another depth of iterative deepening before
    /// time runs out for this search
    pub fn soft_termination(&self, search_start: Instant) -> bool {
        search_start.elapsed() > self.rec_time
    }

    /// Returns true if engine has used the max time allotted to this search
    pub fn hard_termination(&self, search_start: Instant) -> bool {
        search_start.elapsed() > self.max_time
    }

    /// Calculates a recommended amount of time to spend on a given search.
    pub fn recommended_time(&mut self, side: Color) {
        let clock = self.time_remaining[side] - GUI_DELAY;
        let time = clock / 20 + self.time_inc[side] * 3 / 4;
        self.rec_time = time.mul_f64(TIME_FRACTION);
        self.max_time = (time * 2).min(self.time_remaining[side]);
    }
}

impl Default for Clock {
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
