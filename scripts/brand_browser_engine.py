import os
import re
import struct
import ctypes
from ctypes import wintypes

RUNTIME_DIR = os.path.expandvars(r"%APPDATA%\LPBrower-launcher\runtime\ShardX-Windows")
PROJECT_DIR = r"g:\Vibe Tool\LPBrower"
ICO_PATH = os.path.join(PROJECT_DIR, r"src-tauri\icons\browser-icon.ico")
LP_ICON_16 = os.path.join(PROJECT_DIR, r"public\lp-browser-16.png")
LP_ICON_32 = os.path.join(PROJECT_DIR, r"public\lp-browser-32.png")
LP_ICON_64 = os.path.join(PROJECT_DIR, r"public\lp-browser-64.png")

START_PAGE_TEMPLATE = """<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<meta name="color-scheme" content="light dark">
<title>LPBrower</title>
<style>
/* LPBrower start page (chrome://lphome). One file, no network requests. */
:root {
  --font-sans: "Inter Variable", "Inter", ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, "PingFang SC", "Microsoft YaHei UI", sans-serif;
  --bg-page: #f5f5f5;
  --bg-tile-fill: #ffffff;
  --bg-on-tile: #0000000a;
  --shadow-tile: 0 1px 2px 0 #0a0d1408;
  --text-strong: #171717;
  --text-sub: #4d4d4d;
  --text-soft: #696969;
  --icon-sub: #4d4d4d;
  --primary-base: #2563eb;
  --primary-text: #1d4ed8;
  --primary-tint: #2563eb1a;
  --primary-tint-hover: #2563eb2e;
  --focus: #2563eb;
  --focus-width: 2px;
  --radius-tile: 10px;
  --radius-chip: 6px;
  --radius-chip-sm: 4px;
  --spacing-inset: 16px;
  --spacing-inset-side: 18px;
  --tracking-small: -0.01em;
  --tracking-medium: -0.015em;
  --tracking-large: -0.02em;
  --ease-out: cubic-bezier(0.23, 1, 0.32, 1);
  --motion-base: 150ms;
  color-scheme: light;
}
@media (prefers-color-scheme: dark) {
  :root {
    --bg-page: #191919;
    --bg-tile-fill: #ffffff0a;
    --bg-on-tile: #ffffff0a;
    --shadow-tile: 0 0 #0000;
    --text-strong: #ededed;
    --text-sub: #bababa;
    --text-soft: #9e9e9e;
    --icon-sub: #bababa;
    --primary-base: #3b82f6;
    --primary-text: #93c5fd;
    --primary-tint: #3b82f638;
    --primary-tint-hover: #3b82f64d;
    --focus: #93c5fd;
    color-scheme: dark;
  }
}
:root:lang(zh) {
  --tracking-small: 0em;
  --tracking-medium: 0em;
  --tracking-large: 0em;
}
*, *::before, *::after { box-sizing: border-box; }
html, body {
  margin: 0;
  min-height: 100%;
  background: var(--bg-page);
  color: var(--text-strong);
  font-family: var(--font-sans);
  font-size: 13px;
  line-height: 20px;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}
::selection { background-color: var(--primary-tint-hover); }
.page {
  min-height: 100vh;
  display: flex;
  flex-direction: column;
  justify-content: center;
  padding: 48px 32px 64px;
}
.column {
  width: 100%;
  max-width: 760px;
  margin-inline: auto;
}
.mark {
  display: block;
  width: 34px;
  height: 30px;
  color: var(--primary-base);
}
h1 {
  margin: 16px 0 0;
  font-size: 26px;
  line-height: 32px;
  font-weight: 600;
  letter-spacing: var(--tracking-large);
  text-wrap: balance;
}
.lede {
  margin: 4px 0 0;
  font-size: 15px;
  line-height: 20px;
  font-weight: 400;
  letter-spacing: var(--tracking-medium);
  color: var(--text-sub);
  text-wrap: pretty;
}
.features {
  margin-top: 32px;
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px;
}
.feature--wide { grid-column: 1 / -1; }
@media (max-width: 759px) {
  .features { grid-template-columns: 1fr; }
}
.feature {
  min-width: 0;
  padding: var(--spacing-inset) var(--spacing-inset-side);
  border-radius: var(--radius-tile);
  background: var(--bg-tile-fill);
  box-shadow: var(--shadow-tile);
}
.feature-head {
  display: flex;
  align-items: center;
  gap: 8px;
  min-height: 20px;
}
.icon {
  flex: none;
  width: 14px;
  height: 14px;
  color: var(--icon-sub);
}
h2 {
  margin: 0;
  min-width: 0;
  font-size: 15px;
  line-height: 20px;
  font-weight: 600;
  letter-spacing: var(--tracking-medium);
  text-wrap: balance;
}
.feature p {
  margin: 6px 0 0;
  max-width: 36em;
  font-size: 13px;
  line-height: 20px;
  letter-spacing: var(--tracking-small);
  color: var(--text-sub);
  text-wrap: pretty;
}
.chip {
  display: inline-flex;
  align-items: center;
  padding: 1px 7px;
  border-radius: var(--radius-chip);
  font-size: 12px;
  line-height: 16px;
  font-weight: 500;
  letter-spacing: var(--tracking-small);
  white-space: nowrap;
}
.chip--brand {
  margin-left: auto;
  background: var(--primary-tint);
  color: var(--primary-text);
}
.surfaces {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  margin: 12px 0 0;
  padding: 0;
  list-style: none;
}
.surfaces .chip {
  background: var(--bg-on-tile);
  color: var(--text-sub);
}
.foot { margin-top: 24px; }
.link {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  margin-left: -2px;
  padding: 0 2px;
  border-radius: var(--radius-chip-sm);
  font-size: 13px;
  line-height: 20px;
  font-weight: 500;
  letter-spacing: var(--tracking-small);
  color: var(--text-sub);
  text-decoration: none;
  transition: color var(--motion-base) var(--ease-out);
}
.link .icon { color: currentColor; }
.link:hover {
  color: var(--text-strong);
  text-decoration: underline;
  text-decoration-thickness: 1px;
  text-underline-offset: 3px;
}
.link:focus-visible {
  outline: var(--focus-width) solid var(--focus);
  outline-offset: 2px;
}
@media (prefers-reduced-motion: reduce) {
  .link { transition: none; }
}
@media (forced-colors: active) {
  .feature { border: 1px solid CanvasText; }
  .chip { border: 1px solid CanvasText; }
}
</style>
</head>
<body>
<main class="page">
  <div class="column">
    <header>
      <svg class="mark" viewBox="0 0 20 18" fill="none" aria-hidden="true">
        <path d="M1 2C1 1.44772 1.44772 1 2 1H4.5C5.05228 1 5.5 1.44772 5.5 2V12.5H9C9.55228 12.5 10 12.9477 10 13.5V15.5C10 16.0523 9.55228 16.5 9 16.5H2C1.44772 16.5 1 16.0523 1 15.5V2Z" fill="currentColor"/>
        <path fill-rule="evenodd" clip-rule="evenodd" d="M8 2C8 1.44772 8.44772 1 9 1H13.8C16.948 1 19.5 3.552 19.5 6.7C19.5 9.848 16.948 12.4 13.8 12.4H11.5V15.5C11.5 16.0523 11.0523 16.5 10.5 16.5H8.5C7.94772 16.5 7.5 16.0523 7.5 15.5V2H8ZM11.5 4V9.4H13.8C15.291 9.4 16.5 8.191 16.5 6.7C16.5 5.209 15.291 4 13.8 4H11.5Z" fill="currentColor"/>
        <circle cx="13.8" cy="6.7" r="1.5" fill="currentColor"/>
      </svg>
      <h1>LPBrower</h1>
      <p class="lede">Anti-detect browser built for isolated profiles, fingerprints, and privacy.</p>
    </header>

    <section class="features" aria-label="What LPBrower does">
      <article class="feature feature--wide">
        <div class="feature-head">
          <svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            <path d="M7.42857 3.36201C11.3996 0.664089 17.136 2.12432 19.2301 6.41803M10.8571 21.9236C15.5839 22.5822 20 18.8952 20 14.2103V10.3333M7.42857 20.6058C5.35602 19.1977 4 16.8583 4 14.2103V9.75757C4 8.57285 4.27144 7.44988 4.75704 6.44444"/>
            <path d="M16 13.8C16 16.1196 14.2091 18 12 18C9.79086 18 8 16.1196 8 13.8V10.2C8 9.55584 8.13811 8.94555 8.38493 8.4M12 6C14.2091 6 16 7.8804 16 10.2"/>
            <path d="M12 10.5L12 13.5"/>
          </svg>
          <h2>A real machine per profile</h2>
        </div>
        <p>Every fingerprint surface a real browser exposes is pinned to this profile. Two profiles on the same computer appear as two distinct devices.</p>
        <ul class="surfaces" aria-label="Pinned per profile">
          <li class="chip">Client Hints</li>
          <li class="chip">Navigator</li>
          <li class="chip">Screen</li>
          <li class="chip">WebGL</li>
          <li class="chip">WebGPU</li>
          <li class="chip">Canvas</li>
          <li class="chip">Audio</li>
          <li class="chip">Fonts</li>
          <li class="chip">Language</li>
          <li class="chip">Time zone</li>
          <li class="chip">Geolocation</li>
          <li class="chip">WebAuthn</li>
        </ul>
      </article>

      <article class="feature">
        <div class="feature-head">
          <svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            <path d="M19 9H6.65856C5.65277 9 5.14987 9 5.02472 8.69134C4.89957 8.38268 5.25517 8.01942 5.96637 7.29289L8.21091 5"/>
            <path d="M5 15H17.3414C18.3472 15 18.8501 15 18.9753 15.3087C19.1004 15.6173 18.7448 15.9806 18.0336 16.7071L15.7891 19"/>
          </svg>
          <h2>SOCKS5 with UDP</h2>
          <span class="chip chip--brand">LPBrower</span>
        </div>
        <p>Isolated proxy routing prevents WebRTC and QUIC from leaking your real IP over UDP, ensuring sites see only the profile's configured address.</p>
      </article>

      <article class="feature">
        <div class="feature-head">
          <svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            <path d="M16.5962 16.5962C14.0578 19.1346 9.94221 19.1346 7.40381 16.5962C4.8654 14.0578 4.8654 9.94221 7.40381 7.40381C8.92815 5.87946 11.0212 5.27051 13 5.57693"/>
            <path d="M12 12L16.9497 7.05025C17.9829 6.01707 18.4995 5.50048 18.3867 4.75173C18.2739 4.00298 17.7382 3.7198 16.6667 3.15343C15.2736 2.41699 13.6855 2 12 2C6.47715 2 2 6.47715 2 12C2 17.5228 6.47715 22 12 22C17.5228 22 22 17.5228 22 12C22 10.5778 21.7031 9.22492 21.1679 8"/>
          </svg>
          <h2>Isolated Storage</h2>
          <span class="chip chip--brand">LPBrower</span>
        </div>
        <p>Cookies, local storage, cache, extensions, and bookmarks stay strictly separated per profile for clean multi-account sessions.</p>
      </article>
    </section>

    <footer class="foot">
      <a class="link" href="https://github.com/LPBrower/LPBrower">
        github.com/LPBrower/LPBrower
        <svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
          <path d="M9 6.65032C9 6.65032 15.9383 6.10759 16.9154 7.08463C17.8924 8.06167 17.3496 15 17.3496 15M16.5 7.5L6.5 17.5"/>
        </svg>
      </a>
    </footer>
  </div>
</main>
</body>
</html>"""


