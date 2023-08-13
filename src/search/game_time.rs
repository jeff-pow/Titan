use std::time::{Duration, Instant};

use crate::{types::pieces::Color};

/// Average number of moves (per side) in a chess game according to a quick google search :)
/// I subtracted a few moves because I don't think most games go on that long and 
/// this gives the engine a little more time to search
const AVG_NUMBER_MOVES: i32 = 30;

/// Gives the system some wiggle room to communicate between the GUI and the engine
const TIME_BUFFER: Duration = Duration::from_millis(30);

struct GameTime {
    /// Time increase for each side 
    time_inc: [Duration; 2],
    /// Time remaining for each side
    time_remaining: [Duration; 2],
    /// Moves until the next time control
    movestogo: i32,
}

impl GameTime {
    /// If the function returns false and the search has not yet started, it means the side to play
    /// is out of time and any move should be played to avoid from dying. Otherwise calculates
    /// whether or not the program will have time to finish another layer of iterative deepening
    pub fn reached_termination(&self, side: Color, search_start: Instant, history_len: usize) -> bool {
        let recommended_time = self.recommended_time(side, history_len);
        if let Some(recommended_time) = recommended_time {
            // If a previous iteration of iterative deepening hasn't finished in less than 30% of the time for the move, the
            // chances of the next iteration finishing before we go over allotted time are
            // basically none
            let target_elapsed_ms = recommended_time.as_millis() as f64 * 0.3;
            let actual_elapsed_ms = search_start.elapsed().as_millis() as f64;
            if actual_elapsed_ms < target_elapsed_ms {
                return true;
            }
        }
        // Return false if the recommended_time is none, e.g. we have no time left whatsoever
        false
    }

    /// Returns a recommended amount of time to spend on a given search. 
    /// Returns None if player is out of time and should play absolutely anything to keep
    /// themselves alive
    fn recommended_time(&self, side: Color, history_len: usize) -> Option<Duration> {
        let est_moves_left = AVG_NUMBER_MOVES - history_len as i32 / 2;
        let clock = self.time_remaining[side as usize];
        if clock < TIME_BUFFER {
            return None;
        }
        let time_increase = self.time_inc[side as usize];
        let default_time_ms = clock.as_millis() / est_moves_left as u128;
        let recommended_time_ms = default_time_ms + time_increase.as_millis();
        
        Some(Duration::from_millis(recommended_time_ms as u64))
    }
}

impl Default for GameTime {
    fn default() -> Self {
        Self { 
            movestogo: Default::default(),
            time_inc: Default::default(),
            time_remaining: Default::default(),
        }
    }
}
