//! Per-profile window icon: Chromium-core LP browser icon with a profile-accent badge,
//! so a dozen windows are tellable apart in the Dock, taskbar and Alt-Tab.
//! Composed and cached here; the browser only loads the file.
#![allow(dead_code)]

use ab_glyph::{Font, FontRef, PxScale, ScaleFont};
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use tiny_skia::{Color, FillRule, Paint, PathBuilder, Pixmap, Transform};

const FONT: &[u8] = include_bytes!("../assets/NotoSans-Bold.ttf");
/// Chromium-core LP browser icon drawn as the full base of every profile's taskbar/window icon.
const MARK: &[u8] = include_bytes!("../icons/browser-icon.png");

/// Rendered at 512 and downscaled by the toolkit.
const SIZE: f32 = 512.0;

/// Per-platform geometry, matching what each shell draws around an icon.
struct Geometry {
    /// Inset from the canvas edge, as a fraction of SIZE.
    margin: f32,
    /// Corner radius as a fraction of SIZE (ignored for the squircle).
    radius: f32,
    squircle: bool,
    shadow: bool,
    outline: bool,
    /// Whether the body itself carries the profile's colour.
    ///
    /// A macOS icon does not follow the system theme — the same file is shown
    /// on a light Dock and on a dark one — so it has to read on both. A white
    /// card with a thin coloured band does not: against a dark Dock it is a
    /// pale blank with the one thing that tells profiles apart reduced to a
    /// stripe a few pixels tall. Every ordinary Mac icon is a filled shape,
    /// so this one is too, and the profile's colour is what fills it.
    coloured_body: bool,
}

const fn geometry() -> Geometry {
    #[cfg(target_os = "macos")]
    {
        Geometry { margin: 0.098, radius: 0.0, squircle: true, shadow: true, outline: false, coloured_body: true }
    }
    #[cfg(target_os = "windows")]
    {
        Geometry { margin: 0.045, radius: 0.115, squircle: false, shadow: false, outline: false, coloured_body: false }
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        Geometry { margin: 0.055, radius: 0.165, squircle: false, shadow: false, outline: true, coloured_body: false }
    }
}

/// Superellipse exponent; 5.0 is close to the macOS curve.
const N: f32 = 5.0;

/// Accent for a profile with no colour of its own: derived from the name, so
/// the same profile is always the same colour. It is the only thing still
/// telling profiles apart at 32 px, where no type is legible.
fn accent(name: &str) -> Color {
    let mut h: u64 = 1469598103934665603;
    for b in name.as_bytes() {
        h ^= *b as u64;
        h = h.wrapping_mul(1099511628211);
    }
    let hue = (h % 360) as f32;
    let (r, g, b) = hsv_to_rgb(hue, 0.70, 0.86);
    Color::from_rgba8(r, g, b, 255)
}

/// `#rgb` / `#rrggbb`, as the colour picker writes it.
fn parse_hex(raw: &str) -> Option<Color> {
    let h = raw.trim().trim_start_matches('#');
    let (r, g, b) = match h.len() {
        3 => {
            let d = |i: usize| u8::from_str_radix(&h[i..i + 1].repeat(2), 16).ok();
            (d(0)?, d(1)?, d(2)?)
        }
        6 => {
            let d = |i: usize| u8::from_str_radix(&h[i..i + 2], 16).ok();
            (d(0)?, d(2)?, d(4)?)
        }
        _ => return None,
    };
    Some(Color::from_rgba8(r, g, b, 255))
}

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (u8, u8, u8) {
    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;
    let (r, g, b) = match (h as u32) / 60 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    (
        ((r + m) * 255.0) as u8,
        ((g + m) * 255.0) as u8,
        ((b + m) * 255.0) as u8,
    )
}

fn squircle(cx: f32, cy: f32, a: f32) -> Option<tiny_skia::Path> {
    let mut pb = PathBuilder::new();
    let steps = 720;
    for i in 0..steps {
        let t = std::f32::consts::TAU * (i as f32) / (steps as f32);
        let (st, ct) = t.sin_cos();
        let x = cx + a * ct.abs().powf(2.0 / N) * ct.signum();
        let y = cy + a * st.abs().powf(2.0 / N) * st.signum();
        if i == 0 {
            pb.move_to(x, y);
        } else {
            pb.line_to(x, y);
        }
    }
    pb.close();
    pb.finish()
}

