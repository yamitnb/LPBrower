"""Generate Apple .icns icon bundles for LPBrower Launcher and LPBrower Browser."""

from __future__ import annotations

import io
import struct
from pathlib import Path
from PIL import Image

ROOT = Path(__file__).resolve().parent.parent
ICONS_DIR = ROOT / "src-tauri" / "icons"

# Standard modern macOS ICNS PNG chunk types (1x and Retina @2x):
ICNS_SPECS: list[tuple[bytes, int]] = [
    (b"ic11", 32),    # 16x16@2x
    (b"ic12", 64),    # 32x32@2x
    (b"ic07", 128),   # 128x128
    (b"ic13", 256),   # 128x128@2x
    (b"ic08", 256),   # 256x256
    (b"ic14", 512),   # 256x256@2x
    (b"ic09", 512),   # 512x512
    (b"ic10", 1024),  # 512x512@2x (1024x1024)
]


def build_icns(src_png: Path, dst_icns: Path) -> None:
    src = Image.open(src_png).convert("RGBA")
    png_cache: dict[int, bytes] = {}
    chunks: list[bytes] = []

    for ostype, px in ICNS_SPECS:
        if px not in png_cache:
            resized = src.resize((px, px), Image.Resampling.LANCZOS)
            buf = io.BytesIO()
            resized.save(buf, format="PNG", optimize=True)
            png_cache[px] = buf.getvalue()
        data = png_cache[px]
        chunks.append(struct.pack(">4sI", ostype, 8 + len(data)) + data)

    body = b"".join(chunks)
    icns_bytes = struct.pack(">4sI", b"icns", 8 + len(body)) + body
    dst_icns.write_bytes(icns_bytes)
    print(f"[ok] {dst_icns.relative_to(ROOT)} ({len(icns_bytes):,} bytes, {len(chunks)} chunks)")


def verify_icns(icns_path: Path) -> None:
    raw = icns_path.read_bytes()
    magic, total_len = struct.unpack(">4sI", raw[:8])
    assert magic == b"icns", f"Bad magic in {icns_path}: {magic!r}"
    assert total_len == len(raw), f"Length mismatch in {icns_path}: {total_len} != {len(raw)}"
    off = 8
    seen = []
    while off < total_len:
        ctype, clen = struct.unpack(">4sI", raw[off : off + 8])
        payload = raw[off + 8 : off + clen]
        assert payload[:8] == b"\x89PNG\r\n\x1a\n", f"Chunk {ctype!r} is not valid PNG"
        seen.append(f"{ctype.decode()}({len(payload)}B)")
        off += clen
    assert off == total_len
    print(f"[verified] {icns_path.name}: {', '.join(seen)}")


def main() -> None:
    launcher_png = ICONS_DIR / "icon.png"
    browser_png = ICONS_DIR / "browser-icon.png"
    launcher_icns = ICONS_DIR / "icon.icns"
    browser_icns = ICONS_DIR / "browser-icon.icns"

    build_icns(launcher_png, launcher_icns)
    build_icns(browser_png, browser_icns)
    verify_icns(launcher_icns)
    verify_icns(browser_icns)


if __name__ == "__main__":
    main()
