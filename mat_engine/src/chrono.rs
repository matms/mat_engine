//! Framerate and time based stuff
use crate::utils::unwrap_ref;

/// Returns 0.0 for the first frame, and the duration of the previous frame in seconds for all other frames.
///
/// TODO: Maybe offer an alternative that returns Option<f64>.
pub fn delta_time(ctx: &crate::EngineContext) -> f64 {
    unwrap_ref(&ctx.chrono_system)
        .last_frame_seconds
        .unwrap_or(0.0)
}

pub struct ChronoSystem {
    /// First frame is frame 0.
    frame_number: i64,
    /// Time when `start_new_frame()` was last called.
    curr_frame_start: std::time::SystemTime,
    /// Is None for the very first frame, should be Some(Duration) for all other frames.
    last_frame_duration: Option<std::time::Duration>,
    /// Basically the same as last_frame_duration, but represented as a f64 in seconds.
    last_frame_seconds: Option<f64>,
}

impl ChronoSystem {
    pub(crate) fn new() -> Self {
        Self {
            frame_number: 0,
            curr_frame_start: std::time::UNIX_EPOCH,
            last_frame_duration: None,
            last_frame_seconds: None,
        }
    }

    /// Starts a new frame and ends the previous*, generating the relevant delta time data for the previous frame.
    ///
    /// *except if this is the first frame.
    pub(crate) fn start_new_frame(&mut self) {
        // If this isn't the first frame, end the previous one
        if self.frame_number > 0 {
            self.end_prev_frame()
        }

        self.curr_frame_start = std::time::SystemTime::now();

        self.frame_number += 1;
    }

    /// Note: We currently call `end_prev_frame()` inside `start_new_frame()`, instead of calling end frame
    /// from the winit main loop's RedrawEventsCleared, which should only make a difference
    /// if a significant amount of time passes between the end of a frame and the beginning of the next.
    ///
    /// Still, we should think about this and figure out which approach is better.
    fn end_prev_frame(&mut self) {
        let end = std::time::SystemTime::now();
        let dur = end
            .duration_since(self.curr_frame_start)
            .expect("Time progressed non-monotonically, I think.");
        self.last_frame_duration = Some(dur);
        self.last_frame_seconds = Some(dur.as_secs_f64());
    }
}
