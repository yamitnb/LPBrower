import os, re

runtime_dir = os.path.expandvars(r"%APPDATA%\shardx-launcher\runtime\ShardX-Windows")
dll_path = os.path.join(runtime_dir, "chrome.dll")

with open(dll_path, "rb") as f:
    dll = f.read()

# Find all standalone null-terminated "shardx" strings (case-insensitive)
for m in re.finditer(rb'\x00(shardx)\x00', dll, re.IGNORECASE):
    pos = m.start() + 1
    ctx = dll[max(0, pos-50):min(len(dll), pos+60)]
    print(f"standalone '{m.group(1).decode()}' @0x{pos:x}: {ctx!r}")

# Find all occurrences of "chrome://shardx"
for m in re.finditer(rb'chrome://shardx[^\x00]{0,20}', dll, re.IGNORECASE):
    pos = m.start()
    ctx = dll[max(0, pos-40):min(len(dll), pos+60)]
    print(f"url '{m.group(0).decode()}' @0x{pos:x}: {ctx!r}")

# Also check if "newtab" or startup prefs appear near 0x111fad40
for m in re.finditer(rb'shardx_start|start_page|ShardXUI|ShardXStart', dll, re.IGNORECASE):
    pos = m.start()
    ctx = dll[max(0, pos-40):min(len(dll), pos+60)]
    print(f"webui '{m.group(0).decode()}' @0x{pos:x}: {ctx!r}")
