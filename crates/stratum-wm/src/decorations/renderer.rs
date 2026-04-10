//! CPU-side titlebar rendering using tiny-skia + fontdue.

use memmap2::MmapMut;
use stratum_config::{AppearanceConfig, DecorationsConfig};
use tiny_skia::{
    Color, FillRule, Paint, PathBuilder, Pixmap, PixmapPaint, Transform,
};

/// Renders titlebars into raw ARGB8888 pixel buffers.
pub struct TitlebarRenderer {
    font: fontdue::Font,
}

// Embed Inter Medium at compile time (Apache-2.0).
static INTER_MEDIUM: &[u8] =
    include_bytes!("../../../../data/Inter-Medium.ttf");

impl TitlebarRenderer {
    pub fn new() -> Self {
        let font = fontdue::Font::from_bytes(INTER_MEDIUM, fontdue::FontSettings::default())
            .expect("Inter-Medium.ttf is embedded and valid");
        Self { font }
    }

    /// Draw the titlebar for a window into `mmap`.
    ///
    /// `width` × `height` must match the buffer dimensions.
    /// Writes ARGB8888 pixels (premultiplied alpha).
    pub fn draw(
        &self,
        mmap: &mut MmapMut,
        width: i32,
        height: i32,
        title: &str,
        is_active: bool,
        deco: &DecorationsConfig,
        appear: &AppearanceConfig,
    ) {
        let w = width as u32;
        let h = height as u32;

        let mut pixmap = match Pixmap::new(w, h) {
            Some(p) => p,
            None => return,
        };

        let radius = deco.border_radius as f32;

        // ── Background ───────────────────────────────────────────────────────
        let bg_color = if is_active {
            parse_hex_color(&appear.accent_color)
        } else {
            Color::from_rgba8(0x3c, 0x3c, 0x3c, 0xff)
        };

        let mut bg_paint = Paint::default();
        bg_paint.set_color(bg_color);
        bg_paint.anti_alias = true;

        // Rounded top corners only: draw a full rounded rect then overdraw
        // the bottom corners with a square rect.
        let path = {
            let mut pb = PathBuilder::new();
            pb.move_to(0.0, h as f32);                           // bottom-left
            pb.line_to(0.0, radius);
            pb.quad_to(0.0, 0.0, radius, 0.0);                  // top-left curve
            pb.line_to(w as f32 - radius, 0.0);
            pb.quad_to(w as f32, 0.0, w as f32, radius);        // top-right curve
            pb.line_to(w as f32, h as f32);                      // bottom-right
            pb.close();
            match pb.finish() {
                Some(p) => p,
                None => return, // degenerate path — nothing to draw
            }
        };
        pixmap.fill_path(&path, &bg_paint, FillRule::Winding, Transform::identity(), None);

        // ── Buttons (right-aligned, 28px wide each) ──────────────────────────
        let btn_size = 28i32;
        let icon_size = 10.0f32;
        let btn_y_center = h as f32 / 2.0;

        let buttons: &[(&str, Color)] = &[
            ("close", Color::from_rgba8(0xe0, 0x5c, 0x5c, 0xff)),
            ("maximize", Color::from_rgba8(0xcc, 0xcc, 0xcc, 0xff)),
            ("minimize", Color::from_rgba8(0xcc, 0xcc, 0xcc, 0xff)),
        ];

        for (i, (kind, color)) in buttons.iter().enumerate() {
            let center_x = width as f32 - (i as f32 + 0.5) * btn_size as f32;

            let mut paint = Paint::default();
            paint.set_color(*color);
            paint.anti_alias = true;

            match *kind {
                "close" => {
                    // × symbol — two diagonal lines
                    draw_line(&mut pixmap, &paint,
                        center_x - icon_size / 2.0, btn_y_center - icon_size / 2.0,
                        center_x + icon_size / 2.0, btn_y_center + icon_size / 2.0, 1.5);
                    draw_line(&mut pixmap, &paint,
                        center_x + icon_size / 2.0, btn_y_center - icon_size / 2.0,
                        center_x - icon_size / 2.0, btn_y_center + icon_size / 2.0, 1.5);
                }
                "maximize" => {
                    // □ square outline
                    draw_rect_outline(&mut pixmap, &paint,
                        center_x - icon_size / 2.0, btn_y_center - icon_size / 2.0,
                        icon_size, icon_size, 1.5);
                }
                "minimize" => {
                    // − horizontal line
                    draw_line(&mut pixmap, &paint,
                        center_x - icon_size / 2.0, btn_y_center,
                        center_x + icon_size / 2.0, btn_y_center, 1.5);
                }
                _ => {}
            }
        }

        // ── Title text ────────────────────────────────────────────────────────
        let font_size = (h as f32 * 0.45).max(10.0).min(14.0);
        let text_color = if is_active {
            [0xff, 0xff, 0xff, 0xff]
        } else {
            [0xaa, 0xaa, 0xaa, 0xff]
        };
        let button_area = (buttons.len() as i32 * btn_size) + 8;
        let text_max_w = (width - button_area - 8).max(0) as u32;
        self.blit_text(&mut pixmap, title, font_size, 8.0, h as f32, text_color, text_max_w);

        // ── Copy pixmap → mmap as ARGB8888 ───────────────────────────────────
        let pixels = pixmap.data(); // RGBA8888 in tiny-skia (premultiplied)
        let buf = mmap.as_mut();
        let len = (w * h * 4) as usize;
        // Round down to multiple of 4 — each pixel is exactly 4 bytes.
        let copy_len = len.min(pixels.len()).min(buf.len()) & !3;
        // tiny-skia is RGBA; Wayland ARGB8888 is B, G, R, A in memory order.
        for i in (0..copy_len).step_by(4) {
            let r = pixels[i];
            let g = pixels[i + 1];
            let b = pixels[i + 2];
            let a = pixels[i + 3];
            buf[i]     = b; // B
            buf[i + 1] = g; // G
            buf[i + 2] = r; // R
            buf[i + 3] = a; // A
        }
    }

