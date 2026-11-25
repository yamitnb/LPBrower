"""Extract, refine, and export the complete LPBrowser brand icon suite from the showcase artwork."""

from __future__ import annotations

import io
import struct
from collections import deque
from pathlib import Path

import numpy as np
from PIL import Image, ImageDraw, ImageFilter

ROOT = Path(__file__).resolve().parent.parent
ICONS_DIR = ROOT / "src-tauri" / "icons"
NTP_ASSETS_DIR = ROOT / "src-tauri" / "assets" / "ntp"
PUBLIC_DIR = ROOT / "public"
BRAND_DIR = PUBLIC_DIR / "brand-icons"

SRC_SHOWCASE = Path(
    r"C:/Users/Admin/.gemini/antigravity/brain/6659144a-f4cf-41d2-a473-a1d186c1fdb7"
    r"/.user_uploaded/media_1795577525736_223f93ea.jpg"
)
SCRATCH_DIR = Path(
    r"C:/Users/Admin/.gemini/antigravity/brain/6659144a-f4cf-41d2-a473-a1d186c1fdb7/scratch"
)

BG_RGB = np.array([17.0, 25.0, 46.0], dtype=np.float32)


def decontaminate_rgba(rgb: np.ndarray, alpha: np.ndarray, bg: np.ndarray = BG_RGB) -> np.ndarray:
    """Unmix background color from semi-transparent edge pixels and pack straight RGBA uint8."""
    a = np.clip(alpha, 0.0, 1.0)[..., None]
    safe_a = np.maximum(a, 0.08)
    clean_rgb = np.where(a > 0.01, (rgb - bg * (1.0 - a)) / safe_a, 0.0)
    # Blend only on semi-transparent border pixels so opaque interior stays 100% original
    interior = (a >= 0.94)
    final_rgb = np.where(interior, rgb, clean_rgb)
    final_rgb = np.clip(np.round(final_rgb), 0, 255).astype(np.uint8)
    final_a = np.clip(np.round(alpha * 255.0), 0, 255).astype(np.uint8)[..., None]
    return np.concatenate([final_rgb, final_a], axis=2)


def smooth_contour_mask(fg_bool: np.ndarray, erode_px: float = 0.75) -> np.ndarray:
    """Upscale binary mask 4x, smooth JPEG stair-stepping, erode slightly, and return [0,1] alpha."""
    h, w = fg_bool.shape
    up = Image.fromarray((fg_bool.astype(np.uint8) * 255), mode="L").resize(
        (w * 4, h * 4), Image.Resampling.BILINEAR
    )
    up = up.filter(ImageFilter.GaussianBlur(radius=2.6))
    arr = np.array(up, dtype=np.float32) / 255.0
    # Shift threshold inward slightly (~erode_px in 1x space) with smooth anti-aliased ramp
    thresh = 0.50 + (erode_px * 0.14)
    ramp = np.clip((arr - (thresh - 0.14)) / 0.28, 0.0, 1.0)
    down = Image.fromarray(np.round(ramp * 255.0).astype(np.uint8), mode="L").resize(
        (w, h), Image.Resampling.LANCZOS
    )
    return np.array(down, dtype=np.float32) / 255.0


def flood_exterior_bg(crop_rgb: np.ndarray, dist_thresh: float = 20.0) -> np.ndarray:
    """Flood-fill background from all 4 edges; returns foreground boolean mask."""
    h, w, _ = crop_rgb.shape
    dist = np.linalg.norm(crop_rgb - BG_RGB, axis=2)
    is_bg_cand = dist < dist_thresh
    visited = np.zeros((h, w), dtype=bool)
    q: deque[tuple[int, int]] = deque()

    for x in range(w):
        for y in (0, h - 1):
            if is_bg_cand[y, x] and not visited[y, x]:
                visited[y, x] = True
                q.append((y, x))
    for y in range(h):
        for x in (0, w - 1):
            if is_bg_cand[y, x] and not visited[y, x]:
                visited[y, x] = True
                q.append((y, x))

    while q:
        cy, cx = q.popleft()
        for ny, nx in ((cy - 1, cx), (cy + 1, cx), (cy, cx - 1), (cy, cx + 1)):
            if 0 <= ny < h and 0 <= nx < w and not visited[ny, nx] and is_bg_cand[ny, nx]:
                visited[ny, nx] = True
                q.append((ny, nx))

    return ~visited


