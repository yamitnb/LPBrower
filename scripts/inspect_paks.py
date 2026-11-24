import os, struct, gzip, re

runtime_dir = os.path.expandvars(r"%APPDATA%\shardx-launcher\runtime\ShardX-Windows")

def parse_pak(path):
    with open(path, 'rb') as f:
        data = f.read()
    if len(data) < 12:
        return []
    version = struct.unpack_from('<I', data, 0)[0]
    entries = []
    if version == 4:
        num_entries = struct.unpack_from('<I', data, 4)[0]
        encoding = data[8]
        offset = 9
        raw_entries = []
        for i in range(num_entries + 1):
            rid, off = struct.unpack_from('<HI', data, offset)
            offset += 6
            raw_entries.append((rid, off))
        for i in range(num_entries):
            rid, start = raw_entries[i]
            _, end = raw_entries[i+1]
            entries.append((rid, data[start:end]))
    elif version == 5:
        encoding, = struct.unpack_from('<B', data, 4)
        # 3 bytes padding
        num_entries, num_aliases = struct.unpack_from('<HH', data, 8)
        offset = 12
        raw_entries = []
        for i in range(num_entries + 1):
            rid, off = struct.unpack_from('<HI', data, offset)
            offset += 6
            raw_entries.append((rid, off))
        for i in range(num_entries):
            rid, start = raw_entries[i]
            _, end = raw_entries[i+1]
            entries.append((rid, data[start:end]))
    return entries

pat = re.compile(rb'shard|proxyshard|wyocore', re.IGNORECASE)

for rel in ["resources.pak", "chrome_100_percent.pak", "chrome_200_percent.pak", r"locales\en-US.pak", r"locales\vi.pak"]:
    p = os.path.join(runtime_dir, rel)
    if not os.path.exists(p):
        print(f"MISSING: {rel}")
        continue
    entries = parse_pak(p)
    hits = []
    for rid, raw in entries:
        content = raw
        is_gz = False
        if raw[:2] == b'\x1f\x8b':
            try:
                content = gzip.decompress(raw)
                is_gz = True
            except Exception:
                pass
        if pat.search(content):
            snippet = content[:300].decode('utf-8', errors='replace').replace('\n', ' ')
            hits.append((rid, len(raw), is_gz, snippet[:180]))
    print(f"=== {rel}: {len(entries)} entries, {len(hits)} hits ===")
    for h in hits[:30]:
        print(f"  ID={h[0]} len={h[1]} gz={h[2]}: {h[3]}")
    if len(hits) > 30:
        print(f"  ... and {len(hits)-30} more hits")