def patch_pak_v5(pak_path, transform_fn):
    with open(pak_path, "rb") as f:
        data = f.read()
    if len(data) < 12:
        return 0
    version = struct.unpack_from("<I", data, 0)[0]
    if version != 5:
        return 0
    encoding = data[4]
    num_entries, num_aliases = struct.unpack_from("<HH", data, 8)
    offset = 12
    raw_entries = []
    for i in range(num_entries + 1):
        rid, off = struct.unpack_from("<HI", data, offset)
        offset += 6
        raw_entries.append((rid, off))
    aliases = []
    for i in range(num_aliases):
        rid, idx = struct.unpack_from("<HH", data, offset)
        offset += 4
        aliases.append((rid, idx))

    changed = 0
    new_payloads = []
    for i in range(num_entries):
        rid, start = raw_entries[i]
        _, end = raw_entries[i + 1]
        raw = data[start:end]
        updated = transform_fn(rid, raw)
        if updated is not None and updated != raw:
            changed += 1
            new_payloads.append((rid, updated))
        else:
            new_payloads.append((rid, raw))

    if changed == 0:
        return 0

    header_size = 12 + (num_entries + 1) * 6 + num_aliases * 4
    cur_off = header_size
    out = bytearray()
    out += struct.pack("<IBBBBHH", 5, encoding, 0, 0, 0, num_entries, num_aliases)
    for rid, payload in new_payloads:
        out += struct.pack("<HI", rid, cur_off)
        cur_off += len(payload)
    out += struct.pack("<HI", 0, cur_off)
    for rid, idx in aliases:
        out += struct.pack("<HH", rid, idx)
    for _, payload in new_payloads:
        out += payload

    tmp = pak_path + ".tmp"
    with open(tmp, "wb") as f:
        f.write(out)
    os.replace(tmp, pak_path)
    return changed


