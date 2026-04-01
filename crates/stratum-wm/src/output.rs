use crate::protocol::RiverOutputV1;

/// State tracked for each connected output (monitor).
pub struct OutputState {
    pub proxy:   RiverOutputV1,
    pub x:       i32,
    pub y:       i32,
    pub width:   u32,
    pub height:  u32,
    /// Global name of the correlated wl_output (set by RiverOutputV1::wl_output event).
    pub wl_output_name: Option<u32>,
}

impl OutputState {
    pub fn new(proxy: RiverOutputV1) -> Self {
        Self {
            proxy,
            x: 0,
            y: 0,
            width:  1920,
            height: 1080,
            wl_output_name: None,
        }
    }
}
