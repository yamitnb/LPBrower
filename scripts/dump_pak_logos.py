import os, struct

runtime_dir = os.path.expandvars(r"%APPDATA%\shardx-launcher\runtime\ShardX-Windows")
pak_path = os.path.join(runtime_dir, "chrome_200_percent.pak")

with open(pak_path, "rb") as f:
    data = f.read()

version = struct.unpack_from('<I', data, 0)[0]
num_entries, num_aliases = struct.unpack_from('<HH', data, 8)
offset = 12
raw_entries = [struct.unpack_from('<HI', data, offset + i*6) for i in range(num_entries + 1)]

out_dir = r"C:\Users\Admin\.gemini\antigravity\brain\6659144a-f4cf-41d2-a473-a1d186c1fdb7\scratch\pak200_logos"
os.makedirs(out_dir, exist_ok=True)

# Find PNGs that look like square product logos (32x32, 48x48, 64x64, 96x96, 128x128, 256x256)
logos = []
for i in range(num_entries):
    rid, start = raw_entries[i]
    _, end = raw_entries[i+1]
    raw = data[start:end]
    if raw[:8] == b'\x89PNG\r\n\x1a\n':
        w, h = struct.unpack_from('>II', raw, 16)
        if w == h and w >= 32 and len(raw) > 800:
            logos.append((rid, w, h, len(raw)))
            with open(os.path.join(out_dir, f"id_{rid}_{w}x{h}.png"), "wb") as out:
                out.write(raw)

print(f"Found {len(logos)} square >=32px PNGs in chrome_200_percent.pak:")
for l in logos:
    print(" ", l)