/// Rounded rectangle; tiny-skia has none. Quadratic corners are under a pixel
/// off at 512 px.
fn rounded_rect(l: f32, t: f32, r: f32, b: f32, rad: f32) -> Option<tiny_skia::Path> {
    let mut pb = PathBuilder::new();
    pb.move_to(l + rad, t);
    pb.line_to(r - rad, t);
    pb.quad_to(r, t, r, t + rad);
    pb.line_to(r, b - rad);
    pb.quad_to(r, b, r - rad, b);
    pb.line_to(l + rad, b);
    pb.quad_to(l, b, l, b - rad);
    pb.line_to(l, t + rad);
    pb.quad_to(l, t, l + rad, t);
    pb.close();
    pb.finish()
}

/// The body outline for this platform, plus its bounding box.
fn body_path(g: &Geometry) -> Option<(tiny_skia::Path, (f32, f32, f32, f32))> {
    let m = SIZE * g.margin;
    let (l, t, r, b) = (m, m, SIZE - m, SIZE - m);
    let path = if g.squircle {
        squircle(SIZE / 2.0, SIZE / 2.0, (r - l) / 2.0)?
    } else {
        rounded_rect(l, t, r, b, SIZE * g.radius)?
    };
    Some((path, (l, t, r, b)))
}

/// Wraps `name` to at most `max_lines` and picks the largest size that fits.
fn layout<'a>(
    font: &FontRef<'a>,
    name: &str,
    max_w: f32,
    max_h: f32,
    max_lines: usize,
) -> (f32, Vec<String>) {
    let words: Vec<&str> = name.split_whitespace().collect();
    let words = if words.is_empty() { vec![name] } else { words };

    let mut px = SIZE * 0.25;
    while px > 14.0 {
        let scaled = font.as_scaled(PxScale::from(px));
        let width = |s: &str| -> f32 {
            s.chars()
                .map(|c| scaled.h_advance(scaled.scaled_glyph(c).id))
                .sum()
        };
        let mut lines: Vec<String> = Vec::new();
        let mut cur = String::new();
        for w in &words {
            let cand = if cur.is_empty() {
                w.to_string()
            } else {
                format!("{cur} {w}")
            };
            if width(&cand) <= max_w {
                cur = cand;
            } else {
                if !cur.is_empty() {
                    lines.push(std::mem::take(&mut cur));
                }
                cur = w.to_string();
                // A word wider than the line is ellipsised, not overflowed.
                while !cur.is_empty() && width(&format!("{cur}…")) > max_w {
                    cur.pop();
                }
                if width(w) > max_w {
                    cur.push('…');
                }
            }
        }
        if !cur.is_empty() {
            lines.push(cur);
        }
        if lines.len() <= max_lines && (lines.len() as f32) * px * 1.16 <= max_h {
            return (px, lines);
        }
        px -= 4.0;
    }
    (14.0, vec![name.chars().take(9).collect()])
}

fn draw_text(
    pix: &mut Pixmap,
    font: &FontRef<'_>,
    lines: &[String],
    px: f32,
    cx: f32,
    top: f32,
    color: (u8, u8, u8),
) {
    let scaled = font.as_scaled(PxScale::from(px));
    let mut y = top + scaled.ascent();
    for line in lines {
        let w: f32 = line
            .chars()
            .map(|c| scaled.h_advance(scaled.scaled_glyph(c).id))
            .sum();
        let mut x = cx - w / 2.0;
        for ch in line.chars() {
            let glyph = scaled.scaled_glyph(ch);
            let advance = scaled.h_advance(glyph.id);
            if let Some(outline) = font.outline_glyph(glyph.clone()) {
                let bounds = outline.px_bounds();
                outline.draw(|gx, gy, cov| {
                    if cov <= 0.0 {
                        return;
                    }
                    let px_x = (bounds.min.x + gx as f32 + x) as i32;
                    let px_y = (bounds.min.y + gy as f32 + y) as i32;
                    if px_x < 0 || px_y < 0 {
                        return;
                    }
                    let (w, h) = (pix.width() as i32, pix.height() as i32);
                    if px_x >= w || px_y >= h {
                        return;
                    }
                    // tiny_skia has no glyph rasteriser: ab_glyph gives coverage,
                    // we composite it.
                    let idx = (px_y * w + px_x) as usize * 4;
                    let data = pix.data_mut();
                    let a = cov.clamp(0.0, 1.0);
                    for (i, c) in [color.0, color.1, color.2].iter().enumerate() {
                        let dst = data[idx + i] as f32;
                        data[idx + i] = (dst * (1.0 - a) + (*c as f32) * a) as u8;
                    }
                    let dst_a = data[idx + 3] as f32;
                    data[idx + 3] = (dst_a * (1.0 - a) + 255.0 * a) as u8;
                });
            }
            x += advance;
        }
        y += px * 1.16;
    }
}

