use crate::protocol::{RiverNodeV1, RiverWindowV1};

/// State tracked for each open window received from River.
pub struct WindowState {
    pub proxy:       RiverWindowV1,
    pub node:        Option<RiverNodeV1>,
    pub app_id:      Option<String>,
    pub title:       Option<String>,  // null allowed per protocol
    // Current geometry as proposed by the WM.
    pub x:           i32,
    pub y:           i32,
    pub width:       i32,
    pub height:      i32,
    // Geometry as reported by River (set during render sequence).
    pub actual_width:  i32,
    pub actual_height: i32,
    pub floating:    bool,
    pub minimized:   bool,
    pub fullscreen:  bool,
}

impl WindowState {
    pub fn new(proxy: RiverWindowV1) -> Self {
        Self {
            proxy,
            node:          None,
            app_id:        None,
            title:         None,
            x:             0,
            y:             0,
            width:         800,
            height:        600,
            actual_width:  0,
            actual_height: 0,
            floating:      true,
            minimized:     false,
            fullscreen:    false,
        }
    }

    pub fn display_title(&self) -> &str {
        self.title.as_deref()
            .or(self.app_id.as_deref())
            .unwrap_or("(untitled)")
    }
}
