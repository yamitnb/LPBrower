//! Re-brands the downloaded Chromium runtime directory to LPBrower in-place:
//! - Patches `locales/*.pak` (Windows/Linux) and `*.lproj/locale.pak` (macOS) UI strings
//! - Patches `chrome_100_percent.pak` and `chrome_200_percent.pak` product logo PNGs
//! - Patches `chrome.dll` (Windows), `ShardX Framework` (macOS), or `chrome` (Linux) start page
//!   HTML, startup URL (`chrome://newtab/`), window title prefix, CLI version, and error text
//! - On Windows, replaces PE icons (`chrome.exe`, `notification_helper.exe`) with LPBrower's ICO
//! - On macOS, updates `Info.plist` display names, replaces `Resources/app.icns` with LPBrower's
//!   ICNS, clears quarantine xattrs, and re-signs `ShardX.app` ad-hoc so Apple Silicon runs cleanly.

use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

#[path = "ntp_template.rs"]
mod ntp_template;

const MARKER_FILE: &str = ".lpbrower-branded-v9";

#[cfg(windows)]
const ICO_BYTES: &[u8] = include_bytes!("../icons/browser-icon.ico");
#[cfg(any(target_os = "macos", test))]
const ICNS_BYTES: &[u8] = include_bytes!("../icons/browser-icon.icns");
const LP_PNG_16: &[u8] = include_bytes!("../../public/lp-browser-16.png");
const LP_PNG_32: &[u8] = include_bytes!("../../public/lp-browser-32.png");
const LP_PNG_64: &[u8] = include_bytes!("../../public/lp-browser-64.png");

const START_PAGE_HTML: &str = r#"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<meta name="color-scheme" content="light dark">
<title>LPBrower</title>
<style>
/* LPBrower start page. One file, no network requests. */
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
    --bg-page: #0c1427;
    --bg-tile-fill: #131e38cc;
    --bg-on-tile: #ffffff0f;
    --shadow-tile: 0 6px 20px -6px #04091473;
    --text-strong: #f8fafc;
    --text-sub: #cbd5e1;
    --text-soft: #8597b8;
    --icon-sub: #22d3ee;
    --primary-base: #22d3ee;
    --primary-text: #67e8f9;
    --primary-tint: #22d3ee24;
    --primary-tint-hover: #22d3ee38;
    --focus: #22d3ee;
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
      <a class="link" href="https://github.com/yamitnb/LPBrower">
        github.com/yamitnb/LPBrower
        <svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
          <path d="M9 6.65032C9 6.65032 15.9383 6.10759 16.9154 7.08463C17.8924 8.06167 17.3496 15 17.3496 15M16.5 7.5L6.5 17.5"/>
        </svg>
      </a>
    </footer>
  </div>
</main>
</body>
</html>"#;

/// Ensure the extracted engine folder is branded as LPBrower.
/// Idempotent and fast (skips immediately when `MARKER_FILE` exists).
/// Accepts either the engine root (`.../ShardX-Mac-arm64`, `.../ShardX-Windows`)
/// or `bin.parent()` (`.../ShardX.app/Contents/MacOS`).
pub fn ensure_branded(engine_dir: &Path) {
    if !engine_dir.is_dir() {
        return;
    }

    if let Some((marker_dir, app_bundle)) = resolve_macos_bundle(engine_dir) {
        let marker = marker_dir.join(MARKER_FILE);
        let fallback_marker = macos_fallback_marker(&app_bundle);
        if marker.exists() || fallback_marker.as_ref().is_some_and(|p| p.exists()) {
            return;
        }
        if brand_macos_bundle(&app_bundle) {
            if fs::write(&marker, "v9\n").is_err() {
                if let Some(fb) = fallback_marker {
                    if let Some(parent) = fb.parent() {
                        let _ = fs::create_dir_all(parent);
                    }
                    let _ = fs::write(&fb, "v9\n");
                }
            }
        }
        return;
    }

    let marker = engine_dir.join(MARKER_FILE);
    if marker.exists() {
        return;
    }

    let mut all_ok = true;
    patch_locales(engine_dir);
    patch_scale_paks(engine_dir);
    patch_resources_pak(engine_dir);

    let dll = engine_dir.join("chrome.dll");
    if dll.is_file() {
        if let Err(e) = patch_chrome_dll(&dll) {
            eprintln!("[brand_engine] chrome.dll patch skipped: {e}");
            all_ok = false;
        }
    } else {
        #[cfg(target_os = "linux")]
        {
            let linux_bin = engine_dir.join("chrome");
            if linux_bin.is_file() {
                if let Err(e) = patch_chrome_dll(&linux_bin) {
                    eprintln!("[brand_engine] linux chrome patch skipped: {e}");
                    all_ok = false;
                }
            }
        }
    }

    let nh = engine_dir.join("notification_helper.exe");
    if nh.is_file() {
        let _ = patch_notification_helper_strings(&nh);
    }

    #[cfg(windows)]
    {
        for exe_name in ["chrome.exe", "notification_helper.exe"] {
            let p = engine_dir.join(exe_name);
            if p.is_file() {
                if let Err(e) = win_res::update_exe_icon(&p, ICO_BYTES) {
                    eprintln!("[brand_engine] icon update failed for {exe_name}: {e}");
                    all_ok = false;
                }
            }
        }
        for pe_name in ["chrome.exe", "notification_helper.exe", "chrome_elf.dll", "chrome.dll"] {
            let p = engine_dir.join(pe_name);
            if p.is_file() {
                if let Err(e) = win_res::update_version_info(&p) {
                    eprintln!("[brand_engine] version info update skipped for {pe_name}: {e}");
                }
            }
        }
        win_res::scrub_windows_shell_cache();
    }

    if all_ok {
        let _ = fs::write(&marker, "v9\n");
    }
}

/// Locate `(marker_dir, app_bundle)` when `path` is either an engine root containing
/// `ShardX.app` or a subdirectory inside `*.app/Contents/...` (such as `bin.parent()`).
fn resolve_macos_bundle(path: &Path) -> Option<(PathBuf, PathBuf)> {
    let nested = path.join("ShardX.app");
    if nested.join("Contents").is_dir() {
        return Some((path.to_path_buf(), nested));
    }
    for anc in path.ancestors() {
        if anc.extension().and_then(|e| e.to_str()) == Some("app") && anc.join("Contents").is_dir() {
            let marker_dir = anc.parent().unwrap_or(anc).to_path_buf();
            return Some((marker_dir, anc.to_path_buf()));
        }
    }
    None
}

fn macos_fallback_marker(app_bundle: &Path) -> Option<PathBuf> {
    let mut h: u64 = 1469598103934665603;
    for b in app_bundle.to_string_lossy().as_bytes() {
        h ^= *b as u64;
        h = h.wrapping_mul(1099511628211);
    }
    dirs::cache_dir().map(|c| {
        c.join(crate::store::APP_DIR_NAME)
            .join(format!("branded-mac-{h:016x}.marker"))
    })
}

