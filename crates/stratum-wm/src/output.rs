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
    /// Physical width in millimetres from wl_output::Event::Geometry.
    /// None until the event arrives; zero means virtual/projector (treat as unknown).
    pub physical_width_mm:  Option<i32>,
    /// Physical height in millimetres from wl_output::Event::Geometry.
    pub physical_height_mm: Option<i32>,
}

impl OutputState {
    pub fn new(proxy: RiverOutputV1) -> Self {
        Self {
            proxy,
            x: 0,
            y: 0,
            width:  1920,
            height: 1080,
            wl_output_name:    None,
            physical_width_mm:  None,
            physical_height_mm: None,
        }
    }

    /// Diagonal DPI computed from physical mm dimensions reported by the display.
    /// Returns `None` for virtual displays (physical mm == 0) or when the geometry
    /// event has not arrived yet.
    pub fn dpi(&self) -> Option<f32> {
        let pw = self.physical_width_mm.filter(|&v| v > 0)?;
        let ph = self.physical_height_mm.filter(|&v| v > 0)?;
        let px_diag = ((self.width  as f32).powi(2)
                     + (self.height as f32).powi(2)).sqrt();
        let mm_diag = ((pw as f32).powi(2) + (ph as f32).powi(2)).sqrt();
        Some(px_diag / (mm_diag / 25.4))
    }
}
