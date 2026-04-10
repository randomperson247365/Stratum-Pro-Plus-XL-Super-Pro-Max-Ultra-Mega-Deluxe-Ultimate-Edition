use super::TileGeometry;

/// Compute tile geometries using recursive binary-space partitioning.
///
/// Splits alternate vertical/horizontal by depth (even = vertical, odd = horizontal).
/// `split_ratio` controls the proportion given to the first (left/top) half; clamped
/// to [0.1, 0.9].  0.5 gives equal halves.
/// Returns an empty Vec when `count == 0`.
pub fn compute_bsp(
    count:       usize,
    ow:          i32,
    oh:          i32,
    gap_outer:   i32,
    gap_inner:   i32,
    split_ratio: f32,
) -> Vec<TileGeometry> {
    if count == 0 {
        return vec![];
    }
    let ratio = split_ratio.clamp(0.1, 0.9);
    let usable = TileGeometry {
        x:      gap_outer,
        y:      gap_outer,
        width:  (ow - 2 * gap_outer).max(1),
        height: (oh - 2 * gap_outer).max(1),
    };
    let mut out = vec![TileGeometry { x: 0, y: 0, width: 0, height: 0 }; count];
    bsp_split(&mut out, usable, 0, gap_inner, ratio);
    out
}

fn bsp_split(out: &mut [TileGeometry], rect: TileGeometry, depth: usize, gap: i32, ratio: f32) {
    if out.len() == 1 {
        out[0] = rect;
        return;
    }
    let left_count = (out.len() + 1) / 2;
    let (a, b) = if depth % 2 == 0 {
        // Vertical split: left | right
        let avail_w = (rect.width - gap).max(0);
        let lw = (avail_w as f32 * ratio) as i32;
        let rw = (avail_w - lw).max(1);
        (
            TileGeometry { x: rect.x,            y: rect.y, width: lw.max(1), height: rect.height },
            TileGeometry { x: rect.x + lw + gap, y: rect.y, width: rw,        height: rect.height },
        )
    } else {
        // Horizontal split: top / bottom
        let avail_h = (rect.height - gap).max(0);
        let th = (avail_h as f32 * ratio) as i32;
        let bh = (avail_h - th).max(1);
        (
            TileGeometry { x: rect.x, y: rect.y,            width: rect.width, height: th.max(1) },
            TileGeometry { x: rect.x, y: rect.y + th + gap, width: rect.width, height: bh },
        )
    };
    bsp_split(&mut out[..left_count], a, depth + 1, gap, ratio);
    bsp_split(&mut out[left_count..], b, depth + 1, gap, ratio);
}