#[cfg(any(target_os = "macos", test))]
fn brand_macos_bundle(app_bundle: &Path) -> bool {
    let contents = app_bundle.join("Contents");
    if !contents.is_dir() {
        return false;
    }

    let mut all_ok = true;

    // 1. Update display names in Contents/Info.plist (preserving CFBundleExecutable).
    let info_plist = contents.join("Info.plist");
    if info_plist.is_file() {
        let _ = patch_plist_display_names(&info_plist, false);
    }

    // 2. Replace bundle .icns and localized InfoPlist.strings in Contents/Resources/.
    let res_dir = contents.join("Resources");
    if res_dir.is_dir() {
        replace_bundle_icns(&res_dir, &mut all_ok);
        let _ = fs::remove_file(res_dir.join("Assets.car"));
        patch_macos_infoplist_strings(&res_dir);
    }

    // 3. Patch Chromium Framework resources (*.lproj/locale.pak, scale paks, framework binary, helpers).
    let frameworks_dir = contents.join("Frameworks");
    if let Ok(fw_entries) = fs::read_dir(&frameworks_dir) {
        for fw_ent in fw_entries.flatten() {
            let fw_path = fw_ent.path();
            if fw_path.extension().and_then(|e| e.to_str()) != Some("framework") {
                continue;
            }
            let versions_dir = fw_path.join("Versions");
            let Ok(ver_entries) = fs::read_dir(&versions_dir) else {
                continue;
            };
            for ver_ent in ver_entries.flatten() {
                // Skip `Current` symlink so we never patch the same version directory twice.
                if ver_ent.file_type().map(|ft| ft.is_symlink()).unwrap_or(false) {
                    continue;
                }
                let ver_path = ver_ent.path();
                if !ver_path.is_dir() {
                    continue;
                }
                let fw_res = ver_path.join("Resources");
                if fw_res.is_dir() {
                    patch_scale_paks(&fw_res);
                    patch_resources_pak(&fw_res);
                    patch_locales(&fw_res);
                    patch_macos_lproj_locales(&fw_res);
                    patch_macos_infoplist_strings(&fw_res);
                    for p_name in ["Info.plist", "app_mode-Info.plist"] {
                        let p = fw_res.join(p_name);
                        if p.is_file() {
                            let _ = patch_plist_display_names(&p, false);
                        }
                    }
                }

                // Framework Mach-O binary (e.g. `ShardX Framework`).
                if let Some(stem) = fw_path.file_stem().and_then(|s| s.to_str()) {
                    let fw_bin = ver_path.join(stem);
                    if fw_bin.is_file() {
                        if let Err(e) = patch_chrome_dll(&fw_bin) {
                            eprintln!("[brand_engine] macOS framework patch skipped: {e}");
                            all_ok = false;
                        }
                    }
                }

                // Helper apps (`ShardX Helper (Alerts).app`, `ShardX Helper (GPU).app`, etc.).
                // Notification Center on macOS reads the sender name & icon directly from
                // `ShardX Helper (Alerts).app` and `ShardX Helper (Aperitif Alerts).app`!
                let helpers_dir = ver_path.join("Helpers");
                if let Ok(helpers) = fs::read_dir(&helpers_dir) {
                    for h_ent in helpers.flatten() {
                        let hp = h_ent.path();
                        if hp.extension().and_then(|e| e.to_str()) == Some("app") {
                            let is_alerts = hp
                                .file_name()
                                .and_then(|n| n.to_str())
                                .is_some_and(|n| n.contains("Alert"));
                            let h_plist = hp.join("Contents").join("Info.plist");
                            if h_plist.is_file() {
                                let _ = patch_plist_display_names(&h_plist, is_alerts);
                            }
                            let h_res = hp.join("Contents").join("Resources");
                            if h_res.is_dir() {
                                replace_bundle_icns(&h_res, &mut all_ok);
                                let _ = fs::remove_file(h_res.join("Assets.car"));
                                patch_macos_infoplist_strings(&h_res);
                            }
                        }
                    }
                }
            }
        }
    }

    // 4. Re-sign modified Mach-O and .app bundle on macOS so Apple Silicon amfid allows execution.
    #[cfg(target_os = "macos")]
    {
        if let Err(e) = resign_macos_bundle(app_bundle) {
            eprintln!("[brand_engine] macOS ad-hoc codesign failed: {e}");
            all_ok = false;
        }
    }

    all_ok
}

#[cfg(not(any(target_os = "macos", test)))]
fn brand_macos_bundle(_app_bundle: &Path) -> bool {
    false
}

#[cfg(any(target_os = "macos", test))]
fn has_any_icns(dir: &Path) -> bool {
    fs::read_dir(dir)
        .ok()
        .into_iter()
        .flatten()
        .flatten()
        .any(|e| e.path().extension().and_then(|x| x.to_str()) == Some("icns"))
}

#[cfg(any(target_os = "macos", test))]
fn patch_macos_lproj_locales(resources_dir: &Path) {
    let Ok(entries) = fs::read_dir(resources_dir) else {
        return;
    };
    for ent in entries.flatten() {
        let p = ent.path();
        if p.extension().and_then(|e| e.to_str()) != Some("lproj") || !p.is_dir() {
            continue;
        }
        let pak = p.join("locale.pak");
        if !pak.is_file() {
            continue;
        }
        let _ = patch_pak_v5(&pak, |_rid, raw| {
            if find_subslice(raw, b"Shard").is_none() {
                return None;
            }
            let r1 = replace_bytes_all(raw, b"ShardBrowser", b"LPBrower");
            let r2 = replace_bytes_all(&r1, b"ShardX", b"LPBrower");
            if r2 != raw { Some(r2) } else { None }
        });
    }
}

#[cfg(any(target_os = "macos", test))]
fn replace_bundle_icns(res_dir: &Path, all_ok: &mut bool) {
    let app_icns = res_dir.join("app.icns");
    if app_icns.is_file() || !has_any_icns(res_dir) {
        if let Err(e) = fs::write(&app_icns, ICNS_BYTES) {
            eprintln!("[brand_engine] macOS app.icns write failed: {e}");
            *all_ok = false;
        }
    } else if let Ok(entries) = fs::read_dir(res_dir) {
        for ent in entries.flatten() {
            let p = ent.path();
            if p.extension().and_then(|e| e.to_str()) == Some("icns") {
                if let Err(e) = fs::write(&p, ICNS_BYTES) {
                    eprintln!("[brand_engine] macOS icns update failed for {}: {e}", p.display());
                    *all_ok = false;
                }
            }
        }
    }
}

/// Patch localized `*.lproj/InfoPlist.strings` (both binary plists in `Alerts` helpers
/// and UTF-8/UTF-16 `.strings` in main app locales) so macOS Notification Center and
/// permission dialogs display `"LPBrower"` instead of `"ShardX"`.
#[cfg(any(target_os = "macos", test))]
fn patch_macos_infoplist_strings(resources_dir: &Path) {
    let Ok(entries) = fs::read_dir(resources_dir) else {
        return;
    };
    for ent in entries.flatten() {
        let p = ent.path();
        if p.extension().and_then(|e| e.to_str()) != Some("lproj") || !p.is_dir() {
            continue;
        }
        let strings_file = p.join("InfoPlist.strings");
        let Ok(raw) = fs::read(&strings_file) else {
            continue;
        };
        if raw.starts_with(b"bplist") {
            let clean = "\"CFBundleDisplayName\" = \"LPBrower\";\n\"CFBundleName\" = \"LPBrower\";\n";
            let _ = fs::write(&strings_file, clean.as_bytes());
        } else if let Ok(txt) = String::from_utf8(raw.clone()) {
            let updated = txt
                .replace("ShardBrowser", "LPBrower")
                .replace("ShardX", "LPBrower");
            if updated != txt {
                let _ = fs::write(&strings_file, updated.as_bytes());
            }
        }
    }
}

