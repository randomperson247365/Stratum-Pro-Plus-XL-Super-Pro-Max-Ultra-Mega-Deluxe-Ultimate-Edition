use super::{Layout, Rect};

/// Default floating layout: new windows open centered with a reasonable default size,
/// with a slight cascade offset per subsequent window.
pub struct FloatingLayout {
    cascade_offset: u32,
}

impl FloatingLayout {
    pub fn new() -> Self {
        Self { cascade_offset: 0 }
    }
}

impl Default for FloatingLayout {
    fn default() -> Self {
        Self::new()
    }
}

impl Layout for FloatingLayout {
    fn window_added(&mut self, output_width: u32, output_height: u32) -> Rect {
        let default_w = (output_width * 2 / 3).max(640);
        let default_h = (output_height * 2 / 3).max(480);

        let offset = self.cascade_offset;
        self.cascade_offset = (self.cascade_offset + 24) % 200;

        let x = ((output_width.saturating_sub(default_w)) / 2 + offset) as i32;
        let y = ((output_height.saturating_sub(default_h)) / 2 + offset) as i32;

        Rect { x, y, width: default_w, height: default_h }
    }
}
