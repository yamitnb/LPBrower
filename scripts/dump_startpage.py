import os, struct

runtime_dir = os.path.expandvars(r"%APPDATA%\shardx-launcher\runtime\ShardX-Windows")
dll_path = os.path.join(runtime_dir, "chrome.dll")

with open(dll_path, "rb") as f:
    f.seek(0x111fad00)
    chunk = f.read(0x4000)

# Find start of <!DOCTYPE or <html
start = chunk.find(b"<!DOCTYPE")
if start == -1:
    start = chunk.find(b"<html")
end = chunk.find(b"</html>", start) + len(b"</html>")
html_bytes = chunk[start:end]
print(f"HTML offset in chrome.dll: 0x{0x111fad00 + start:x} .. 0x{0x111fad00 + end:x} (len={len(html_bytes)})")
# Check trailing null byte
after = chunk[end:end+16]
print(f"Bytes right after </html>: {after!r}")
print("========== EMBEDDED START PAGE HTML ==========")
print(html_bytes.decode("utf-8", errors="replace"))
print("==============================================")

# Check all locales/*.pak files count
locales_dir = os.path.join(runtime_dir, "locales")
paks = [f for f in os.listdir(locales_dir) if f.endswith(".pak") and "_" not in f]
print(f"\nMain locale .pak files ({len(paks)}): {', '.join(sorted(paks)[:15])} ...")
