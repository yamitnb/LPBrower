import os, struct, gzip, re

runtime_dir = os.path.expandvars(r"%APPDATA%\shardx-launcher\runtime\ShardX-Windows")

# 1. Check chrome.dll for ShardBrowser / ShardX / shardx.com / chrome://shardx / html
dll_path = os.path.join(runtime_dir, "chrome.dll")
with open(dll_path, "rb") as f:
    dll = f.read()

print(f"chrome.dll size: {len(dll)} bytes")

for term in [b"ShardBrowser", b"ShardX", b"shardx.com", b"chrome://shardx", b"ProxyShard", b"shardx-profile", b"Wyocore"]:
    matches = [m.start() for m in re.finditer(re.escape(term), dll, re.IGNORECASE)]
    print(f"ASCII '{term.decode()}': {len(matches)} matches")
    for pos in matches[:10]:
        ctx = dll[max(0, pos-40):min(len(dll), pos+100)]
        printable = ''.join(chr(b) if 32 <= b < 127 else '.' for b in ctx)
        print(f"  @0x{pos:x}: {printable}")

for term_s in ["ShardBrowser", "ShardX", "shardx.com", "chrome://shardx", "ProxyShard"]:
    term = term_s.encode("utf-16le")
    matches = [m.start() for m in re.finditer(re.escape(term), dll)]
    print(f"UTF16 '{term_s}': {len(matches)} matches")
    for pos in matches[:5]:
        ctx = dll[max(0, pos-40):min(len(dll), pos+120)]
        try:
            s = ctx.decode("utf-16le", errors="replace")
            s = ''.join(c if 32 <= ord(c) < 127 else '.' for c in s)
        except Exception:
            s = ""
        print(f"  @0x{pos:x}: {s}")

# 2. Check brotli entries in resources.pak
pak_path = os.path.join(runtime_dir, "resources.pak")
with open(pak_path, "rb") as f:
    data = f.read()
version = struct.unpack_from('<I', data, 0)[0]
num_entries, num_aliases = struct.unpack_from('<HH', data, 8)
print(f"\nresources.pak v{version}, entries={num_entries}, aliases={num_aliases}")
br_count = 0
gz_count = 0
offset = 12
raw_entries = [struct.unpack_from('<HI', data, offset + i*6) for i in range(num_entries + 1)]
for i in range(num_entries):
    rid, start = raw_entries[i]
    _, end = raw_entries[i+1]
    raw = data[start:end]
    if raw[:2] == b'\x1f\x8b':
        gz_count += 1
    elif raw[:2] == b'\x1e\x9b':
        br_count += 1
print(f"gz={gz_count}, br={br_count}")

# Try decompressing brotli if brotli module available, else check
try:
    import brotli
    has_br = True
except ImportError:
    has_br = False
print("has brotli:", has_br)

if has_br:
    pat = re.compile(rb'shard|proxyshard|wyocore', re.IGNORECASE)
    for i in range(num_entries):
        rid, start = raw_entries[i]
        _, end = raw_entries[i+1]
        raw = data[start:end]
        if raw[:2] == b'\x1e\x9b':
            # Chromium brotli header: 2 magic bytes (0x1e, 0x9b) + 6 bytes uncompressed size
            try:
                dec = brotli.decompress(raw[8:])
                if pat.search(dec):
                    print(f"  BR HIT ID={rid} len={len(dec)}: {dec[:200].decode('utf-8', errors='replace')}")
            except Exception as e:
                pass