def patch_locales(runtime_dir):
    locales_dir = os.path.join(runtime_dir, "locales")
    if not os.path.isdir(locales_dir):
        return
    total_files = 0
    total_entries = 0

    def repl(rid, raw):
        if b"Shard" not in raw and b"shard" not in raw:
            return None
        new_raw = raw.replace(b"ShardBrowser", b"LPBrower").replace(b"ShardX", b"LPBrower")
        return new_raw if new_raw != raw else None

    for name in sorted(os.listdir(locales_dir)):
        if not name.endswith(".pak"):
            continue
        p = os.path.join(locales_dir, name)
        c = patch_pak_v5(p, repl)
        if c > 0:
            total_files += 1
            total_entries += c
    print(f"Patched {total_entries} strings across {total_files} locale .pak files")


def patch_scale_paks(runtime_dir):
    with open(LP_ICON_16, "rb") as f:
        png16 = f.read()
    with open(LP_ICON_32, "rb") as f:
        png32 = f.read()
    with open(LP_ICON_64, "rb") as f:
        png64 = f.read()

    # chrome_100_percent.pak: ID 15304 = 32x32, ID 15306 = 16x16
    p100 = os.path.join(runtime_dir, "chrome_100_percent.pak")
    if os.path.exists(p100):
        c = patch_pak_v5(p100, lambda rid, raw: png32 if rid == 15304 else (png16 if rid == 15306 else None))
        print(f"Patched {c} product logos in chrome_100_percent.pak")

    # chrome_200_percent.pak: ID 15304 = 64x64, ID 15306 = 32x32
    p200 = os.path.join(runtime_dir, "chrome_200_percent.pak")
    if os.path.exists(p200):
        c = patch_pak_v5(p200, lambda rid, raw: png64 if rid == 15304 else (png32 if rid == 15306 else None))
        print(f"Patched {c} product logos in chrome_200_percent.pak")