/// Update only human-facing display name keys in an Apple XML `Info.plist`,
/// leaving `CFBundleExecutable` and `CFBundleIdentifier` untouched, and removing
/// `CFBundleIconName` (`Assets.car`) so macOS uses `app.icns`.
#[cfg(any(target_os = "macos", test))]
fn patch_plist_display_names(plist_path: &Path, force_pure_brand_name: bool) -> std::io::Result<bool> {
    let raw = fs::read_to_string(plist_path)?;
    let mut cur = raw.clone();
    for key in [
        "CFBundleDisplayName",
        "CFBundleName",
        "CFBundleGetInfoString",
        "NSHumanReadableCopyright",
    ] {
        let needle = format!("<key>{key}</key>");
        let mut search_from = 0usize;
        while let Some(rel_k) = cur[search_from..].find(&needle) {
            let after_key = search_from + rel_k + needle.len();
            let rest = &cur[after_key..];
            let next_key = rest.find("<key>").unwrap_or(rest.len());
            let window = &rest[..next_key];
            if let (Some(s_open), Some(s_close)) = (window.find("<string>"), window.find("</string>")) {
                let val_start = after_key + s_open + "<string>".len();
                let val_end = after_key + s_close;
                if val_start <= val_end {
                    let updated_val = if force_pure_brand_name && (key == "CFBundleDisplayName" || key == "CFBundleName") {
                        "LPBrower".to_string()
                    } else {
                        cur[val_start..val_end]
                            .replace("ShardBrowser", "LPBrower")
                            .replace("ShardX", "LPBrower")
                    };
                    cur.replace_range(val_start..val_end, &updated_val);
                    search_from = val_start + updated_val.len();
                    continue;
                }
            }
            search_from = after_key;
        }
    }
    // Remove CFBundleIconName -> AppIcon so macOS uses our patched app.icns instead of Shard's Assets.car.
    let icon_key = "<key>CFBundleIconName</key>";
    if let Some(ik) = cur.find(icon_key) {
        let rest = &cur[ik + icon_key.len()..];
        if let Some(end_s) = rest.find("</string>") {
            let remove_end = ik + icon_key.len() + end_s + "</string>".len();
            cur.replace_range(ik..remove_end, "");
        }
    }
    if cur != raw {
        fs::write(plist_path, cur.as_bytes())?;
        return Ok(true);
    }
    Ok(false)
}

#[cfg(target_os = "macos")]
fn resign_macos_bundle(app_bundle: &Path) -> Result<(), String> {
    use std::process::Command;

    // Strip Gatekeeper quarantine xattrs before signing.
    let _ = Command::new("/usr/bin/xattr")
        .args(["-cr"])
        .arg(app_bundle)
        .output();

    let sign_one = |target: &Path, deep: bool| -> Result<(), String> {
        let mut cmd = Command::new("/usr/bin/codesign");
        cmd.arg("--force");
        if deep {
            cmd.arg("--deep");
        }
        let out = cmd
            .args(["--sign", "-"])
            .arg(target)
            .output()
            .map_err(|e| format!("codesign spawn failed for {}: {e}", target.display()))?;
        if !out.status.success() {
            return Err(format!(
                "codesign failed for {}: {}",
                target.display(),
                String::from_utf8_lossy(&out.stderr).trim()
            ));
        }
        Ok(())
    };

    // Sign nested Mach-Os inside Frameworks from deepest to shallowest first.
    let frameworks_dir = app_bundle.join("Contents").join("Frameworks");
    if let Ok(fw_entries) = fs::read_dir(&frameworks_dir) {
        for fw_ent in fw_entries.flatten() {
            let fw_path = fw_ent.path();
            if fw_path.extension().and_then(|e| e.to_str()) != Some("framework") {
                continue;
            }
            let versions_dir = fw_path.join("Versions");
            if let Ok(ver_entries) = fs::read_dir(&versions_dir) {
                for ver_ent in ver_entries.flatten() {
                    if ver_ent.file_type().map(|ft| ft.is_symlink()).unwrap_or(false) {
                        continue;
                    }
                    let ver_path = ver_ent.path();
                    if !ver_path.is_dir() {
                        continue;
                    }
                    let libs_dir = ver_path.join("Libraries");
                    if let Ok(libs) = fs::read_dir(&libs_dir) {
                        for l_ent in libs.flatten() {
                            let lp = l_ent.path();
                            if lp.extension().and_then(|e| e.to_str()) == Some("dylib") {
                                let _ = sign_one(&lp, false);
                            }
                        }
                    }
                    let helpers_dir = ver_path.join("Helpers");
                    if let Ok(helpers) = fs::read_dir(&helpers_dir) {
                        for h_ent in helpers.flatten() {
                            let hp = h_ent.path();
                            if hp.extension().and_then(|e| e.to_str()) == Some("app") {
                                let _ = sign_one(&hp, true);
                            } else if hp.is_file() {
                                let _ = sign_one(&hp, false);
                            }
                        }
                    }
                }
            }
            let _ = sign_one(&fw_path, false);
        }
    }

    // Finally sign the outer .app bundle.
    sign_one(app_bundle, true)
}

/// Migrate any existing profile's `Default/Preferences` that still has `chrome://shardx/`
/// as its startup URL to `chrome://newtab/`.
pub fn migrate_profile_prefs(udd: &Path) {
    let prefs = udd.join("Default").join("Preferences");
    if let Ok(raw) = fs::read(&prefs) {
        if find_subslice(&raw, b"chrome://shardx/").is_some()
            || find_subslice(&raw, b"chrome://lphome/").is_some()
        {
            let s = String::from_utf8_lossy(&raw);
            let updated = s
                .replace("chrome://shardx/", "chrome://newtab/")
                .replace("chrome://lphome/", "chrome://newtab/");
            let _ = fs::write(&prefs, updated.as_bytes());
        }
    }
}

fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || needle.len() > haystack.len() {
        return None;
    }
    haystack.windows(needle.len()).position(|w| w == needle)
}

fn replace_bytes_all(data: &[u8], from: &[u8], to: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(data.len());
    let mut i = 0;
    while i < data.len() {
        if i + from.len() <= data.len() && &data[i..i + from.len()] == from {
            out.extend_from_slice(to);
            i += from.len();
        } else {
            out.push(data[i]);
            i += 1;
        }
    }
    out
}