/// Compute deterministic cache key hash for `(taskbar_title, profile_name, color)`.
fn icon_hash(taskbar_title: &str, profile_name: &str, color: Option<&str>) -> u64 {
    let badge = taskbar_title.trim();
    let key = format!(
        "chromium-v3\u{1}{badge}\u{1}{}\u{1}{}\u{1}{}\u{1}{}",
        if badge.is_empty() { "" } else { profile_name },
        color.unwrap_or(""),
        env!("CARGO_PKG_VERSION"),
        MARK.len()
    );
    let mut h: u64 = 1469598103934665603;
    for b in key.as_bytes() {
        h ^= *b as u64;
        h = h.wrapping_mul(1099511628211);
    }
    h
}

/// Render the 512x512 RGBA premultiplied pixmap for a profile's browser icon.
/// When `taskbar_title` is non-empty, draws a bold high-contrast pill badge at the bottom
/// so the label is immediately readable on the Windows taskbar, Dock, and Desktop shortcuts.
/// When `taskbar_title` is empty, renders the clean Chromium-LP browser icon without a text pill.
fn render_icon_pixmap(
    taskbar_title: &str,
    profile_name: &str,
    color: Option<&str>,
) -> Result<Pixmap> {
    let s = SIZE as u32;
    let mut pix = Pixmap::new(s, s).context("allocate pixmap")?;
    let cx = SIZE / 2.0;
    let badge = taskbar_title.trim();

    if badge.is_empty() {
        draw_mark(&mut pix, cx, cx, SIZE * 0.92);
        return Ok(pix);
    }

    // 1. Draw the Chromium-core LP browser icon filling the taskbar canvas,
    //    slightly lifted upward so the bottom Taskbar Title pill sits cleanly.
    draw_mark(&mut pix, cx, SIZE * 0.43, SIZE * 0.84);

    // 2. Draw a crisp profile-accent pill badge with the Taskbar Title along the bottom edge.
    let accent_seed = profile_name.strip_prefix("LPB|").unwrap_or(profile_name).trim();
    let accent_seed = if accent_seed.is_empty() { badge } else { accent_seed };
    let tint = color.and_then(parse_hex).unwrap_or_else(|| accent(accent_seed));

    let badge_h = SIZE * 0.33;
    let badge_w = SIZE * 0.94;
    let bx0 = cx - badge_w / 2.0;
    let by0 = SIZE - badge_h - SIZE * 0.02;
    let bx1 = bx0 + badge_w;
    let by1 = by0 + badge_h;
    let badge_r = badge_h * 0.44;

    if let Some(pill) = rounded_rect(bx0, by0, bx1, by1, badge_r) {
        let mut paint = Paint::default();
        paint.anti_alias = true;

        // Crisp white outer halo stroke so the pill separates cleanly from the Chromium wheel.
        paint.set_color(Color::from_rgba8(255, 255, 255, 248));
        let stroke = tiny_skia::Stroke { width: SIZE * 0.034, ..Default::default() };
        pix.stroke_path(&pill, &paint, &stroke, Transform::identity(), None);

        // Filled pill in the profile's accent colour.
        paint.set_color(tint);
        pix.fill_path(&pill, &paint, FillRule::Winding, Transform::identity(), None);
    }

    let font = FontRef::try_from_slice(FONT).context("load font")?;
    let (px, lines) = layout(&font, badge, badge_w * 0.86, badge_h * 0.80, 1);
    let block_h = lines.len() as f32 * px * 1.05;
    let text_top = by0 + (badge_h - block_h) / 2.0 - px * 0.04;
    let ink = if luminance(tint) > 0.62 { (20, 24, 32) } else { (255, 255, 255) };
    draw_text(&mut pix, &font, &lines, px, cx, text_top, ink);

    Ok(pix)
}

