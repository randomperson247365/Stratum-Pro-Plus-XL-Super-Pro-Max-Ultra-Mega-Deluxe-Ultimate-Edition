use super::TileGeometry;

/// Compute tile geometries using recursive binary-space partitioning.
///
/// Splits alternate vertical/horizontal by depth (even = vertical, odd = horizontal).
/// Returns an empty Vec when `count == 0`.
pub fn compute_bsp(
    count:     usize,
    ow:        i32,
    oh:        i32,
    gap_outer: i32,
    gap_inner: i32,
) -> Vec<TileGeometry> {
    if count == 0 {
        return vec![];
    }
    let usable = TileGeometry {
        x:      gap_outer,
        y:      gap_outer,
        width:  (ow - 2 * gap_outer).max(1),
        height: (oh - 2 * gap_outer).max(1),
    };
    let mut out = vec![TileGeometry { x: 0, y: 0, width: 0, height: 0 }; count];
    bsp_split(&mut out, usable, 0, gap_inner);
    out
}

fn bsp_split(out: &mut [TileGeometry], rect: TileGeometry, depth: usize, gap: i32) {
    if out.len() == 1 {
        out[0] = rect;
        return;
    }
    let left_count = (out.len() + 1) / 2;
    let (a, b) = if depth % 2 == 0 {
        // Vertical split: left | right
        let lw = (rect.width - gap) / 2;
        let rw = rect.width - lw - gap;
        (
            TileGeometry { x: rect.x,            y: rect.y, width: lw, height: rect.height },
            TileGeometry { x: rect.x + lw + gap, y: rect.y, width: rw, height: rect.height },
        )
    } else {
        // Horizontal split: top / bottom
        let th = (rect.height - gap) / 2;
        let bh = rect.height - th - gap;
        (
            TileGeometry { x: rect.x, y: rect.y,            width: rect.width, height: th },
            TileGeometry { x: rect.x, y: rect.y + th + gap, width: rect.width, height: bh },
        )
    };
    bsp_split(&mut out[..left_count], a, depth + 1, gap);
    bsp_split(&mut out[left_count..], b, depth + 1, gap);
}