fn patch_pak_v5<F>(path: &Path, mut transform: F) -> std::io::Result<usize>
where
    F: FnMut(u16, &[u8]) -> Option<Vec<u8>>,
{
    let data = fs::read(path)?;
    if data.len() < 12 {
        return Ok(0);
    }
    let version = u32::from_le_bytes(data[0..4].try_into().unwrap());
    if version != 5 {
        return Ok(0);
    }
    let encoding = data[4];
    let num_entries = u16::from_le_bytes(data[8..10].try_into().unwrap()) as usize;
    let num_aliases = u16::from_le_bytes(data[10..12].try_into().unwrap()) as usize;

    let mut off = 12usize;
    let mut raw_entries: Vec<(u16, usize)> = Vec::with_capacity(num_entries + 1);
    for _ in 0..=num_entries {
        if off + 6 > data.len() {
            return Ok(0);
        }
        let rid = u16::from_le_bytes(data[off..off + 2].try_into().unwrap());
        let pos = u32::from_le_bytes(data[off + 2..off + 6].try_into().unwrap()) as usize;
        raw_entries.push((rid, pos));
        off += 6;
    }
    let mut aliases: Vec<(u16, u16)> = Vec::with_capacity(num_aliases);
    for _ in 0..num_aliases {
        if off + 4 > data.len() {
            return Ok(0);
        }
        let rid = u16::from_le_bytes(data[off..off + 2].try_into().unwrap());
        let idx = u16::from_le_bytes(data[off + 2..off + 4].try_into().unwrap());
        aliases.push((rid, idx));
        off += 4;
    }

    let mut changed = 0usize;
    let mut payloads: Vec<(u16, Vec<u8>)> = Vec::with_capacity(num_entries);
    for i in 0..num_entries {
        let (rid, start) = raw_entries[i];
        let end = raw_entries[i + 1].1;
        if start > end || end > data.len() {
            return Ok(0);
        }
        let slice = &data[start..end];
        if let Some(updated) = transform(rid, slice) {
            if updated != slice {
                changed += 1;
                payloads.push((rid, updated));
                continue;
            }
        }
        payloads.push((rid, slice.to_vec()));
    }

    if changed == 0 {
        return Ok(0);
    }

    let header_size = 12 + (num_entries + 1) * 6 + num_aliases * 4;
    let mut cur_off = header_size as u32;
    let mut out = Vec::with_capacity(data.len() + 4096);
    out.extend_from_slice(&5u32.to_le_bytes());
    out.push(encoding);
    out.extend_from_slice(&[0u8; 3]);
    out.extend_from_slice(&(num_entries as u16).to_le_bytes());
    out.extend_from_slice(&(num_aliases as u16).to_le_bytes());

    for (rid, payload) in &payloads {
        out.extend_from_slice(&rid.to_le_bytes());
        out.extend_from_slice(&cur_off.to_le_bytes());
        cur_off = cur_off.wrapping_add(payload.len() as u32);
    }
    out.extend_from_slice(&0u16.to_le_bytes());
    out.extend_from_slice(&cur_off.to_le_bytes());

    for (rid, idx) in &aliases {
        out.extend_from_slice(&rid.to_le_bytes());
        out.extend_from_slice(&idx.to_le_bytes());
    }
    for (_, payload) in &payloads {
        out.extend_from_slice(payload);
    }

    let tmp = path.with_extension("pak.lptmp");
    fs::write(&tmp, &out)?;
    fs::rename(&tmp, path)?;
    Ok(changed)
}

fn patch_locales(engine_dir: &Path) {
    let locales_dir = engine_dir.join("locales");
    let Ok(entries) = fs::read_dir(&locales_dir) else {
        return;
    };
    for ent in entries.flatten() {
        let p = ent.path();
        if p.extension().and_then(|e| e.to_str()) != Some("pak") {
            continue;
        }
        let _ = patch_pak_v5(&p, |_rid, raw| {
            if find_subslice(raw, b"Shard").is_none() {
                return None;
            }
            let r1 = replace_bytes_all(raw, b"ShardBrowser", b"LPBrower");
            let r2 = replace_bytes_all(&r1, b"ShardX", b"LPBrower");
            if r2 != raw { Some(r2) } else { None }
        });
    }
}

fn patch_scale_paks(engine_dir: &Path) {
    let p100 = engine_dir.join("chrome_100_percent.pak");
    if p100.is_file() {
        let _ = patch_pak_v5(&p100, |rid, _| match rid {
            // 697 = IDR_NTP_FAVICON (chrome://newtab tab icon),
            // 15306 / 41292 = product logo 16px / chrome:// WebUI favicons,
            // 15304 = product logo 32px.
            697 | 15306 | 41292 => Some(LP_PNG_16.to_vec()),
            15304 => Some(LP_PNG_32.to_vec()),
            _ => None,
        });
    }
    let p200 = engine_dir.join("chrome_200_percent.pak");
    if p200.is_file() {
        let _ = patch_pak_v5(&p200, |rid, _| match rid {
            697 | 15306 | 41292 => Some(LP_PNG_32.to_vec()),
            15304 => Some(LP_PNG_64.to_vec()),
            _ => None,
        });
    }
}

fn gzip_bytes(raw: &[u8]) -> Option<Vec<u8>> {
    let mut enc = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast());
    enc.write_all(raw).ok()?;
    enc.finish().ok()
}

fn gunzip_bytes(raw: &[u8]) -> Option<Vec<u8>> {
    if raw.len() < 2 || raw[0] != 0x1f || raw[1] != 0x8b {
        return None;
    }
    let mut dec = flate2::read::GzDecoder::new(raw);
    let mut out = Vec::new();
    dec.read_to_end(&mut out).ok()?;
    Some(out)
}

/// Patch Chromium's `resources.pak` so `chrome://newtab/` (both `chrome://new-tab-page/`
/// and `chrome://new-tab-page-third-party/`) renders the branded LPBrowser Blank Tab.
/// Matches entries by decompressed content signature so Windows, macOS, and Linux work
/// identically even when platform resource IDs shift.
fn patch_resources_pak(res_dir: &Path) {
    let pak = res_dir.join("resources.pak");
    if !pak.is_file() {
        return;
    }
    let Some(main_html_gz) = gzip_bytes(
        ntp_template::build_ntp_html("new_tab_page.js", "LPBROWER_NTP_MAIN_HTML").as_bytes(),
    ) else {
        return;
    };
    let Some(main_js_gz) = gzip_bytes(ntp_template::build_ntp_js("LPBROWER_NTP_MAIN_JS").as_bytes()) else {
        return;
    };
    let Some(tp_html_gz) = gzip_bytes(
        ntp_template::build_ntp_html("new_tab_page_third_party.js", "LPBROWER_NTP_3P_HTML").as_bytes(),
    ) else {
        return;
    };
    let Some(tp_js_gz) = gzip_bytes(ntp_template::build_ntp_js("LPBROWER_NTP_3P_JS").as_bytes()) else {
        return;
    };

    let _ = patch_pak_v5(&pak, |_rid, raw| {
        if raw.len() < 10 || raw.len() > 160_000 || raw[0] != 0x1f || raw[1] != 0x8b {
            return None;
        }
        let dec = gunzip_bytes(raw)?;
        if find_subslice(&dec, b"<ntp-app></ntp-app>").is_some()
            || find_subslice(&dec, b"LPBROWER_NTP_MAIN_HTML").is_some()
        {
            return Some(main_html_gz.clone());
        }
        if find_subslice(&dec, b"<cr-most-visited></cr-most-visited>").is_some()
            || find_subslice(&dec, b"LPBROWER_NTP_3P_HTML").is_some()
        {
            return Some(tp_html_gz.clone());
        }
        if find_subslice(&dec, b"recordSparseValueWithPersistentHash,recordVoiceAction").is_some()
            || find_subslice(&dec, b"LPBROWER_NTP_MAIN_JS").is_some()
        {
            return Some(main_js_gz.clone());
        }
        if find_subslice(&dec, b"mostVisitedElement.theme=theme.mostVisited").is_some()
            || find_subslice(&dec, b"LPBROWER_NTP_3P_JS").is_some()
        {
            return Some(tp_js_gz.clone());
        }
        None
    });
}