def patch_chrome_dll(runtime_dir):
    dll_path = os.path.join(runtime_dir, "chrome.dll")
    if not os.path.exists(dll_path):
        print("chrome.dll not found")
        return

    with open(dll_path, "rb") as f:
        data = bytearray(f.read())

    patched_items = []

    # 1. Start page HTML (exact byte-length in-place replacement)
    start_marker = b"<!doctype html>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n<meta name=\"color-scheme\" content=\"light dark\">\n<title>ShardBrowser</title>"
    pos = data.find(start_marker)
    if pos != -1:
        end_marker = b"</html>"
        end_pos = data.find(end_marker, pos)
        if end_pos != -1:
            end_pos += len(end_marker)
            orig_len = end_pos - pos
            new_html = START_PAGE_TEMPLATE.encode("utf-8")
            if len(new_html) > orig_len:
                raise ValueError(f"Replacement HTML too long: {len(new_html)} > {orig_len}")
            pad_needed = orig_len - len(new_html)
            if pad_needed >= 7:
                # Insert HTML comment right before </body> so exact byte length matches
                comment = b"<!--" + (b" " * (pad_needed - 7)) + b"-->"
                new_html = new_html.replace(b"</body>", comment + b"</body>", 1)
            else:
                new_html = new_html + (b" " * pad_needed)
            assert len(new_html) == orig_len
            data[pos:end_pos] = new_html
            patched_items.append(f"start_page_html ({orig_len} bytes @0x{pos:x})")
    else:
        print("  (start_page_html already patched or not matched)")

    # 2. Ensure WebUI host right before style-src 'unsafe-inline'; stays "shardx"
    # so Chromium's sorted WebUI host table remains valid and chrome://shardx/ renders our LPBrower HTML.
    host_pat_old = b"\x00lphome\x00\x00\x00\x00\x00\x00style-src 'unsafe-inline';"
    hp = data.find(host_pat_old)
    if hp != -1:
        data[hp + 1 : hp + 7] = b"shardx"
        patched_items.append(f"webui_host restored->shardx @0x{hp+1:x}")

    # 3. Default startup URL "chrome://shardx/" (or lphome) -> "chrome://newtab/" (exact 16 bytes!)
    for old_url in (b"\x00chrome://shardx/\x00", b"\x00chrome://lphome/\x00"):
        up = data.find(old_url)
        if up != -1:
            data[up + 1 : up + 17] = b"chrome://newtab/"
            patched_items.append(f"startup_url->chrome://newtab/ @0x{up+1:x}")

    # 4. Window title prefix UTF-16 anchored right after `shardx-hide-profile-name` (exact 8 UTF-16 chars)
    #    so browser titles render cleanly as `Window (LPB|<Name>) — <Page> - LPBrower`.
    anchor = data.find(b"shardx-hide-profile-name\x00")
    if anchor != -1:
        window_slice_end = min(len(data), anchor + 128)
        for old_win_str in ("ShardX (", "LP App (", "LPBrower (", "Profile "):
            win_pat = old_win_str.encode("utf-16le")
            wp = data.find(win_pat, anchor, window_slice_end)
            if wp != -1:
                repl_win = "Window (".encode("utf-16le")
                data[wp : wp + len(repl_win)] = repl_win
                if len(win_pat) > len(repl_win):
                    data[wp + len(repl_win) : wp + len(win_pat)] = b"\x00" * (len(win_pat) - len(repl_win))
                patched_items.append(f"window_title_prefix->'Window (' @0x{wp:x}")
                break

    # 5. CLI --version product string "product-version\0version\0ShardX\0\0"
    ver_pat = b"product-version\x00version\x00ShardX\x00\x00"
    vp = data.find(ver_pat)
    if vp != -1:
        off = vp + len(b"product-version\x00version\x00")
        data[off : off + 6] = b"LPApp\x00"
        patched_items.append(f"cli_version_name @0x{off:x}")

    # 6. Windows startup error UTF-16 "cannot start ShardX." -> "can't open LPBrower."
    err_old = "cannot start ShardX.".encode("utf-16le")
    err_new = "can't open LPBrower.".encode("utf-16le")
    assert len(err_old) == len(err_new)
    ep = data.find(err_old)
    if ep != -1:
        data[ep : ep + len(err_old)] = err_new
        patched_items.append(f"startup_err_msg @0x{ep:x}")

    if patched_items:
        with open(dll_path, "wb") as f:
            f.write(data)
        print("Patched chrome.dll:", ", ".join(patched_items))
    else:
        print("chrome.dll: no changes needed (already branded)")


def migrate_existing_profiles():
    udd_root = os.path.expandvars(r"%APPDATA%\LPBrower-launcher\user-data")
    if not os.path.isdir(udd_root):
        return
    migrated = 0
    for prof in os.listdir(udd_root):
        prefs = os.path.join(udd_root, prof, "Default", "Preferences")
        if os.path.isfile(prefs):
            with open(prefs, "rb") as f:
                raw = f.read()
            if b"chrome://shardx/" in raw or b"chrome://lphome/" in raw:
                updated = raw.replace(b"chrome://shardx/", b"chrome://newtab/").replace(b"chrome://lphome/", b"chrome://newtab/")
                with open(prefs, "wb") as f:
                    f.write(updated)
                migrated += 1
    if migrated:
        print(f"Migrated {migrated} existing profile Preferences to chrome://newtab/")


if __name__ == "__main__":
    patch_locales(RUNTIME_DIR)
    patch_scale_paks(RUNTIME_DIR)
    patch_chrome_dll(RUNTIME_DIR)
    migrate_existing_profiles()
    marker = os.path.join(RUNTIME_DIR, ".lpbrower-branded-v4")
    with open(marker, "w") as f:
        f.write("v4\n")
