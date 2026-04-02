/// The proposed position and size for a single tiled window.
#[derive(Debug, Clone, Copy)]
pub struct TileGeometry {
    pub x:      i32,
    pub y:      i32,
    pub width:  i32,
    pub height: i32,
}

/// Compute tile geometries for `count` visible windows.
///
/// Windows are ordered master-first (index 0 = master, 1..N-1 = stack).
/// The master occupies the left half; the stack windows divide the right half evenly.
///
/// For a single window the full usable area is returned.
/// Returns an empty Vec when `count == 0`.
pub fn compute(
    count:      usize,
    ow:         i32,
    oh:         i32,
    gap_outer:  i32,
    gap_inner:  i32,
) -> Vec<TileGeometry> {
    if count == 0 {
        return Vec::new();
    }

    let usable_x = gap_outer;
    let usable_y = gap_outer;
    let usable_w = (ow - 2 * gap_outer).max(1);
    let usable_h = (oh - 2 * gap_outer).max(1);

    if count == 1 {
        return vec![TileGeometry {
            x: usable_x,
            y: usable_y,
            width: usable_w,
            height: usable_h,
        }];
    }

    // Split horizontally: master left, stack right.
    let half_gap  = gap_inner / 2;
    let master_w  = (usable_w / 2) - half_gap;
    let stack_x   = usable_x + master_w + gap_inner;
    let stack_w   = usable_w - master_w - gap_inner;
    let stack_count = count - 1;

    let total_gaps  = gap_inner * (stack_count as i32 - 1);
    let stack_h     = ((usable_h - total_gaps) / stack_count as i32).max(1);

    let mut tiles = Vec::with_capacity(count);

    // Master
    tiles.push(TileGeometry {
        x: usable_x,
        y: usable_y,
        width: master_w,
        height: usable_h,
    });

    // Stack
    for i in 0..stack_count {
        let tile_y = usable_y + i as i32 * (stack_h + gap_inner);
        tiles.push(TileGeometry {
            x: stack_x,
            y: tile_y,
            width: stack_w,
            height: stack_h,
        });
    }

    tiles
}
