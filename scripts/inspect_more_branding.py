import os, struct, gzip, brotli, re

runtime_dir = os.path.expandvars(r"%APPDATA%\shardx-launcher\runtime\ShardX-Windows")

# Check chrome.dll around 0x112029e8 and 0x1122f460 and 0x1052a648 and all ShardBrowser/ShardX UI strings
dll_path = os.path.join(runtime_dir, "chrome.dll")
with open(dll_path, "rb") as f:
    for name, off in [("version_str", 0x1052a630), ("chrome_url", 0x11202980), ("utf16_shardx", 0x1122f440), ("utf16_err", 0x1052a790)]:
        f.seek(off)
        b = f.read(160)
        print(f"=== {name} @0x{off:x} ===")
        print("  raw:", repr(b[:120]))

# Check resources.pak for any SVG/HTML containing product_logo or shardx or Chromium logo
pak_path = os.path.join(runtime_dir, "resources.pak")
with open(pak_path, "rb") as f:
    data = f.read()
num_entries, num_aliases = struct.unpack_from('<HH', data, 8)
raw_entries = [struct.unpack_from('<HI', data, 12 + i*6) for i in range(num_entries + 1)]
for i in range(num_entries):
    rid, start = raw_entries[i]
    _, end = raw_entries[i+1]
    raw = data[start:end]
    dec = raw
    if raw[:2] == b'\x1f\x8b':
        try: dec = gzip.decompress(raw)
        except: pass
    elif raw[:2] == b'\x1e\x9b':
        try: dec = brotli.decompress(raw[8:])
        except: pass
    if b'product_logo' in dec or b'shardx' in dec.lower():
        if len(dec) < 20000:
            print(f"resources.pak ID={rid} len={len(dec)}: {dec[:160]!r}")