/// Progressive box/bicubic downscale of a square `Pixmap` to `target_size` for crisp small icons.
fn downscale_pixmap(src: &Pixmap, target_size: u32) -> Result<Pixmap> {
    let mut cur: Option<Pixmap> = None;
    let mut cur_ref: &Pixmap = src;
    let paint = tiny_skia::PixmapPaint {
        quality: tiny_skia::FilterQuality::Bicubic,
        ..Default::default()
    };
    while cur_ref.width() / 2 >= target_size && cur_ref.width() > target_size {
        let next_sz = cur_ref.width() / 2;
        let mut next = Pixmap::new(next_sz, next_sz).context("allocate downscale step")?;
        let scale = next_sz as f32 / cur_ref.width() as f32;
        next.draw_pixmap(0, 0, cur_ref.as_ref(), &paint, Transform::from_scale(scale, scale), None);
        cur = Some(next);
        cur_ref = cur.as_ref().unwrap();
    }
    if cur_ref.width() == target_size {
        return Ok(cur.unwrap_or_else(|| src.clone()));
    }
    let mut out = Pixmap::new(target_size, target_size).context("allocate target icon pixmap")?;
    let scale = target_size as f32 / cur_ref.width() as f32;
    out.draw_pixmap(0, 0, cur_ref.as_ref(), &paint, Transform::from_scale(scale, scale), None);
    Ok(out)
}

/// Encode a `Pixmap` (< 256 px) as a 32-bit BGRA `BITMAPINFOHEADER` DIB + 1bpp AND mask for `.ico`.
fn encode_ico_dib(pix: &Pixmap) -> Vec<u8> {
    let w = pix.width();
    let h = pix.height();
    let xor_bytes = (w * h * 4) as usize;
    let and_row_bytes = (((w + 31) / 32) * 4) as usize;
    let and_bytes = and_row_bytes * (h as usize);
    let mut out = Vec::with_capacity(40 + xor_bytes + and_bytes);

    // BITMAPINFOHEADER (40 bytes)
    out.extend_from_slice(&40u32.to_le_bytes());
    out.extend_from_slice(&(w as i32).to_le_bytes());
    out.extend_from_slice(&((h * 2) as i32).to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes()); // planes
    out.extend_from_slice(&32u16.to_le_bytes()); // bpp
    out.extend_from_slice(&0u32.to_le_bytes()); // BI_RGB
    out.extend_from_slice(&((xor_bytes + and_bytes) as u32).to_le_bytes());
    out.extend_from_slice(&[0u8; 16]);

    let src = pix.data();
    // Bottom-up BGRA XOR mask (unpremultiplied from tiny_skia's premultiplied RGBA)
    for y in (0..h).rev() {
        for x in 0..w {
            let idx = ((y * w + x) * 4) as usize;
            let (pr, pg, pb, a) = (src[idx], src[idx + 1], src[idx + 2], src[idx + 3]);
            let (r, g, b) = if a == 0 {
                (0, 0, 0)
            } else if a == 255 {
                (pr, pg, pb)
            } else {
                let un = |c: u8| (((c as u32) * 255 + (a as u32 / 2)) / (a as u32)).min(255) as u8;
                (un(pr), un(pg), un(pb))
            };
            out.extend_from_slice(&[b, g, r, a]);
        }
    }

    // Bottom-up 1bpp AND mask (1 = fully transparent pixel)
    for y in (0..h).rev() {
        let mut row = vec![0u8; and_row_bytes];
        for x in 0..w {
            let a = src[((y * w + x) * 4 + 3) as usize];
            if a == 0 {
                row[(x / 8) as usize] |= 1 << (7 - (x % 8));
            }
        }
        out.extend_from_slice(&row);
    }

    out
}

/// Composes the 512x512 PNG icon for `(taskbar_title, profile_name, color)` and returns its path.
pub fn ensure_icon(
    cache_dir: &Path,
    taskbar_title: &str,
    profile_name: &str,
    color: Option<&str>,
) -> Result<PathBuf> {
    let h = icon_hash(taskbar_title, profile_name, color);
    let dir = cache_dir.join("profile-icons");
    std::fs::create_dir_all(&dir).ok();
    let out = dir.join(format!("{h:016x}.png"));
    if out.exists() {
        return Ok(out);
    }
    let pix = render_icon_pixmap(taskbar_title, profile_name, color)?;
    pix.save_png(&out).context("write icon png")?;
    Ok(out)
}

