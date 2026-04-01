//! Server-side decoration lifecycle management.
//!
//! Each window gets a `WindowDecoration` that owns:
//!   • a `wl_surface` used as the titlebar canvas
//!   • a `river_decoration_v1` that tells River to composite it above the window
//!   • a Wayland SHM buffer (pool + buffer + mmap) for CPU-rendered pixels
//!
//! Decorations must be updated inside the render sequence
//! (between `RenderStart` and `render_finish`).

pub mod renderer;
pub mod shm;

use memmap2::MmapMut;
use wayland_client::{
    protocol::{wl_buffer, wl_compositor, wl_shm, wl_shm_pool, wl_surface},
    QueueHandle,
};

use stratum_config::StratumConfig;

use crate::{
    protocol::{RiverDecorationV1, RiverWindowV1},
    state::AppState,
};
pub use renderer::{parse_hex_to_rgb, TitlebarRenderer};

// ── WindowDecoration ──────────────────────────────────────────────────────────

pub struct WindowDecoration {
    pub titlebar_surface: wl_surface::WlSurface,
    pub titlebar_deco:    RiverDecorationV1,
    pub shm_pool:         wl_shm_pool::WlShmPool,
    pub buffer:           wl_buffer::WlBuffer,
    pub mmap:             MmapMut,
    /// Current buffer width (= window content width).
    pub width:            i32,
    /// Buffer height (= config.decorations.titlebar_height).
    pub height:           i32,
}

// ── Lifecycle ─────────────────────────────────────────────────────────────────

/// Create a decoration for the given window proxy.
///
/// The initial buffer is sized to a sensible default; it will be resized on
/// the first `update` call once we know the actual window width.
pub fn create(
    win_proxy: &RiverWindowV1,
    compositor: &wl_compositor::WlCompositor,
    wl_shm: &wl_shm::WlShm,
    qh: &QueueHandle<AppState>,
    config: &StratumConfig,
) -> anyhow::Result<WindowDecoration> {
    let height = config.decorations.titlebar_height as i32;
    let width = 800i32; // placeholder; resized in update()

    let surface = compositor.create_surface(qh, ());
    let (pool, buffer, mmap) = shm::alloc(wl_shm, qh, width, height)?;
    let deco = win_proxy.get_decoration_above(&surface, qh, ());

    Ok(WindowDecoration {
        titlebar_surface: surface,
        titlebar_deco: deco,
        shm_pool: pool,
        buffer,
        mmap,
        width,
        height,
    })
}

/// Redraw the titlebar into the SHM buffer, resizing if the window changed width.
///
/// Call this inside the render sequence, before `commit_in_render_sequence`.
pub fn update(
    deco: &mut WindowDecoration,
    wl_shm: &wl_shm::WlShm,
    qh: &QueueHandle<AppState>,
    new_width: i32,
    is_active: bool,
    title: &str,
    config: &StratumConfig,
    renderer: &TitlebarRenderer,
) {
    // Re-allocate if the window became wider (or on first render with real width).
    if new_width != deco.width {
        // Destroy old buffer/pool, allocate new ones.
        deco.buffer.destroy();
        deco.shm_pool.destroy();
        match shm::alloc(wl_shm, qh, new_width, deco.height) {
            Ok((pool, buf, mm)) => {
                deco.shm_pool = pool;
                deco.buffer   = buf;
                deco.mmap     = mm;
                deco.width    = new_width;
            }
            Err(e) => {
                eprintln!("stratum-wm: decoration resize failed: {e}");
                return;
            }
        }
    }

    renderer.draw(
        &mut deco.mmap,
        deco.width,
        deco.height,
        title,
        is_active,
        &config.decorations,
        &config.appearance,
    );
}

/// Commit the decoration surface in sync with the current render sequence.
///
/// **Must** be called after `update` and before `render_finish`.
///
/// Protocol contract:
///   1. `sync_next_commit()` — declare intent to sync
///   2. attach buffer + damage + commit on the `wl_surface`
///   3. `set_offset` to position the decoration relative to window top-left
pub fn commit_in_render_sequence(deco: &WindowDecoration) {
    // Step 1: sync with render_finish
    deco.titlebar_deco.sync_next_commit();

    // Step 2: attach, damage, commit
    deco.titlebar_surface.attach(Some(&deco.buffer), 0, 0);
    deco.titlebar_surface.damage_buffer(0, 0, deco.width, deco.height);
    deco.titlebar_surface.commit();

    // Step 3: position — directly above the window content
    deco.titlebar_deco.set_offset(0, -deco.height);
}

/// Destroy all Wayland objects owned by the decoration.
pub fn destroy(deco: WindowDecoration) {
    deco.buffer.destroy();
    deco.shm_pool.destroy();
    deco.titlebar_deco.destroy();
    deco.titlebar_surface.destroy();
}
