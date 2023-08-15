use std::time::{Duration, Instant};

use crate::types::pieces::Color;

/// Gives the system some wiggle room to communicate between the GUI and the engine
const TIME_BUFFER: Duration = Duration::from_millis(30);

const TIME_FRACTION: f64 = 0.3;

/// Limit the maximum time the engine thinks for
const MAX_THINK_TIME: Duration = Duration::from_millis(20000);

#[derive(Copy, Clone, Debug, Default)]
pub struct GameTime {
    /// Time increase for each side
    pub time_inc: [Duration; 2],
    /// Time remaining for each side
    pub time_remaining: [Duration; 2],
    /// Moves until the next time control
    pub movestogo: i32,
}

impl GameTime {
    /// If the function returns true and the search has not yet started, it means the side to play
    /// is out of time and any move should be played to avoid from dying.
    /// Otherwise returns false if the program should have time to finish another level of iterative
    /// deepening
    pub fn reached_termination(&self, search_start: Instant, recommended_time: Duration) -> bool {
        if recommended_time != Duration::ZERO {
            // If a previous iteration of iterative deepening hasn't finished in less than a small percentage of the time for the move, the
            // chances of the next iteration finishing before we go over allotted time are
            // basically none
            let target_elapsed_ms = recommended_time.as_millis() as f64 * TIME_FRACTION;
            let actual_elapsed_ms = search_start.elapsed().as_millis() as f64;
            if actual_elapsed_ms < target_elapsed_ms {
                return false;
            }
        }
        // Return true if the recommended_time is none, e.g. we have no time left whatsoever
        true
    }

    /// Returns a recommended amount of time to spend on a given search.
    /// Returns None if player is out of time and should play absolutely anything to keep
    /// themselves alive
    pub fn recommended_time(&mut self, side: Color) -> Duration {
        let clock = self.time_remaining[side as usize];
        // If engine has less than 50 ms to make a move, play anything to keep itself alive
        if clock < Duration::from_millis(50) {
            return Duration::ZERO;
        }
        let increment = self.time_inc[side as usize];
        let recommended_time = clock.div_f64(30.);
        let recommended_time = recommended_time.min(MAX_THINK_TIME);
        recommended_time + increment
    }
    // pub fn recommended_time(&mut self, side: Color, history_len: usize) -> Option<Duration> {
    //     let mut est_moves_left = AVG_NUMBER_MOVES - history_len as i32 / 2;
    //     if est_moves_left <= 0 {
    //         est_moves_left = 15;
    //     }
    //     let clock = self.time_remaining[side as usize];
    //     if clock == Duration::ZERO {
    //         return None;
    //     }
    //     let time_increase = self.time_inc[side as usize];
    //     let default_time_ms = clock.as_millis() / est_moves_left as u128;
    //     let recommended_time_ms = default_time_ms + time_increase.as_millis();
    //
    //     Some(Duration::from_millis(recommended_time_ms as u64))
    // }
}
