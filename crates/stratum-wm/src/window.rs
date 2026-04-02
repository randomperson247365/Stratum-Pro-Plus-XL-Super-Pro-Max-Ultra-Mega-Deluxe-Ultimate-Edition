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
    // ── Deferred manage-sequence informs ─────────────────────────────────────
    // Window management state requests (inform_fullscreen, inform_maximized,
    // inform_resize_start) may ONLY be sent inside a manage sequence.  When
    // the compositor fires the corresponding request events they arrive BEFORE
    // manage_start, so we record the intent here and apply it in
    // handle_manage_start.
    pub pending_fullscreen_inform: Option<bool>, // Some(true) → inform_fullscreen
    pub pending_maximized_inform:  Option<bool>, // Some(true) → inform_maximized
    pub pending_resize_start:      bool,         // true → inform_resize_start
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
            pending_fullscreen_inform: None,
            pending_maximized_inform:  None,
            pending_resize_start:      false,
        }
    }

    pub fn display_title(&self) -> &str {
        self.title.as_deref()
            .or(self.app_id.as_deref())
            .unwrap_or("(untitled)")
    }
}