def extract_main_app(full_rgb: np.ndarray) -> Image.Image:
    """Extract MAIN APP ICON with a 4x supersampled rounded-squircle vector mask."""
    crop = full_rgb[165:435, 45:308]
    fg = flood_exterior_bg(crop, dist_thresh=22.0)
    ys, xs = np.where(fg)
    x0, x1 = int(xs.min()), int(xs.max())
    y0, y1 = int(ys.min()), int(ys.max())

    # Inset 1.2px inside the outer dark drop-shadow rim so the squircle edge is ultra-crisp
    h, w, _ = crop.shape
    ss = 4
    mask_img = Image.new("L", (w * ss, h * ss), 0)
    draw = ImageDraw.Draw(mask_img)
    inset = 1.2
    lx = (x0 + inset) * ss
    ty = (y0 + inset) * ss
    rx = (x1 - inset) * ss
    # Bottom has a ~5px cast shadow in the showcase; trim bottom to match square aspect of w
    box_w = (x1 - x0) - 2 * inset
    by = ty + box_w * ss
    rad = box_w * 0.225 * ss
    draw.rounded_rectangle([lx, ty, rx, by], radius=rad, fill=255)
    alpha = np.array(mask_img.resize((w, h), Image.Resampling.LANCZOS), dtype=np.float32) / 255.0

    rgba = decontaminate_rgba(crop, alpha)
    return Image.fromarray(rgba, mode="RGBA")


def extract_taskbar_browser(full_rgb: np.ndarray) -> Image.Image:
    """Extract LP BROWSER Taskbar circular icon with a geometric 4x supersampled circle + hat check."""
    crop = full_rgb[162:283, 368:492]
    fg = flood_exterior_bg(crop, dist_thresh=21.0)
    ys, xs = np.where(fg)
    x0, x1 = float(xs.min()), float(xs.max())
    y0, y1 = float(ys.min()), float(ys.max())
    cx = 0.5 * (x0 + x1)
    cy = 0.5 * (y0 + y1)
    r = 0.5 * min(x1 - x0, y1 - y0) - 0.8

    h, w, _ = crop.shape
    ss = 4
    circ_img = Image.new("L", (w * ss, h * ss), 0)
    draw = ImageDraw.Draw(circ_img)
    draw.ellipse([(cx - r) * ss, (cy - r) * ss, (cx + r) * ss, (cy + r) * ss], fill=255)
    circ_alpha = np.array(circ_img.resize((w, h), Image.Resampling.LANCZOS), dtype=np.float32) / 255.0

    # Also allow any bright Nón lá brim tip if it peeks 1px past the circle
    contour_alpha = smooth_contour_mask(fg, erode_px=1.0)
    # Keep circle dominant, union only strongly-colored brim pixels if any
    bright = np.linalg.norm(crop - BG_RGB, axis=2) > 55.0
    alpha = np.maximum(circ_alpha, np.where(bright, contour_alpha, 0.0))

    rgba = decontaminate_rgba(crop, alpha)
    return Image.fromarray(rgba, mode="RGBA")


def extract_standard_icon(
    full_rgb: np.ndarray, box: tuple[int, int, int, int], dist_thresh: float = 20.0, erode_px: float = 0.8
) -> Image.Image:
    x0, y0, x1, y1 = box
    crop = full_rgb[y0:y1, x0:x1]
    fg = flood_exterior_bg(crop, dist_thresh=dist_thresh)
    alpha = smooth_contour_mask(fg, erode_px=erode_px)
    rgba = decontaminate_rgba(crop, alpha)
    return Image.fromarray(rgba, mode="RGBA")


