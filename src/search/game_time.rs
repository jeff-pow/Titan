use std::time::{Duration, Instant};

use strum::IntoEnumIterator;

use crate::types::pieces::Color;

const TIME_FRACTION: f64 = 0.40;

/// Limit the maximum time the engine thinks for
const MAX_THINK_TIME: Duration = Duration::from_millis(15000);

const GUI_DELAY: Duration = Duration::from_millis(50);

#[derive(Copy, Clone, Debug, Default)]
pub struct GameTime {
    /// Time increase for each side
    pub time_inc: [Duration; 2],
    /// Time remaining for each side
    pub time_remaining: [Duration; 2],
    /// Moves until the next time control
    pub movestogo: i32,
    pub time_recommendation: [Duration; 2],
}

impl GameTime {
    /// If the function returns true and the search has not yet started, it means the side to play
    /// is out of time and any move should be played to avoid from dying.
    /// Otherwise returns false if the program should have time to finish another level of iterative
    /// deepening
    pub fn soft_termination(&self, search_start: Instant, side: Color) -> bool {
        if self.time_recommendation[side].as_millis() > GUI_DELAY.as_millis() {
            // If a previous iteration of iterative deepening hasn't finished in less than a small percentage of the time for the move, the
            // chances of the next iteration finishing before we go over allotted time are
            // basically none
            let target_elapsed = self.time_recommendation[side].mul_f64(TIME_FRACTION);
            let actual_elapsed = search_start.elapsed();
            if actual_elapsed < target_elapsed {
                return false;
            }
        }
        // Return true if the recommended_time is none, e.g. we have no time left whatsoever
        true
    }

    pub fn hard_termination(&self, search_start: Instant, side: Color) -> bool {
        search_start.elapsed() > self.time_recommendation[side]
    }

    /// Returns a recommended amount of time to spend on a given search.
    /// Returns None if player is out of time and should play absolutely anything to keep
    /// themselves alive
    pub fn recommended_time(&mut self) {
        for side in Color::iter() {
            let clock = self.time_remaining[side];
            // If engine has less than GUI_DELAY ms to make a move, play anything to keep itself alive
            if clock < GUI_DELAY {
                self.time_recommendation[side] = Duration::ZERO;
            }
            let increment = self.time_inc[side] / 2;
            let recommended_time = clock.div_f64(20.);
            let recommended_time = recommended_time.min(MAX_THINK_TIME);
            self.time_recommendation[side] = recommended_time + increment;
        }
    }

    pub fn unlimited_time(&mut self) {
        self.time_recommendation = [Duration::MAX; 2];
    }
}