fn utf16le(s: &str) -> Vec<u8> {
    s.encode_utf16().flat_map(|c| c.to_le_bytes()).collect()
}

fn patch_chrome_dll(dll_path: &Path) -> std::io::Result<()> {
    let mut data = fs::read(dll_path)?;
    let mut modified = false;

    // 1. Start page HTML (exact byte length; matches fresh ShardBrowser or previously branded LPBrower)
    let start_marker_shard = b"<!doctype html>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n<meta name=\"color-scheme\" content=\"light dark\">\n<title>ShardBrowser</title>";
    let start_marker_lp = b"<!doctype html>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n<meta name=\"color-scheme\" content=\"light dark\">\n<title>LPBrower</title>";
    if let Some(pos) = find_subslice(&data, start_marker_shard).or_else(|| find_subslice(&data, start_marker_lp)) {
        let end_marker = b"</html>";
        if let Some(rel_end) = find_subslice(&data[pos..], end_marker) {
            let end_pos = pos + rel_end + end_marker.len();
            let orig_len = end_pos - pos;
            let base_html = START_PAGE_HTML.as_bytes();
            if base_html.len() <= orig_len {
                let pad_needed = orig_len - base_html.len();
                let mut new_html = Vec::with_capacity(orig_len);
                if pad_needed >= 7 {
                    let body_close = b"</body>";
                    if let Some(bpos) = find_subslice(base_html, body_close) {
                        new_html.extend_from_slice(&base_html[..bpos]);
                        new_html.extend_from_slice(b"<!--");
                        new_html.extend(std::iter::repeat(b' ').take(pad_needed - 7));
                        new_html.extend_from_slice(b"-->");
                        new_html.extend_from_slice(&base_html[bpos..]);
                    }
                }
                if new_html.len() != orig_len {
                    new_html.clear();
                    new_html.extend_from_slice(base_html);
                    new_html.extend(std::iter::repeat(b' ').take(pad_needed));
                }
                if new_html.len() == orig_len {
                    data[pos..end_pos].copy_from_slice(&new_html);
                    modified = true;
                }
            }
        }
    }

    // 2. Startup URL: chrome://shardx/ -> chrome://newtab/ (exact 16 bytes)
    if let Some(up) = find_subslice(&data, b"\0chrome://shardx/\0") {
        data[up + 1..up + 17].copy_from_slice(b"chrome://newtab/");
        modified = true;
    }

    // 3. Window title prefix UTF-16 (`u"ShardX ("` -> `u"Window ("`, exact 8 UTF-16 chars)
    //    so titles render: `Window (LPB|<Name>) — <Page> - LPBrower`.
    //    Check anchored near `shardx-hide-profile-name` first (PE `.rdata`), then fallback
    //    to direct UTF-16LE match for Mach-O (`__TEXT,__ustring` pooled separately on macOS).
    let mut patched_win_prefix = false;
    let win_new = utf16le("Window (");
    if let Some(anchor) = find_subslice(&data, b"shardx-hide-profile-name\0") {
        let end = (anchor + 128).min(data.len());
        for old_s in ["ShardX (", "LP App (", "Profile "] {
            let win_old = utf16le(old_s);
            if let Some(rel) = find_subslice(&data[anchor..end], &win_old) {
                let wp = anchor + rel;
                data[wp..wp + win_new.len()].copy_from_slice(&win_new);
                modified = true;
                patched_win_prefix = true;
                break;
            }
        }
    }
    if !patched_win_prefix {
        for old_s in ["ShardX (", "LP App ("] {
            let win_old = utf16le(old_s);
            if let Some(wp) = find_subslice(&data, &win_old) {
                data[wp..wp + win_new.len()].copy_from_slice(&win_new);
                modified = true;
                break;
            }
        }
    }

    // 4. CLI --version name
    let ver_pat = b"product-version\0version\0ShardX\0\0";
    if let Some(vp) = find_subslice(&data, ver_pat) {
        let off = vp + b"product-version\0version\0".len();
        data[off..off + 6].copy_from_slice(b"LPApp\0");
        modified = true;
    }

    // 5. Startup error UTF-16: "cannot start ShardX." -> "can't open LPBrower."
    let err_old = utf16le("cannot start ShardX.");
    let err_new = utf16le("can't open LPBrower.");
    if err_old.len() == err_new.len() {
        if let Some(ep) = find_subslice(&data, &err_old) {
            data[ep..ep + err_new.len()].copy_from_slice(&err_new);
            modified = true;
        }
    }

    if modified {
        fs::write(dll_path, &data)?;
    }
    Ok(())
}

fn patch_notification_helper_strings(nh_path: &Path) -> std::io::Result<()> {
    let mut data = fs::read(nh_path)?;
    let needle = b"notification-helper\0ShardX\0";
    if let Some(pos) = find_subslice(&data, needle) {
        let off = pos + b"notification-helper\0".len();
        data[off..off + 7].copy_from_slice(b"LPApp\0\0");
        fs::write(nh_path, &data)?;
    }
    Ok(())
}

#[cfg(windows)]
mod win_res {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use std::path::Path;
    use windows_sys::Win32::System::LibraryLoader::{
        BeginUpdateResourceW, EndUpdateResourceW, UpdateResourceW,
    };

    const RT_ICON: usize = 3;
    const RT_GROUP_ICON: usize = 14;