def extract_anti_fingerprint(full_rgb: np.ndarray) -> Image.Image:
    """Unmix cyan-emerald ridges directly from #11192e so zero dark gaps remain between ridges."""
    crop = full_rgb[354:482, 372:494]
    h, w, _ = crop.shape

    # 1. Exterior flood-fill captures overall silhouette
    fg_outer = flood_exterior_bg(crop, dist_thresh=18.0)
    outer_alpha = smooth_contour_mask(fg_outer, erode_px=0.6)

    # 2. Confine shield solid mask strictly to the bottom-right shield box so it never walks into ridges
    yy, xx = np.mgrid[0:h, 0:w]
    in_shield_box = (xx >= int(w * 0.60)) & (yy >= int(h * 0.56))
    seed_x, seed_y = int(w * 0.76), int(h * 0.72)
    # Shield interior is bright green/cyan (G & B significantly above BG_RGB)
    bright_shield = ((crop[:, :, 1] - BG_RGB[1] > 35.0) | (crop[:, :, 2] - BG_RGB[2] > 35.0)) & in_shield_box
    shield_cc = np.zeros((h, w), dtype=bool)
    q: deque[tuple[int, int]] = deque()
    for dy in range(-4, 5):
        for dx in range(-4, 5):
            sy, sx = seed_y + dy, seed_x + dx
            if 0 <= sy < h and 0 <= sx < w and bright_shield[sy, sx] and not shield_cc[sy, sx]:
                shield_cc[sy, sx] = True
                q.append((sy, sx))
    while q:
        cy, cx = q.popleft()
        for ny, nx in ((cy - 1, cx), (cy + 1, cx), (cy, cx - 1), (cy, cx + 1)):
            if 0 <= ny < h and 0 <= nx < w and not shield_cc[ny, nx] and bright_shield[ny, nx]:
                shield_cc[ny, nx] = True
                q.append((ny, nx))

    shield_pil = (
        Image.fromarray((shield_cc.astype(np.uint8) * 255), mode="L")
        .filter(ImageFilter.MaxFilter(7))
        .filter(ImageFilter.GaussianBlur(radius=0.8))
    )
    shield_mask = np.clip(np.array(shield_pil, dtype=np.float32) / 220.0, 0.0, 1.0) * outer_alpha

    # 3. For the fingerprint ridges: measure POSITIVE cyan/mint luminance above BG_RGB
    #    so dark cast shadows behind the shield stay 100% transparent (alpha = 0)!
    ridge_signal = np.maximum(crop[:, :, 1] - BG_RGB[1], crop[:, :, 2] - BG_RGB[2])
    ridge_alpha = np.clip((ridge_signal - 14.0) / 42.0, 0.0, 1.0) * outer_alpha

    alpha = np.maximum(shield_mask, ridge_alpha)
    rgba = decontaminate_rgba(crop, alpha)
    return Image.fromarray(rgba, mode="RGBA")


