import os, re

runtime_dir = os.path.expandvars(r"%APPDATA%\shardx-launcher\runtime\ShardX-Windows")
dll_path = os.path.join(runtime_dir, "chrome.dll")

with open(dll_path, "rb") as f:
    dll = f.read()

# Find all printable ASCII null-terminated-ish strings containing 'shard' (case-insensitive)
strings = set()
for m in re.finditer(rb'[ -~]{0,80}shard[ -~]{0,80}', dll, re.IGNORECASE):
    s = m.group(0).decode('ascii', errors='ignore')
    # Skip if inside the HTML block 0x111fad30..0x111fd940
    if 0x111fad30 <= m.start() <= 0x111fd940:
        continue
    strings.add((m.start(), s))

print(f"Found {len(strings)} non-HTML ASCII strings with 'shard' in chrome.dll:")
for pos, s in sorted(strings):
    print(f"  @0x{pos:x}: {s}")