    pub fn update_exe_icon(exe_path: &Path, ico_data: &[u8]) -> Result<(), String> {
        if ico_data.len() < 6 {
            return Err("ICO too short".into());
        }
        let ico_type = u16::from_le_bytes(ico_data[2..4].try_into().unwrap());
        let count = u16::from_le_bytes(ico_data[4..6].try_into().unwrap()) as usize;
        if ico_type != 1 || count == 0 || ico_data.len() < 6 + count * 16 {
            return Err("Invalid ICO header".into());
        }

        let mut grp_dir: Vec<u8> = Vec::with_capacity(6 + count * 14);
        grp_dir.extend_from_slice(&0u16.to_le_bytes());
        grp_dir.extend_from_slice(&1u16.to_le_bytes());
        grp_dir.extend_from_slice(&(count as u16).to_le_bytes());

        let mut images: Vec<(u16, &[u8])> = Vec::with_capacity(count);
        for i in 0..count {
            let base = 6 + i * 16;
            let size = u32::from_le_bytes(ico_data[base + 8..base + 12].try_into().unwrap()) as usize;
            let img_off = u32::from_le_bytes(ico_data[base + 12..base + 16].try_into().unwrap()) as usize;
            if img_off + size > ico_data.len() {
                return Err("ICO entry out of bounds".into());
            }
            let id = (i + 1) as u16;
            grp_dir.extend_from_slice(&ico_data[base..base + 12]);
            grp_dir.extend_from_slice(&id.to_le_bytes());
            images.push((id, &ico_data[img_off..img_off + size]));
        }

        let wide_path: Vec<u16> = OsStr::new(exe_path)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        unsafe {
            let h = BeginUpdateResourceW(wide_path.as_ptr(), 0);
            if h.is_null() {
                return Err("BeginUpdateResourceW failed".into());
            }

            for (id, bytes) in &images {
                for lang in [0u16, 1033u16] {
                    UpdateResourceW(
                        h,
                        RT_ICON as *const u16,
                        *id as usize as *const u16,
                        lang,
                        bytes.as_ptr() as *const _,
                        bytes.len() as u32,
                    );
                }
            }

            for group_id in [1usize, 101usize] {
                for lang in [0u16, 1033u16] {
                    UpdateResourceW(
                        h,
                        RT_GROUP_ICON as *const u16,
                        group_id as *const u16,
                        lang,
                        grp_dir.as_ptr() as *const _,
                        grp_dir.len() as u32,
                    );
                }
            }

            if EndUpdateResourceW(h, 0) == 0 {
                return Err("EndUpdateResourceW failed".into());
            }
        }
        Ok(())
    }

    const RT_VERSION: usize = 16;
    const LOAD_LIBRARY_AS_DATAFILE: u32 = 0x0000_0002;

    pub fn update_version_info(pe_path: &Path) -> Result<(), String> {
        use windows_sys::Win32::System::LibraryLoader::{
            FindResourceExW, LoadLibraryExW, LoadResource, LockResource, SizeofResource,
        };
        #[link(name = "kernel32")]
        extern "system" {
            fn FreeLibrary(h_lib_module: *mut std::ffi::c_void) -> i32;
        }

        let wide_path: Vec<u16> = OsStr::new(pe_path)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        let mut patches: Vec<(u16, Vec<u8>)> = Vec::new();
        unsafe {
            let h_mod = LoadLibraryExW(wide_path.as_ptr(), std::ptr::null_mut(), LOAD_LIBRARY_AS_DATAFILE);
            if h_mod.is_null() {
                return Err("LoadLibraryExW failed".into());
            }
            for lang in [1033u16, 0u16] {
                let h_res = FindResourceExW(
                    h_mod,
                    RT_VERSION as *const u16,
                    1usize as *const u16,
                    lang,
                );
                if !h_res.is_null() {
                    let sz = SizeofResource(h_mod, h_res) as usize;
                    let h_glob = LoadResource(h_mod, h_res);
                    let ptr = LockResource(h_glob) as *const u8;
                    if !ptr.is_null() && sz >= 6 {
                        let raw = std::slice::from_raw_parts(ptr, sz);
                        if let Some((mut root, _)) = super::ver_info::parse_node(raw, 0) {
                            super::ver_info::rebrand_node(&mut root);
                            patches.push((lang, super::ver_info::serialize_node(&root)));
                        }
                    }
                    break;
                }
            }
            FreeLibrary(h_mod);

            if patches.is_empty() {
                return Ok(());
            }

            let h_upd = BeginUpdateResourceW(wide_path.as_ptr(), 0);
            if h_upd.is_null() {
                return Err("BeginUpdateResourceW(RT_VERSION) failed".into());
            }
            for (lang, bytes) in &patches {
                UpdateResourceW(
                    h_upd,
                    RT_VERSION as *const u16,
                    1usize as *const u16,
                    *lang,
                    bytes.as_ptr() as *const _,
                    bytes.len() as u32,
                );
            }
            if EndUpdateResourceW(h_upd, 0) == 0 {
                return Err("EndUpdateResourceW(RT_VERSION) failed".into());
            }
        }
        Ok(())
    }

    /// Update any stale Windows Shell `MuiCache` or `AppUserModelId` display names
    /// that cached `"ShardX"` / `"ShardBrowser"` from an earlier run.
    pub fn scrub_windows_shell_cache() {
        use std::process::Command;
        use std::os::windows::process::CommandExt;

        let ps = r#"
$ErrorActionPreference = 'SilentlyContinue'
$paths = @(
  'HKCU:\Software\Classes\Local Settings\Software\Microsoft\Windows\Shell\MuiCache',
  'HKCU:\Software\Classes\AppUserModelId',
  'HKCU:\Software\Microsoft\Windows\CurrentVersion\Notifications\Settings'
)
foreach ($base in $paths) {
  if (-not (Test-Path $base)) { continue }
  $item = Get-Item $base
  foreach ($vn in $item.GetValueNames()) {
    $val = [string]$item.GetValue($vn)
    if ($val -match 'ShardX|ShardBrowser|Wyocore') {
      Set-ItemProperty -Path $base -Name $vn -Value 'LPBrower'
    }
  }
  Get-ChildItem $base | ForEach-Object {
    $sub = $_
    foreach ($vn in $sub.GetValueNames()) {
      $val = [string]$sub.GetValue($vn)
      if ($val -match 'ShardX|ShardBrowser|Shard') {
        Set-ItemProperty -Path $sub.PSPath -Name $vn -Value 'LPBrower'
      }
    }
  }
}
"#;
        let _ = Command::new("powershell.exe")
            .args(["-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-Command", ps])
            .creation_flags(0x08000000)
            .output();
    }
}

#[cfg(any(windows, test))]
mod ver_info {
    #[derive(Clone, Debug)]
    pub struct VerNode {
        pub key: String,
        pub val_type: u16,
        pub value: Vec<u8>,
        pub children: Vec<VerNode>,
    }

    const fn align4(n: usize) -> usize {
        (n + 3) & !3
    }

    pub fn parse_node(data: &[u8], mut offset: usize) -> Option<(VerNode, usize)> {
        if offset + 6 > data.len() {
            return None;
        }
        let start = offset;
        let w_len = u16::from_le_bytes(data[offset..offset + 2].try_into().ok()?) as usize;
        let w_val_len = u16::from_le_bytes(data[offset + 2..offset + 4].try_into().ok()?) as usize;
        let w_type = u16::from_le_bytes(data[offset + 4..offset + 6].try_into().ok()?);
        if w_len < 6 || start + w_len > data.len() {
            return None;
        }
        let end = start + w_len;
        offset += 6;

        let mut key_u16 = Vec::new();
        while offset + 2 <= end {
            let ch = u16::from_le_bytes(data[offset..offset + 2].try_into().ok()?);
            offset += 2;
            if ch == 0 {
                break;
            }
            key_u16.push(ch);
        }
        let key = String::from_utf16_lossy(&key_u16);
        offset = align4(offset);

        let mut value = Vec::new();
        if w_val_len > 0 {
            let byte_len = if w_type == 1 { w_val_len * 2 } else { w_val_len };
            if offset + byte_len > end {
                return None;
            }
            value.extend_from_slice(&data[offset..offset + byte_len]);
            offset = align4(offset + byte_len);
        }

        let mut children = Vec::new();
        while offset + 6 <= end {
            let (child, next_off) = parse_node(data, offset)?;
            children.push(child);
            offset = align4(next_off);
        }

        Some((
            VerNode {
                key,
                val_type: w_type,
                value,
                children,
            },
            end,
        ))
    }

