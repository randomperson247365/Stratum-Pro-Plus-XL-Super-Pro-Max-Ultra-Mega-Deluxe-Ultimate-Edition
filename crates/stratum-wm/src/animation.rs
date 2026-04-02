use std::time::{Duration, Instant};

/// Per-window slide-in animation. The window starts off-screen (below the
/// output) and eases to its computed layout position over `duration`.
pub struct WindowAnimation {
    pub start_x:  i32,
    pub start_y:  i32,
    pub target_x: i32,
    pub target_y: i32,
    pub started:  Instant,
    pub duration: Duration,
}

impl WindowAnimation {
    pub fn new(screen_height: i32, target_x: i32, target_y: i32) -> Self {
        Self {
            start_x:  target_x,
            start_y:  screen_height, // begin off-screen at the bottom
            target_x,
            target_y,
            started:  Instant::now(),
            duration: Duration::from_millis(220),
        }
    }

    /// Interpolated position using an ease-out cubic curve.
    /// At t=0 → (start_x, start_y); at t=1 → (target_x, target_y).
    pub fn current_pos(&self) -> (i32, i32) {
        let t = (self.started.elapsed().as_secs_f32()
            / self.duration.as_secs_f32())
        .clamp(0.0, 1.0);
        let ease = 1.0 - (1.0 - t).powi(3); // ease-out cubic
        let x = self.start_x + ((self.target_x - self.start_x) as f32 * ease) as i32;
        let y = self.start_y + ((self.target_y - self.start_y) as f32 * ease) as i32;
        (x, y)
    }

    pub fn is_done(&self) -> bool {
        self.started.elapsed() >= self.duration
    }

    /// Update the landing position each frame so the animation converges on
    /// the current computed layout target (handles windows that move while
    /// animating in, e.g. when a second window triggers re-layout).
    pub fn set_target(&mut self, x: i32, y: i32) {
        self.target_x = x;
        self.target_y = y;
    }
}