/// Composes a multi-resolution Windows `.ico` (`256, 64, 48, 32, 16`) for Desktop shortcuts
/// and returns its cached `.ico` path.
pub fn ensure_ico(
    cache_dir: &Path,
    taskbar_title: &str,
    profile_name: &str,
    color: Option<&str>,
) -> Result<PathBuf> {
    let h = icon_hash(taskbar_title, profile_name, color);
    let dir = cache_dir.join("profile-icons");
    std::fs::create_dir_all(&dir).context("create profile-icons dir")?;
    let out = dir.join(format!("{h:016x}.ico"));
    if out.exists() {
        return Ok(out);
    }

    let master = render_icon_pixmap(taskbar_title, profile_name, color)?;
    let sizes: [u32; 5] = [256, 64, 48, 32, 16];
    let mut payloads: Vec<(u32, Vec<u8>)> = Vec::with_capacity(sizes.len());
    for sz in sizes {
        let scaled = downscale_pixmap(&master, sz)?;
        let bytes = if sz >= 256 {
            scaled.encode_png().context("encode 256px ico png")?
        } else {
            encode_ico_dib(&scaled)
        };
        payloads.push((sz, bytes));
    }

    let count = payloads.len();
    let mut ico = Vec::new();
    // ICONDIR header (6 bytes)
    ico.extend_from_slice(&0u16.to_le_bytes());
    ico.extend_from_slice(&1u16.to_le_bytes());
    ico.extend_from_slice(&(count as u16).to_le_bytes());

    let mut offset = (6 + 16 * count) as u32;
    for (sz, bytes) in &payloads {
        let dim = if *sz >= 256 { 0u8 } else { *sz as u8 };
        ico.push(dim);
        ico.push(dim);
        ico.push(0); // palette
        ico.push(0); // reserved
        ico.extend_from_slice(&1u16.to_le_bytes()); // planes
        ico.extend_from_slice(&32u16.to_le_bytes()); // bpp
        ico.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
        ico.extend_from_slice(&offset.to_le_bytes());
        offset += bytes.len() as u32;
    }
    for (_, bytes) in payloads {
        ico.extend_from_slice(&bytes);
    }

    std::fs::write(&out, ico).context("write icon ico")?;
    Ok(out)
}

/// The same colour, darker, for the band under the mark.
fn shade(c: Color, factor: f32) -> Color {
    Color::from_rgba8(
        (c.red() * 255.0 * factor) as u8,
        (c.green() * 255.0 * factor) as u8,
        (c.blue() * 255.0 * factor) as u8,
        255,
    )
}

/// Perceived brightness, for deciding whether type on this colour is dark or
/// light. A pale colour an operator picked must not end up white on white.
fn luminance(c: Color) -> f32 {
    0.2126 * c.red() + 0.7152 * c.green() + 0.0722 * c.blue()
}

/// The product mark, centred on (`cx`, `cy`) at `height` pixels.
fn draw_mark(pix: &mut Pixmap, cx: f32, cy: f32, height: f32) {
    let Some(mark) = mark_pixmap() else { return };
    if height <= 1.0 {
        return;
    }
    let scale = height / mark.height() as f32;
    let w = mark.width() as f32 * scale;
    let paint = tiny_skia::PixmapPaint {
        quality: tiny_skia::FilterQuality::Bicubic,
        ..Default::default()
    };
    // draw_pixmap places the pixmap's origin at (x, y) and then applies the
    // transform, so the scale has to be folded into the offset by hand.
    pix.draw_pixmap(
        0,
        0,
        mark.as_ref(),
        &paint,
        Transform::from_scale(scale, scale)
            .post_translate(cx - w / 2.0, cy - height / 2.0),
        None,
    );
}

/// Pre-decode the embedded Chromium-LP base icon in the background so the first
/// profile icon generation does not wait on PNG decoding.
pub fn warm_up() {
    let _ = mark_pixmap();
}

/// The mark, decoded once and premultiplied — tiny-skia wants premultiplied
/// alpha, PNG stores straight.
fn mark_pixmap() -> Option<&'static Pixmap> {
    static CACHE: std::sync::OnceLock<Option<Pixmap>> = std::sync::OnceLock::new();
    CACHE
        .get_or_init(|| {
            let mut reader = png::Decoder::new(MARK).read_info().ok()?;
            let mut buf = vec![0; reader.output_buffer_size()];
            let info = reader.next_frame(&mut buf).ok()?;
            if info.color_type != png::ColorType::Rgba || info.bit_depth != png::BitDepth::Eight {
                return None;
            }
            buf.truncate(info.buffer_size());
            for px in buf.chunks_exact_mut(4) {
                let a = px[3] as u32;
                for c in 0..3 {
                    px[c] = ((px[c] as u32 * a + 127) / 255) as u8;
                }
            }
            Pixmap::from_vec(buf, tiny_skia::IntSize::from_wh(info.width, info.height)?)
        })
        .as_ref()
}