    pub fn serialize_node(node: &VerNode) -> Vec<u8> {
        let mut key_bytes: Vec<u8> = node
            .key
            .encode_utf16()
            .chain(std::iter::once(0))
            .flat_map(|c| c.to_le_bytes())
            .collect();
        let w_val_len = if node.val_type == 1 {
            (node.value.len() / 2) as u16
        } else {
            node.value.len() as u16
        };

        let mut children_bytes = Vec::new();
        for (idx, child) in node.children.iter().enumerate() {
            if idx > 0 {
                let pad = align4(children_bytes.len()) - children_bytes.len();
                children_bytes.resize(children_bytes.len() + pad, 0);
            }
            children_bytes.extend_from_slice(&serialize_node(child));
        }

        let header_len = 6 + key_bytes.len();
        let aligned_header = align4(header_len);
        key_bytes.resize(key_bytes.len() + (aligned_header - header_len), 0);

        let val_len = node.value.len();
        let child_pad = if val_len > 0 && !children_bytes.is_empty() {
            align4(val_len) - val_len
        } else {
            0
        };

        let total_len = aligned_header + val_len + child_pad + children_bytes.len();
        let mut out = Vec::with_capacity(total_len);
        out.extend_from_slice(&(total_len as u16).to_le_bytes());
        out.extend_from_slice(&w_val_len.to_le_bytes());
        out.extend_from_slice(&node.val_type.to_le_bytes());
        out.extend_from_slice(&key_bytes);
        out.extend_from_slice(&node.value);
        out.resize(out.len() + child_pad, 0);
        out.extend_from_slice(&children_bytes);
        out
    }

    fn utf16z(s: &str) -> Vec<u8> {
        s.encode_utf16()
            .chain(std::iter::once(0))
            .flat_map(|c| c.to_le_bytes())
            .collect()
    }

