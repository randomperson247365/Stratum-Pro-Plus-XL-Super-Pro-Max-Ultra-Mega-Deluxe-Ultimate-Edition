//! Wayland SHM buffer allocation for decoration surfaces.

use std::{fs::File, os::unix::io::AsFd};

use memmap2::MmapMut;
use wayland_client::{
    protocol::{wl_buffer, wl_shm, wl_shm_pool},
    QueueHandle,
};

use crate::state::AppState;

/// Allocate a Wayland SHM buffer backed by an anonymous file.
///
/// Returns `(pool, buffer, mmap)`.  The pool must stay alive while any buffer
/// from it is in use by a surface.
pub fn alloc(
    wl_shm: &wl_shm::WlShm,
    qh: &QueueHandle<AppState>,
    width: i32,
    height: i32,
) -> anyhow::Result<(wl_shm_pool::WlShmPool, wl_buffer::WlBuffer, MmapMut)> {
    let stride = width * 4;
    let size   = stride * height;

    let file = create_anon_file(size as u64)?;

    // Map memory — safe because file is valid, size > 0.
    let mmap = unsafe { MmapMut::map_mut(&file)? };

    // Create pool — compositor receives an fd via SCM_RIGHTS (kernel-duplicated).
    let pool   = wl_shm.create_pool(file.as_fd(), size, qh, ());
    let buffer = pool.create_buffer(0, width, height, stride, wl_shm::Format::Argb8888, qh, ());

    // file drops here — mmap and compositor share are independently valid.
    Ok((pool, buffer, mmap))
}

/// Create an anonymous, unlinked file of the given byte size.
fn create_anon_file(size: u64) -> anyhow::Result<File> {
    use std::fs::OpenOptions;

    // Try /dev/shm (in-memory FS, ideal for SHM buffers).
    let path = format!("/dev/shm/stratum-deco-{}", std::process::id());
    let result = OpenOptions::new().read(true).write(true).create(true).open(&path);

    let (file, path_to_unlink) = match result {
        Ok(f) => (f, path),
        Err(_) => {
            let tmp = format!("/tmp/stratum-deco-{}", std::process::id());
            (OpenOptions::new().read(true).write(true).create(true).open(&tmp)?, tmp)
        }
    };

    // Unlink immediately — fd stays open, no visible filesystem entry.
    let _ = std::fs::remove_file(&path_to_unlink);

    // Pre-fill with transparent pixels (ARGB 0x00000000).
    file.set_len(size)?;

    Ok(file)
}
