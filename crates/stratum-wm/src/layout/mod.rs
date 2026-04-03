pub mod bsp;
pub mod floating;
pub mod tiling;

pub use bsp::compute_bsp;
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

/// Active layout mode for an output.
///
/// `super+t` cycles: Floating → MasterStack → Bsp → Floating.
/// There is no standalone Deck mode; both tiled modes fall back to a deck
/// arrangement automatically when any tile shrinks below the configured
/// minimum size (DPI-scaled from the display's physical mm dimensions).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LayoutMode {
    #[default]
    Floating,
    MasterStack,
    Bsp,
}

impl LayoutMode {
    pub fn cycle(self) -> Self {
        match self {
            Self::Floating    => Self::MasterStack,
            Self::MasterStack => Self::Bsp,
            Self::Bsp         => Self::Floating,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Floating    => "floating",
            Self::MasterStack => "master_stack",
            Self::Bsp         => "bsp",
        }
    }
}
