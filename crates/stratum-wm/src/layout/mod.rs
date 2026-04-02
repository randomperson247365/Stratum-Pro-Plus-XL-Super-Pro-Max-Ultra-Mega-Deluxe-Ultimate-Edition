pub mod floating;
pub mod tiling;

pub use floating::FloatingLayout;
pub use tiling::{compute as compute_tiles, TileGeometry};

/// A rectangle describing a window's proposed position and size.
#[derive(Debug, Clone, Copy)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width:  u32,
    pub height: u32,
}

/// Trait for layout engines. Phase 5 adds a TilingLayout implementation.
pub trait Layout {
    /// Called when a new window is added. Returns the initial proposed geometry.
    fn window_added(&mut self, output_width: u32, output_height: u32) -> Rect;
}
