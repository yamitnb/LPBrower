import os, struct, re

runtime_dir = os.path.expandvars(r"%APPDATA%\shardx-launcher\runtime\ShardX-Windows")
dll_path = os.path.join(runtime_dir, "chrome.dll")

with open(dll_path, "rb") as f:
    for label, off, sz in [
        ("window_title_area", 0x1122f3c0, 320),
        ("chrome_shardx_url_area", 0x11202980, 320),
    ]:
        f.seek(off)
        b = f.read(sz)
        print(f"=== {label} @0x{off:x} ===")
        print(repr(b))

# Also check chrome_100_percent.pak & chrome_200_percent.pak for IDs 15300..15315 and any other blue chromium icons
for pak_name in ["chrome_100_percent.pak", "chrome_200_percent.pak"]:
    p = os.path.join(runtime_dir, pak_name)
    with open(p, "rb") as f:
        data = f.read()
    num_entries, num_aliases = struct.unpack_from('<HH', data, 8)
    raw_entries = [struct.unpack_from('<HI', data, 12 + i*6) for i in range(num_entries + 1)]
    print(f"\n=== {pak_name} IDs 15295..15320 ===")
    for i in range(num_entries):
        rid, start = raw_entries[i]
        _, end = raw_entries[i+1]
        if 15295 <= rid <= 15320:
            raw = data[start:end]
            if raw[:8] == b'\x89PNG\r\n\x1a\n':
                w, h = struct.unpack_from('>II', raw, 16)
                print(f"  ID={rid} PNG {w}x{h} len={len(raw)}")
            else:
                print(f"  ID={rid} len={len(raw)} head={raw[:16]!r}")