    /// Rasterize `text` with fontdue and blit glyphs into `pixmap`.
    fn blit_text(
        &self,
        pixmap: &mut Pixmap,
        text: &str,
        size: f32,
        x_start: f32,
        bar_height: f32,
        color: [u8; 4],
        max_width: u32,
    ) {
        let mut cursor_x = x_start;
        let pw = pixmap.width();
        let ph = pixmap.height();
        let pixels = pixmap.data_mut();

        for ch in text.chars() {
            if cursor_x as u32 >= max_width {
                break;
            }
            let (metrics, bitmap) = self.font.rasterize(ch, size);
            if metrics.width == 0 || metrics.height == 0 {
                cursor_x += metrics.advance_width;
                continue;
            }

            // Vertical centering
            let y_offset = ((bar_height - size) / 2.0 + (size - metrics.height as f32) / 2.0
                - metrics.ymin as f32) as i32;

            for row in 0..metrics.height {
                for col in 0..metrics.width {
                    let px = (cursor_x as i32 + col as i32 + metrics.xmin) as u32;
                    let py = (y_offset + row as i32) as u32;
                    if px >= pw || py >= ph {
                        continue;
                    }
                    let alpha = bitmap[row * metrics.width + col] as u32;
                    if alpha == 0 {
                        continue;
                    }
                    let idx = ((py * pw + px) * 4) as usize;
                    // Alpha-blend glyph over existing pixel (premultiplied)
                    let a = alpha * color[3] as u32 / 255;
                    let inv = 255 - a;
                    pixels[idx]     = ((color[0] as u32 * a + pixels[idx]     as u32 * inv) / 255) as u8;
                    pixels[idx + 1] = ((color[1] as u32 * a + pixels[idx + 1] as u32 * inv) / 255) as u8;
                    pixels[idx + 2] = ((color[2] as u32 * a + pixels[idx + 2] as u32 * inv) / 255) as u8;
                    pixels[idx + 3] = (a + pixels[idx + 3] as u32 * inv / 255) as u8;
                }
            }
            cursor_x += metrics.advance_width;
        }
    }
}

// ── Drawing helpers ───────────────────────────────────────────────────────────

fn draw_line(pixmap: &mut Pixmap, paint: &Paint, x0: f32, y0: f32, x1: f32, y1: f32, width: f32) {
    use tiny_skia::Stroke;
    let mut pb = PathBuilder::new();
    pb.move_to(x0, y0);
    pb.line_to(x1, y1);
    if let Some(path) = pb.finish() {
        let mut stroke = Stroke::default();
        stroke.width = width;
        pixmap.stroke_path(&path, paint, &stroke, Transform::identity(), None);
    }
}

fn draw_rect_outline(pixmap: &mut Pixmap, paint: &Paint, x: f32, y: f32, w: f32, h: f32, lw: f32) {
    use tiny_skia::Stroke;
    let mut pb = PathBuilder::new();
    pb.move_to(x, y);
    pb.line_to(x + w, y);
    pb.line_to(x + w, y + h);
    pb.line_to(x, y + h);
    pb.close();
    if let Some(path) = pb.finish() {
        let mut stroke = Stroke::default();
        stroke.width = lw;
        pixmap.stroke_path(&path, paint, &stroke, Transform::identity(), None);
    }
}

/// Parse a CSS hex color like `#5e81f4` into a tiny-skia Color.
pub fn parse_hex_color(s: &str) -> Color {
    let s = s.trim_start_matches('#');
    if s.len() < 6 {
        return Color::from_rgba8(0x5e, 0x81, 0xf4, 0xff);
    }
    let r = u8::from_str_radix(&s[0..2], 16).unwrap_or(0x5e);
    let g = u8::from_str_radix(&s[2..4], 16).unwrap_or(0x81);
    let b = u8::from_str_radix(&s[4..6], 16).unwrap_or(0xf4);
    Color::from_rgba8(r, g, b, 0xff)
}

/// Parse hex color to (r, g, b) as u32 for Wayland set_borders.
pub fn parse_hex_to_rgb(s: &str) -> (u32, u32, u32) {
    let s = s.trim_start_matches('#');
    let r = u8::from_str_radix(s.get(0..2).unwrap_or("5e"), 16).unwrap_or(0x5e) as u32;
    let g = u8::from_str_radix(s.get(2..4).unwrap_or("81"), 16).unwrap_or(0x81) as u32;
    let b = u8::from_str_radix(s.get(4..6).unwrap_or("f4"), 16).unwrap_or(0xf4) as u32;
    (r, g, b)
}