    pub fn rebrand_node(node: &mut VerNode) {
        if node.val_type == 1 && node.value.len() >= 2 {
            let new_str = match node.key.as_str() {
                "ProductName" | "ProductShortName" | "FileDescription" | "CompanyName" | "CompanyShortName" => {
                    Some("LPBrower".to_string())
                }
                "LegalCopyright" => Some("Copyright 2026 LPBrower. All rights reserved.".to_string()),
                _ => {
                    let u16s: Vec<u16> = node
                        .value
                        .chunks_exact(2)
                        .map(|c| u16::from_le_bytes([c[0], c[1]]))
                        .take_while(|&c| c != 0)
                        .collect();
                    let s = String::from_utf16_lossy(&u16s);
                    if s.contains("Shard") || s.contains("Wyocore") {
                        Some(
                            s.replace("ShardBrowser", "LPBrower")
                                .replace("ShardX", "LPBrower")
                                .replace("Wyocore Tecnologiens LLC", "LPBrower"),
                        )
                    } else {
                        None
                    }
                }
            };
            if let Some(rep) = new_str {
                node.value = utf16z(&rep);
            }
        }
        for child in &mut node.children {
            rebrand_node(child);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_pak_v5(entries: &[(u16, &[u8])]) -> Vec<u8> {
        let num_entries = entries.len();
        let header_size = 12 + (num_entries + 1) * 6;
        let mut cur_off = header_size as u32;
        let mut out = Vec::new();
        out.extend_from_slice(&5u32.to_le_bytes());
        out.push(1u8); // UTF-8 encoding
        out.extend_from_slice(&[0u8; 3]);
        out.extend_from_slice(&(num_entries as u16).to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes());
        for (rid, payload) in entries {
            out.extend_from_slice(&rid.to_le_bytes());
            out.extend_from_slice(&cur_off.to_le_bytes());
            cur_off += payload.len() as u32;
        }
        out.extend_from_slice(&0u16.to_le_bytes());
        out.extend_from_slice(&cur_off.to_le_bytes());
        for (_, payload) in entries {
            out.extend_from_slice(payload);
        }
        out
    }

    #[test]
    fn macos_bundle_branding_patches_plist_icns_locales_and_framework() {
        let tmp = std::env::temp_dir().join(format!(
            "lpbrower-mac-brand-test-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let engine_root = tmp.join("ShardX-Mac-arm64");
        let app = engine_root.join("ShardX.app");
        let contents = app.join("Contents");
        let macos_dir = contents.join("MacOS");
        let res_dir = contents.join("Resources");
        let fw_ver = contents
            .join("Frameworks")
            .join("ShardX Framework.framework")
            .join("Versions")
            .join("157.0.8103.50");
        let fw_lproj = fw_ver.join("Resources").join("en.lproj");
        let alerts_app = fw_ver.join("Helpers").join("ShardX Helper (Alerts).app");
        let alerts_res = alerts_app.join("Contents").join("Resources");
        let alerts_base_lproj = alerts_res.join("base.lproj");
        let outer_vi_lproj = res_dir.join("vi.lproj");

        fs::create_dir_all(&macos_dir).unwrap();
        fs::create_dir_all(&outer_vi_lproj).unwrap();
        fs::create_dir_all(&fw_lproj).unwrap();
        fs::create_dir_all(&alerts_base_lproj).unwrap();

        let plist = r#"<?xml version="1.0" encoding="UTF-8"?>
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>ShardX</string>
    <key>CFBundleName</key>
    <string>ShardX</string>
    <key>CFBundleDisplayName</key>
    <string>ShardBrowser</string>
    <key>CFBundleIconName</key>
    <string>AppIcon</string>
</dict>
</plist>"#;
        fs::write(contents.join("Info.plist"), plist).unwrap();
        fs::write(res_dir.join("app.icns"), b"old-icns").unwrap();
        fs::write(res_dir.join("Assets.car"), b"old-car").unwrap();
        fs::write(
            outer_vi_lproj.join("InfoPlist.strings"),
            "CFBundleGetInfoString = \"ShardX 158, Tác giả ShardX\";\n",
        )
        .unwrap();
        fs::write(macos_dir.join("ShardX"), b"stub").unwrap();

        let alerts_plist = r#"<?xml version="1.0" encoding="UTF-8"?>
<plist version="1.0">
<dict>
    <key>CFBundleDisplayName</key>
    <string>ShardX Helper (Alerts)</string>
    <key>CFBundleExecutable</key>
    <string>ShardX Helper (Alerts)</string>
    <key>CFBundleIconFile</key>
    <string>app.icns</string>
    <key>CFBundleIconName</key>
    <string>AppIcon</string>
</dict>
</plist>"#;
        fs::write(alerts_app.join("Contents").join("Info.plist"), alerts_plist).unwrap();
        fs::write(alerts_res.join("app.icns"), b"old-alert-icns").unwrap();
        fs::write(alerts_res.join("Assets.car"), b"old-alert-car").unwrap();
        fs::write(alerts_base_lproj.join("InfoPlist.strings"), b"bplist00\xd1\x01\x02ShardX").unwrap();

        let pak = make_pak_v5(&[(101, b"About ShardBrowser and ShardX")]);
        fs::write(fw_lproj.join("locale.pak"), &pak).unwrap();

        let fake_ntp_html = gzip_bytes(b"<!doctype html><body><ntp-app></ntp-app><script type=\"module\" src=\"new_tab_page.js\"></script></body>").unwrap();
        let fake_ntp_js = gzip_bytes(b"export {recordSparseValueWithPersistentHash,recordVoiceAction};").unwrap();
        let res_pak = make_pak_v5(&[(693, &fake_ntp_html), (698, &fake_ntp_js)]);
        fs::write(fw_ver.join("Resources").join("resources.pak"), &res_pak).unwrap();

        let mut fake_fw = Vec::new();
        fake_fw.extend_from_slice(b"prefix\0chrome://shardx/\0middle");
        fake_fw.extend_from_slice(&utf16le("ShardX ("));
        fake_fw.extend_from_slice(b"suffix");
        fs::write(fw_ver.join("ShardX Framework"), &fake_fw).unwrap();

        // Calling with `bin.parent()` (`.../Contents/MacOS`) must locate `ShardX.app` and `engine_root`.
        ensure_branded(&macos_dir);

        let updated_plist = fs::read_to_string(contents.join("Info.plist")).unwrap();
        assert!(updated_plist.contains("<key>CFBundleExecutable</key>\n    <string>ShardX</string>"));
        assert!(updated_plist.contains("<key>CFBundleName</key>\n    <string>LPBrower</string>"));
        assert!(updated_plist.contains("<key>CFBundleDisplayName</key>\n    <string>LPBrower</string>"));
        assert!(!updated_plist.contains("CFBundleIconName"));
        assert!(!res_dir.join("Assets.car").exists());

        let vi_strings = fs::read_to_string(outer_vi_lproj.join("InfoPlist.strings")).unwrap();
        assert!(!vi_strings.contains("Shard"), "{vi_strings}");
        assert!(vi_strings.contains("LPBrower"));

        let updated_alerts_plist = fs::read_to_string(alerts_app.join("Contents").join("Info.plist")).unwrap();
        assert!(updated_alerts_plist.contains("<key>CFBundleDisplayName</key>\n    <string>LPBrower</string>"));
        assert!(updated_alerts_plist.contains("<key>CFBundleExecutable</key>\n    <string>ShardX Helper (Alerts)</string>"));
        assert!(!updated_alerts_plist.contains("CFBundleIconName"));
        assert!(!alerts_res.join("Assets.car").exists());
        assert_eq!(&fs::read(alerts_res.join("app.icns")).unwrap()[..4], b"icns");
        let alerts_strings = fs::read_to_string(alerts_base_lproj.join("InfoPlist.strings")).unwrap();
        assert!(alerts_strings.contains("\"CFBundleDisplayName\" = \"LPBrower\""));
        assert!(!alerts_strings.contains("Shard"));

        let updated_icns = fs::read(res_dir.join("app.icns")).unwrap();
        assert_eq!(&updated_icns[..4], b"icns");
        assert_eq!(updated_icns.len(), ICNS_BYTES.len());

        let updated_pak = fs::read(fw_lproj.join("locale.pak")).unwrap();
        assert!(find_subslice(&updated_pak, b"About LPBrower and LPBrower").is_some());
        assert!(find_subslice(&updated_pak, b"Shard").is_none());

        let updated_res_pak = fs::read(fw_ver.join("Resources").join("resources.pak")).unwrap();
        assert!(updated_res_pak.len() > res_pak.len());
        assert!(START_PAGE_HTML.len() <= 11000, "START_PAGE_HTML too long: {}", START_PAGE_HTML.len());
        assert!(!ntp_template::build_ntp_html("new_tab_page.js", "M").contains('$'));
        assert!(!ntp_template::build_ntp_js("M").contains('$'));

        let updated_fw = fs::read(fw_ver.join("ShardX Framework")).unwrap();
        assert!(find_subslice(&updated_fw, b"\0chrome://newtab/\0").is_some());
        assert!(find_subslice(&updated_fw, &utf16le("Window (")).is_some());

        assert!(engine_root.join(MARKER_FILE).exists());

        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn ver_info_rebrands_product_and_description_cleanly() {
        let root = ver_info::VerNode {
            key: "VS_VERSION_INFO".into(),
            val_type: 0,
            value: vec![0xbd, 0x04, 0xef, 0xfe],
            children: vec![ver_info::VerNode {
                key: "StringFileInfo".into(),
                val_type: 1,
                value: Vec::new(),
                children: vec![ver_info::VerNode {
                    key: "040904b0".into(),
                    val_type: 1,
                    value: Vec::new(),
                    children: vec![
                        ver_info::VerNode {
                            key: "ProductName".into(),
                            val_type: 1,
                            value: "ShardX\0".encode_utf16().flat_map(|c| c.to_le_bytes()).collect(),
                            children: Vec::new(),
                        },
                        ver_info::VerNode {
                            key: "FileDescription".into(),
                            val_type: 1,
                            value: "ShardX\0".encode_utf16().flat_map(|c| c.to_le_bytes()).collect(),
                            children: Vec::new(),
                        },
                    ],
                }],
            }],
        };
        let raw = ver_info::serialize_node(&root);
        let (mut parsed, _) = ver_info::parse_node(&raw, 0).unwrap();
        ver_info::rebrand_node(&mut parsed);
        let raw2 = ver_info::serialize_node(&parsed);
        let (round, _) = ver_info::parse_node(&raw2, 0).unwrap();
        let props = &round.children[0].children[0].children;
        let val_of = |n: &ver_info::VerNode| -> String {
            let u16s: Vec<u16> = n.value.chunks_exact(2).map(|c| u16::from_le_bytes([c[0], c[1]])).take_while(|&c| c != 0).collect();
            String::from_utf16_lossy(&u16s)
        };
        assert_eq!(val_of(&props[0]), "LPBrower");
        assert_eq!(val_of(&props[1]), "LPBrower");
    }

    #[cfg(windows)]
    #[test]
    fn live_pe_update_version_info_rebrands_chrome_elf() {
        let Some(appdata) = std::env::var_os("APPDATA") else { return };
        let rt_dir = PathBuf::from(appdata)
            .join("LPBrower-launcher")
            .join("runtime")
            .join("ShardX-Windows");
        let src_elf = rt_dir.join("chrome_elf.dll");
        if !src_elf.is_file() {
            return;
        }
        let tmp_elf = std::env::temp_dir().join(format!("lpb-elf-test-{}.dll", std::process::id()));
        fs::copy(&src_elf, &tmp_elf).unwrap();
        win_res::update_version_info(&tmp_elf).unwrap();

        let data = fs::read(&tmp_elf).unwrap();
        let _ = fs::remove_file(&tmp_elf);
        let shardx_u16 = utf16le("ShardX");
        assert!(
            find_subslice(&data, &shardx_u16).is_none(),
            "patched chrome_elf.dll still contains UTF-16 ShardX"
        );
        assert!(find_subslice(&data, &utf16le("LPBrower")).is_some());

        // Best-effort upgrade of the local runtime when no browser process locks it.
        ensure_branded(&rt_dir);
    }
}