def place_on_square(img: Image.Image, canvas_size: int, fill_ratio: float = 0.90) -> Image.Image:
    """Trim transparent borders and center cleanly on a square RGBA canvas."""
    arr = np.array(img)
    ys, xs = np.where(arr[:, :, 3] > 8)
    trimmed = img.crop((int(xs.min()), int(ys.min()), int(xs.max()) + 1, int(ys.max()) + 1))
    tw, th = trimmed.size
    scale = (canvas_size * fill_ratio) / max(tw, th)
    nw = max(1, int(round(tw * scale)))
    nh = max(1, int(round(th * scale)))
    resized = trimmed.resize((nw, nh), Image.Resampling.LANCZOS)
    out = Image.new("RGBA", (canvas_size, canvas_size), (0, 0, 0, 0))
    out.alpha_composite(resized, ((canvas_size - nw) // 2, (canvas_size - nh) // 2))
    return out


def encode_ico_dib_rgba(img: Image.Image) -> bytes:
    """Encode RGBA PIL image (< 256px) as 32-bit BGRA BITMAPINFOHEADER DIB + 1bpp AND mask."""
    rgba = np.array(img.convert("RGBA"), dtype=np.uint8)
    h, w, _ = rgba.shape
    xor_bytes = w * h * 4
    and_row_bytes = ((w + 31) // 32) * 4
    and_bytes = and_row_bytes * h

    header = struct.pack(
        "<IIIHHIIIIII",
        40,
        w,
        h * 2,
        1,
        32,
        0,
        xor_bytes + and_bytes,
        0,
        0,
        0,
        0,
    )
    # Flip vertically and swap RGBA -> BGRA
    bgra = rgba[::-1, :, [2, 1, 0, 3]].copy()
    # Zero RGB where alpha == 0
    zero_a = bgra[:, :, 3] == 0
    bgra[zero_a, 0:3] = 0

    and_buf = bytearray(and_bytes)
    for row_idx in range(h):
        src_y = h - 1 - row_idx
        row_off = row_idx * and_row_bytes
        for x in range(w):
            if rgba[src_y, x, 3] == 0:
                and_buf[row_off + (x // 8)] |= 1 << (7 - (x % 8))

    return header + bgra.tobytes() + bytes(and_buf)


def write_multi_ico(
    master: Image.Image, dst: Path, sizes: tuple[int, ...] = (256, 64, 48, 40, 32, 24, 16)
) -> None:
    payloads: list[tuple[int, bytes]] = []
    for sz in sizes:
        scaled = master.resize((sz, sz), Image.Resampling.LANCZOS)
        if sz >= 256:
            buf = io.BytesIO()
            scaled.save(buf, format="PNG", optimize=True)
            payloads.append((sz, buf.getvalue()))
        else:
            payloads.append((sz, encode_ico_dib_rgba(scaled)))

    count = len(payloads)
    out = bytearray(struct.pack("<HHH", 0, 1, count))
    offset = 6 + 16 * count
    for sz, data in payloads:
        dim = 0 if sz >= 256 else sz
        out.extend(struct.pack("<BBBBHHII", dim, dim, 0, 0, 1, 32, len(data), offset))
        offset += len(data)
    for _, data in payloads:
        out.extend(data)

    dst.write_bytes(bytes(out))
    print(f"[ok] {dst.relative_to(ROOT)} ({len(out):,} bytes, sizes={sizes})")


def main() -> None:
    BRAND_DIR.mkdir(parents=True, exist_ok=True)
    ICONS_DIR.mkdir(parents=True, exist_ok=True)
    NTP_ASSETS_DIR.mkdir(parents=True, exist_ok=True)
    ntp_dir = NTP_ASSETS_DIR

    full_rgb = np.array(Image.open(SRC_SHOWCASE).convert("RGB"), dtype=np.float32)

    raw_icons: dict[str, Image.Image] = {
        "app-main": extract_main_app(full_rgb),
        "taskbar": extract_taskbar_browser(full_rgb),
        "multi-profiles": extract_standard_icon(full_rgb, (516, 164, 650, 280), dist_thresh=20.0, erode_px=0.8),
        "ghost-mode": extract_standard_icon(full_rgb, (672, 180, 804, 276), dist_thresh=20.0, erode_px=0.8),
        "proxy-ip": extract_standard_icon(full_rgb, (834, 164, 966, 282), dist_thresh=20.0, erode_px=0.8),
        "anti-fingerprint": extract_anti_fingerprint(full_rgb),
        "cookie-manager": extract_standard_icon(full_rgb, (516, 352, 650, 482), dist_thresh=20.0, erode_px=0.8),
        "user-agent": extract_standard_icon(full_rgb, (672, 366, 814, 480), dist_thresh=20.0, erode_px=0.8),
        "encryption-vault": extract_standard_icon(full_rgb, (840, 358, 948, 480), dist_thresh=20.0, erode_px=0.8),
    }

    # 1. Launcher Main App Icon (squircle at 92% fill for macOS/Windows standard margin, 96% for .ico)
    launcher_1024 = place_on_square(raw_icons["app-main"], 1024, fill_ratio=0.90)
    launcher_ico_master = place_on_square(raw_icons["app-main"], 512, fill_ratio=0.95)

    launcher_1024.save(ICONS_DIR / "icon.png", format="PNG", optimize=True)
    tauri_png_specs = {
        "32x32.png": 32,
        "128x128.png": 128,
        "128x128@2x.png": 256,
        "Square30x30Logo.png": 30,
        "Square44x44Logo.png": 44,
        "Square71x71Logo.png": 71,
        "Square89x89Logo.png": 89,
        "Square107x107Logo.png": 107,
        "Square142x142Logo.png": 142,
        "Square150x150Logo.png": 150,
        "Square284x284Logo.png": 284,
        "Square310x310Logo.png": 310,
        "StoreLogo.png": 50,
    }
    for fname, px in tauri_png_specs.items():
        launcher_ico_master.resize((px, px), Image.Resampling.LANCZOS).save(
            ICONS_DIR / fname, format="PNG", optimize=True
        )
    write_multi_ico(launcher_ico_master, ICONS_DIR / "icon.ico")

    # 2. Browser Taskbar & Shortcut Icon (`LP BROWSER — Taskbar`)
    browser_512 = place_on_square(raw_icons["taskbar"], 512, fill_ratio=0.95)
    # Save straight 8-bit RGBA PNG (profile_icon.rs requires ColorType::Rgba + BitDepth::Eight!)
    browser_512.convert("RGBA").save(ICONS_DIR / "browser-icon.png", format="PNG")
    write_multi_ico(browser_512, ICONS_DIR / "browser-icon.ico")

    for px in (16, 32, 64):
        browser_512.resize((px, px), Image.Resampling.LANCZOS).save(
            PUBLIC_DIR / f"lp-browser-{px}.png", format="PNG", optimize=True
        )

    # 3. Export crisp 256x256 PNGs to `public/brand-icons/` + compact 96x96 PNGs for New Tab Page
    placed_256: dict[str, Image.Image] = {}
    for slug, img in raw_icons.items():
        fill = 0.94 if slug in ("app-main", "taskbar") else 0.90
        sq256 = place_on_square(img, 256, fill_ratio=fill)
        placed_256[slug] = sq256
        sq256.save(BRAND_DIR / f"{slug}.png", format="PNG", optimize=True)

        # Quantized 96x96 PNG for compact base64 / WebUI embedding (~3-5 KB each)
        sq96 = sq256.resize((96, 96), Image.Resampling.LANCZOS)
        q96 = sq96.quantize(colors=256, method=Image.Quantize.FASTOCTREE, dither=Image.Dither.NONE)
        q96.save(ntp_dir / f"{slug}.png", format="PNG", optimize=True)
        sz_kb = (ntp_dir / f"{slug}.png").stat().st_size / 1024.0
        print(f"[ok] brand-icons/{slug}.png + ntp 96px ({sz_kb:.1f} KB)")

    # Also save favicon.png (64x64) in public/
    placed_256["app-main"].resize((64, 64), Image.Resampling.LANCZOS).save(
        PUBLIC_DIR / "favicon.png", format="PNG", optimize=True
    )

    # 4. Build verification contact sheet on BOTH pure white (#ffffff) and dark navy (#0c1527)
    cols = len(placed_256)
    cell = 128
    sheet = Image.new("RGBA", (cols * cell, cell * 2 + 20), (255, 255, 255, 255))
    dark_strip = Image.new("RGBA", (cols * cell, cell + 10), (12, 21, 39, 255))
    sheet.paste(dark_strip, (0, cell + 10))

    for i, (slug, sq256) in enumerate(placed_256.items()):
        icon96 = sq256.resize((96, 96), Image.Resampling.LANCZOS)
        x = i * cell + (cell - 96) // 2
        sheet.alpha_composite(icon96, (x, 16))
        sheet.alpha_composite(icon96, (x, cell + 22))

    verify_path = SCRATCH_DIR / "_verified_light_dark.png"
    sheet.save(verify_path)
    print(f"[verified] Saved contact sheet -> {verify_path}")


if __name__ == "__main__":
    main()
