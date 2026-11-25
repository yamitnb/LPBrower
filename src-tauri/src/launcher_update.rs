//! Self-updater for LPBrower Launcher (`yamitnb/LPBrower` GitHub Releases).
//!
//! Supports in-place updates for:
//! - Windows portable `LPBrower.exe` (cross-drive stage + atomic same-volume swap + relaunch)
//! - Windows installed builds via NSIS `LPBrower_Setup_x64.exe` or `.msi`
//! - macOS `.app` bundle via `LPBrower_Launcher_macOS_arm64.app.tar.gz` (extract + swap + relaunch)
//! - Graceful fallback (`installable: false` + `release_url`) for debug builds or non-writable installs.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tauri::{AppHandle, Emitter};
use tokio::io::AsyncWriteExt;

const RELEASE_REPO: &str = "yamitnb/LPBrower";
const RELEASE_API_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);
const MIN_BINARY_SIZE: u64 = 1024 * 1024; // 1 MiB sanity floor for release assets

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InstallKind {
    PortableExe,
    WindowsInstaller,
    MacosApp,
    Manual,
}

impl InstallKind {
    pub fn as_str(self) -> &'static str {
        match self {
            InstallKind::PortableExe => "portable_exe",
            InstallKind::WindowsInstaller => "windows_installer",
            InstallKind::MacosApp => "macos_app",
            InstallKind::Manual => "manual",
        }
    }

    pub fn is_installable(self) -> bool {
        !matches!(self, InstallKind::Manual)
    }
}

/// What the UI receives from `launcher_update_offer`.
#[derive(Serialize, Clone, Debug)]
pub struct LauncherOffer {
    pub current: String,
    pub version: String,
    pub installable: bool,
    pub install_kind: String,
    pub asset_name: Option<String>,
    pub asset_size: Option<u64>,
    pub release_notes: Option<String>,
    pub release_url: Option<String>,
    pub published_at: Option<String>,
}

#[derive(Clone, Debug)]
struct ReleaseAsset {
    name: String,
    download_url: String,
    /// GitHub API asset endpoint (`.../releases/assets/<id>`), required for private repo downloads.
    api_url: Option<String>,
    size: u64,
    digest_sha256: Option<String>,
}

#[derive(Clone, Debug)]
struct FoundUpdate {
    offer: LauncherOffer,
    kind: InstallKind,
    download_url: Option<String>,
    api_url: Option<String>,
    auth_token: Option<String>,
    digest_sha256: Option<String>,
}

#[derive(Clone, Debug)]
struct DownloadedUpdate {
    version: String,
    kind: InstallKind,
    path: PathBuf,
}

static FOUND: Mutex<Option<FoundUpdate>> = Mutex::new(None);
static DOWNLOADED: Mutex<Option<DownloadedUpdate>> = Mutex::new(None);

#[derive(Serialize, Clone)]
struct Progress {
    received: u64,
    total: Option<u64>,
    percent: u64,
}

fn norm_ver(v: &str) -> &str {
    v.trim().strip_prefix('v').unwrap_or(v.trim())
}

fn split_ver(s: &str) -> Vec<&str> {
    norm_ver(s)
        .split(['-', '+'])
        .next()
        .unwrap_or("")
        .split('.')
        .collect()
}

/// SemVer-aware numeric comparison (returns true if `latest` > `current`).
pub(crate) fn is_newer(latest: &str, current: &str) -> bool {
    let a = split_ver(latest);
    let b = split_ver(current);
    for i in 0..a.len().max(b.len()) {
        let x = a.get(i).copied().unwrap_or("0");
        let y = b.get(i).copied().unwrap_or("0");
        match (x.parse::<u64>(), y.parse::<u64>()) {
            (Ok(xn), Ok(yn)) => {
                if xn != yn {
                    return xn > yn;
                }
            }
            _ => {
                if x != y {
                    return x > y;
                }
            }
        }
    }
    false
}

/// Directory under app data where downloaded launcher updates are staged.
fn updates_dir() -> Result<PathBuf, String> {
    let parent = dirs::data_dir().ok_or("platform data dir not available")?;
    #[cfg(not(test))]
    crate::store::migrate_legacy_dir(&parent);
    Ok(parent.join(crate::store::APP_DIR_NAME).join("updates"))
}

/// Check whether a directory can be written to by creating and deleting a probe file.
fn dir_is_writable(dir: &Path) -> bool {
    let probe = dir.join(format!(".lpb-write-probe-{}", std::process::id()));
    match fs::OpenOptions::new().write(true).create_new(true).open(&probe) {
        Ok(_) => {
            let _ = fs::remove_file(&probe);
            true
        }
        Err(_) => false,
    }
}

/// Detect whether the running Windows binary is an installed copy (NSIS/MSI)
/// rather than a standalone portable `LPBrower.exe`.
/// Note: `tauri build` patches the release `.exe` in place with `bundle_type`
/// when producing `.msi`/`.exe` installers, so we inspect the actual install
/// directory (`uninstall.exe`, Program Files, or `%LOCALAPPDATA%\LPBrower Launcher`)
/// instead of trusting `bundle_type()` alone.
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
fn detect_windows_installed(exe_path: &Path) -> bool {
    if let Some(dir) = exe_path.parent() {
        if dir.join("uninstall.exe").is_file() {
            return true;
        }
        let lower = dir.to_string_lossy().to_ascii_lowercase().replace('/', "\\");
        if lower.contains("\\program files\\")
            || lower.contains("\\program files (x86)\\")
            || lower.ends_with("\\appdata\\local\\lpbrower launcher")
            || lower.ends_with("\\appdata\\local\\lpbrowser")
        {
            return true;
        }
    }
    false
}

/// Find the enclosing `.app` bundle on macOS from `current_exe()`.
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
fn enclosing_macos_app(exe_path: &Path) -> Option<PathBuf> {
    let mut cur = exe_path.parent();
    while let Some(p) = cur {
        if p.extension().and_then(|e| e.to_str()) == Some("app") {
            return Some(p.to_path_buf());
        }
        cur = p.parent();
    }
    None
}

/// Pick the best release asset and installation strategy for the current host.
fn select_asset<'a>(
    assets: &'a [ReleaseAsset],
    os: &str,
    arch: &str,
    prefer_installer: bool,
    target_writable: bool,
    allow_self_install: bool,
) -> (InstallKind, Option<&'a ReleaseAsset>) {
    let find_ci = |pred: &dyn Fn(&str) -> bool| -> Option<&'a ReleaseAsset> {
        assets.iter().find(|a| pred(&a.name.to_ascii_lowercase()))
    };

    if !allow_self_install {
        // Still pick a sensible asset for display/size info, but mark Manual.
        let picked = match os {
            "windows" => find_ci(&|n| n == "lpbrower.exe" || n.contains("setup") && n.ends_with(".exe")),
            "macos" => find_ci(&|n| n.ends_with(".dmg") || n.ends_with(".app.tar.gz")),
            _ => None,
        };
        return (InstallKind::Manual, picked);
    }

    match os {
        "windows" => {
            let setup_exe = find_ci(&|n| n.ends_with(".exe") && n.contains("setup"));
            let msi = find_ci(&|n| n.ends_with(".msi"));
            let portable = find_ci(&|n| n == "lpbrower.exe" || (n.contains("portable") && n.ends_with(".exe")));

            if prefer_installer || !target_writable {
                if let Some(a) = setup_exe.or(msi) {
                    return (InstallKind::WindowsInstaller, Some(a));
                }
                if let Some(a) = portable {
                    return (InstallKind::Manual, Some(a));
                }
                return (InstallKind::Manual, None);
            }

            if let Some(a) = portable {
                return (InstallKind::PortableExe, Some(a));
            }
            if let Some(a) = setup_exe.or(msi) {
                return (InstallKind::WindowsInstaller, Some(a));
            }
            (InstallKind::Manual, None)
        }
        "macos" => {
            let arch_tag = if arch == "aarch64" { "arm64" } else { arch };
            let app_tar = find_ci(&|n| n.ends_with(".app.tar.gz") && (n.contains(arch_tag) || n.contains("universal")))
                .or_else(|| find_ci(&|n| n.ends_with(".app.tar.gz")));
            let dmg = find_ci(&|n| n.ends_with(".dmg") && (n.contains(arch_tag) || n.contains("universal")))
                .or_else(|| find_ci(&|n| n.ends_with(".dmg")));

            if target_writable {
                if let Some(a) = app_tar {
                    return (InstallKind::MacosApp, Some(a));
                }
            }
            (InstallKind::Manual, dmg.or(app_tar))
        }
        _ => (InstallKind::Manual, None),
    }
}

/// Remove `.old` leftover binary after a successful portable self-update, and
/// sweep stale cached update folders for versions <= current.
pub fn cleanup_at_start(current_version: &str) {
    if let Ok(exe) = std::env::current_exe() {
        #[cfg(windows)]
        {
            let old = exe.with_extension("exe.old");
            if old.exists() {
                // The previous process may take a few hundred ms to exit completely.
                for _ in 0..10 {
                    if fs::remove_file(&old).is_ok() || !old.exists() {
                        break;
                    }
                    std::thread::sleep(std::time::Duration::from_millis(250));
                }
            }
            let staged = exe.with_extension("exe.new");
            if staged.exists() {
                let _ = fs::remove_file(&staged);
            }
        }
        #[cfg(target_os = "macos")]
        if let Some(app_bundle) = enclosing_macos_app(&exe) {
            let old = app_bundle.with_extension("app.old");
            if old.exists() {
                let _ = fs::remove_dir_all(&old);
            }
        }
    }

    if let Ok(dir) = updates_dir() {
        if let Ok(entries) = fs::read_dir(&dir) {
            for ent in entries.flatten() {
                let name = ent.file_name().to_string_lossy().to_string();
                if !is_newer(&name, current_version) {
                    let p = ent.path();
                    if p.is_dir() {
                        let _ = fs::remove_dir_all(&p);
                    } else {
                        let _ = fs::remove_file(&p);
                    }
                }
            }
        }
    }
}

fn resolve_update_config() -> (String, Option<String>) {
    let settings = crate::settings::load().ok();
    let url = std::env::var("LPBROWER_UPDATE_URL")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .or_else(|| {
            settings
                .as_ref()
                .and_then(|s| s.update_url.as_deref())
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(String::from)
        })
        .unwrap_or_else(|| format!("https://api.github.com/repos/{RELEASE_REPO}/releases/latest"));

    let token = std::env::var("LPBROWER_UPDATE_TOKEN")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .or_else(|| {
            settings
                .as_ref()
                .and_then(|s| s.update_token.as_deref())
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(String::from)
        });
    (url, token)
}

/// Parses either a GitHub Releases API payload (`assets[{browser_download_url, url, ...}]`)
/// or a static `latest.json` manifest (`assets[{name, url, size, sha256}]`).
fn parse_release_assets(body: &serde_json::Value) -> Vec<ReleaseAsset> {
    body.get("assets")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|item| {
                    let name = item.get("name")?.as_str()?.to_string();
                    let download_url = item
                        .get("browser_download_url")
                        .or_else(|| item.get("url"))
                        .and_then(|v| v.as_str())?
                        .to_string();
                    let api_url = item
                        .get("browser_download_url")
                        .and_then(|_| item.get("url"))
                        .and_then(|v| v.as_str())
                        .filter(|u| u.contains("/releases/assets/"))
                        .map(String::from);
                    let size = item.get("size").and_then(|s| s.as_u64()).unwrap_or(0);
                    let digest_sha256 = item
                        .get("digest")
                        .and_then(|d| d.as_str())
                        .and_then(|s| s.strip_prefix("sha256:"))
                        .or_else(|| item.get("sha256").and_then(|s| s.as_str()))
                        .map(|h| h.trim().to_ascii_lowercase())
                        .filter(|h| h.len() == 64);
                    Some(ReleaseAsset {
                        name,
                        download_url,
                        api_url,
                        size,
                        digest_sha256,
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Fallback for public GitHub repos when `api.github.com` returns 403/429 rate-limit:
/// follow `https://github.com/<repo>/releases/latest` redirect to extract the latest tag
/// and build direct release download URLs without consuming API quota.
async fn fetch_via_github_redirect(
    client: &reqwest::Client,
) -> Option<(String, String, Vec<ReleaseAsset>)> {
    let latest_web = format!("https://github.com/{RELEASE_REPO}/releases/latest");
    let resp = client
        .get(&latest_web)
        .timeout(RELEASE_API_TIMEOUT)
        .send()
        .await
        .ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let final_url = resp.url().as_str().to_string();
    let tag = final_url.rsplit("/tag/").next()?.trim().to_string();
    if tag.is_empty() || tag.contains('/') || tag == "latest" {
        return None;
    }
    let base = format!("https://github.com/{RELEASE_REPO}/releases/download/{tag}");
    let known = [
        "LPBrower.exe",
        "LPBrower_Setup_x64.exe",
        "LPBrower_Setup_x64.msi",
        "LPBrower_Launcher_macOS_arm64.app.tar.gz",
        "LPBrower_Setup_macOS_arm64.dmg",
    ];
    let assets = known
        .iter()
        .map(|name| ReleaseAsset {
            name: (*name).to_string(),
            download_url: format!("{base}/{name}"),
            api_url: None,
            size: 0,
            digest_sha256: None,
        })
        .collect();
    Some((tag, final_url, assets))
}

/// Query configured update feed (`latest.json` or GitHub Releases API) for a newer launcher.
/// Returns `Ok(Some(offer))` if a newer version exists, `Ok(None)` if up to date,
/// or `Err(...)` if the server is unreachable or inaccessible.
#[tauri::command]
pub async fn launcher_update_offer(app: AppHandle) -> Result<Option<LauncherOffer>, String> {
    let current = app.package_info().version.to_string();
    let (url, token) = resolve_update_config();
    let client = reqwest::Client::builder()
        .user_agent(format!("LPBrower-launcher/{current}"))
        .build()
        .map_err(|e| format!("Could not build HTTP client: {e}"))?;

    let mut req = client
        .get(&url)
        .header("Accept", "application/vnd.github+json, application/json")
        .timeout(RELEASE_API_TIMEOUT);
    if let Some(ref tok) = token {
        req = req.header("Authorization", format!("Bearer {tok}"));
    }

    let resp = req
        .send()
        .await
        .map_err(|e| format!("Could not reach update server: {e}"))?;

    let (latest_ver, release_url, release_notes, published_at, assets) = if resp.status().is_success() {
        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("Invalid update response: {e}"))?;
        let Some(tag) = body
            .get("tag_name")
            .or_else(|| body.get("version"))
            .and_then(|v| v.as_str())
        else {
            return Ok(None);
        };
        let ver = norm_ver(tag).to_string();
        let rel_url = body
            .get("html_url")
            .or_else(|| body.get("release_url"))
            .and_then(|v| v.as_str())
            .map(String::from)
            .or_else(|| Some(format!("https://github.com/{RELEASE_REPO}/releases/latest")));
        let notes = body
            .get("body")
            .or_else(|| body.get("notes"))
            .and_then(|v| v.as_str())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        let pub_at = body
            .get("published_at")
            .and_then(|v| v.as_str())
            .map(String::from);
        let assets = parse_release_assets(&body);
        (ver, rel_url, notes, pub_at, assets)
    } else if matches!(resp.status().as_u16(), 403 | 429) && token.is_none() && url.contains("api.github.com") {
        // Rate-limited on public GitHub API -> fall back to web redirect cleanly.
        let Some((tag, rel_url, assets)) = fetch_via_github_redirect(&client).await else {
            return Err(format!("Update server returned HTTP {}", resp.status().as_u16()));
        };
        (norm_ver(&tag).to_string(), Some(rel_url), None, None, assets)
    } else {
        return Err(format!("Update server returned HTTP {}", resp.status().as_u16()));
    };

    if !is_newer(&latest_ver, &current) {
        *FOUND.lock().unwrap_or_else(|e| e.into_inner()) = None;
        return Ok(None);
    }

    let allow_self_install = !cfg!(debug_assertions);
    let exe = std::env::current_exe().ok();

    let (prefer_installer, target_writable) = {
        #[cfg(target_os = "windows")]
        {
            let installed = exe
                .as_deref()
                .map(detect_windows_installed)
                .unwrap_or(true);
            let writable = exe
                .as_deref()
                .and_then(|p| p.parent())
                .map(dir_is_writable)
                .unwrap_or(false);
            (installed, writable)
        }
        #[cfg(target_os = "macos")]
        {
            let writable = exe
                .as_deref()
                .and_then(enclosing_macos_app)
                .and_then(|app_dir| app_dir.parent().map(dir_is_writable))
                .unwrap_or(false);
            (false, writable)
        }
        #[cfg(not(any(target_os = "windows", target_os = "macos")))]
        {
            let _ = &exe;
            (false, false)
        }
    };

    let (kind, asset) = select_asset(
        &assets,
        std::env::consts::OS,
        std::env::consts::ARCH,
        prefer_installer,
        target_writable,
        allow_self_install,
    );

    let offer = LauncherOffer {
        current,
        version: latest_ver,
        installable: kind.is_installable(),
        install_kind: kind.as_str().to_string(),
        asset_name: asset.map(|a| a.name.clone()),
        asset_size: asset.map(|a| a.size).filter(|&s| s > 0),
        release_notes,
        release_url,
        published_at,
    };

    *FOUND.lock().unwrap_or_else(|e| e.into_inner()) = Some(FoundUpdate {
        offer: offer.clone(),
        kind,
        download_url: asset.map(|a| a.download_url.clone()),
        api_url: asset.and_then(|a| a.api_url.clone()),
        auth_token: token,
        digest_sha256: asset.and_then(|a| a.digest_sha256.clone()),
    });

    Ok(Some(offer))
}

fn verify_file_integrity(path: &Path, kind: InstallKind, expected_sha256: Option<&str>) -> Result<(), String> {
    let meta = fs::metadata(path).map_err(|e| format!("Could not read downloaded file: {e}"))?;
    if meta.len() < MIN_BINARY_SIZE {
        return Err(format!(
            "Downloaded file is unexpectedly small ({} bytes)",
            meta.len()
        ));
    }

    let mut f = fs::File::open(path).map_err(|e| format!("Could not open downloaded file: {e}"))?;
    let mut head = [0u8; 4];
    f.read_exact(&mut head)
        .map_err(|e| format!("Could not read downloaded file header: {e}"))?;

    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    match kind {
        InstallKind::PortableExe => {
            if &head[..2] != b"MZ" {
                return Err("Downloaded executable is not a valid Windows binary".into());
            }
        }
        InstallKind::WindowsInstaller => {
            if ext == "msi" {
                if head != [0xD0, 0xCF, 0x11, 0xE0] {
                    return Err("Downloaded installer is not a valid MSI package".into());
                }
            } else if &head[..2] != b"MZ" {
                return Err("Downloaded installer is not a valid Windows executable".into());
            }
        }
        InstallKind::MacosApp => {
            if &head[..2] != [0x1F, 0x8B] {
                return Err("Downloaded macOS update is not a valid .tar.gz archive".into());
            }
        }
        InstallKind::Manual => {}
    }

    if let Some(want) = expected_sha256 {
        let mut f = fs::File::open(path).map_err(|e| format!("Could not hash downloaded file: {e}"))?;
        let mut hasher = Sha256::new();
        let mut buf = [0u8; 64 * 1024];
        loop {
            let n = f.read(&mut buf).map_err(|e| format!("Read failed during SHA-256 check: {e}"))?;
            if n == 0 {
                break;
            }
            hasher.update(&buf[..n]);
        }
        let got = format!("{:x}", hasher.finalize());
        if got != want {
            return Err(format!(
                "SHA-256 checksum mismatch (expected {want}, got {got})"
            ));
        }
    }
    Ok(())
}

/// Download the asset for the previously discovered update into app data,
/// emitting `launcher-update://progress` events along the way.
#[tauri::command]
pub async fn launcher_update_download(app: AppHandle) -> Result<String, String> {
    let found = FOUND
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone()
        .ok_or("No update was found to download")?;

    if !found.offer.installable {
        return Err("This build cannot install updates automatically; download from the release page.".into());
    }
    let download_url = found
        .download_url
        .clone()
        .ok_or("No downloadable release asset found for this platform")?;
    let asset_name = found
        .offer
        .asset_name
        .clone()
        .ok_or("Release asset name missing")?;

    // Reuse an already-downloaded and verified file for this exact version.
    if let Some(prev) = DOWNLOADED.lock().unwrap_or_else(|e| e.into_inner()).as_ref() {
        if prev.version == found.offer.version && prev.path.is_file() {
            if verify_file_integrity(&prev.path, prev.kind, found.digest_sha256.as_deref()).is_ok() {
                return Ok(found.offer.version);
            }
        }
    }

    let stage_dir = updates_dir()?.join(&found.offer.version);
    fs::create_dir_all(&stage_dir).map_err(|e| format!("Could not create updates folder: {e}"))?;
    let final_path = stage_dir.join(&asset_name);
    let part_path = stage_dir.join(format!("{asset_name}.part"));

    // If a valid file is already sitting on disk from an earlier interrupted apply, reuse it.
    if final_path.is_file()
        && verify_file_integrity(&final_path, found.kind, found.digest_sha256.as_deref()).is_ok()
    {
        *DOWNLOADED.lock().unwrap_or_else(|e| e.into_inner()) = Some(DownloadedUpdate {
            version: found.offer.version.clone(),
            kind: found.kind,
            path: final_path,
        });
        let _ = app.emit(
            "launcher-update://progress",
            Progress {
                received: found.offer.asset_size.unwrap_or(0),
                total: found.offer.asset_size,
                percent: 100,
            },
        );
        return Ok(found.offer.version);
    }

    let _ = fs::remove_file(&part_path);

    let client = reqwest::Client::builder()
        .user_agent(format!("LPBrower-launcher/{}", found.offer.current))
        .build()
        .map_err(|e| format!("Could not build HTTP client: {e}"))?;

    // For private GitHub repos, download via the API asset URL with `Accept: application/octet-stream`
    // and Bearer auth (reqwest automatically strips Authorization on cross-host blob redirects).
    let mut req = if let (Some(ref tok), Some(ref api_u)) = (&found.auth_token, &found.api_url) {
        client
            .get(api_u)
            .header("Authorization", format!("Bearer {tok}"))
            .header("Accept", "application/octet-stream")
    } else if let Some(ref tok) = found.auth_token {
        client
            .get(&download_url)
            .header("Authorization", format!("Bearer {tok}"))
    } else {
        client.get(&download_url)
    };
    req = req.header("User-Agent", format!("LPBrower-launcher/{}", found.offer.current));

    let mut resp = req
        .send()
        .await
        .map_err(|e| format!("Download failed: {e}"))?
        .error_for_status()
        .map_err(|e| format!("Download failed: {e}"))?;

    let total = resp.content_length().or(found.offer.asset_size);
    let mut out = tokio::fs::File::create(&part_path)
        .await
        .map_err(|e| format!("Could not create download file: {e}"))?;

    let mut received: u64 = 0;
    let mut last_pct: u64 = u64::MAX;
    let mut last_emitted_bytes: u64 = 0;

    while let Some(chunk) = resp
        .chunk()
        .await
        .map_err(|e| format!("Download interrupted: {e}"))?
    {
        out.write_all(&chunk)
            .await
            .map_err(|e| format!("Could not write download file: {e}"))?;
        received += chunk.len() as u64;
        let pct = match total {
            Some(t) if t > 0 => (received * 100 / t).min(99),
            _ => 0,
        };
        if pct != last_pct || received.saturating_sub(last_emitted_bytes) >= 256 * 1024 {
            last_pct = pct;
            last_emitted_bytes = received;
            let _ = app.emit(
                "launcher-update://progress",
                Progress {
                    received,
                    total,
                    percent: pct,
                },
            );
        }
    }
    out.flush().await.map_err(|e| format!("Flush failed: {e}"))?;
    out.sync_all().await.map_err(|e| format!("Sync failed: {e}"))?;
    drop(out);

    if let Some(t) = total {
        if t > 0 && received != t {
            let _ = fs::remove_file(&part_path);
            return Err(format!("Download ended early ({received} of {t} bytes)"));
        }
    }

    let check_path = part_path.clone();
    let kind = found.kind;
    let digest = found.digest_sha256.clone();
    tokio::task::spawn_blocking(move || verify_file_integrity(&check_path, kind, digest.as_deref()))
        .await
        .map_err(|e| format!("Verification task failed: {e}"))?
        .inspect_err(|_| {
            let _ = fs::remove_file(&part_path);
        })?;

    if final_path.exists() {
        let _ = fs::remove_file(&final_path);
    }
    fs::rename(&part_path, &final_path).map_err(|e| format!("Could not finalize downloaded file: {e}"))?;

    let _ = app.emit(
        "launcher-update://progress",
        Progress {
            received,
            total: total.or(Some(received)),
            percent: 100,
        },
    );

    *DOWNLOADED.lock().unwrap_or_else(|e| e.into_inner()) = Some(DownloadedUpdate {
        version: found.offer.version.clone(),
        kind: found.kind,
        path: final_path,
    });

    Ok(found.offer.version)
}

/// Swap a portable Windows executable cleanly even when `downloaded_exe` is on
/// a different drive letter (e.g. `C:\Users\...\updates` -> `G:\...\LPBrower.exe`).
///
/// 1. Copy `downloaded_exe` -> `<target_dir>/<exe>.new` (proves writability + space first)
/// 2. Rename running `<current_exe>` -> `<current_exe>.old` (same-volume atomic rename)
/// 3. Rename `<exe>.new` -> `<current_exe>` (same-volume atomic rename; rolls back on error)
#[cfg_attr(not(any(target_os = "windows", test)), allow(dead_code))]
fn swap_portable_exe(current_exe: &Path, downloaded_exe: &Path) -> Result<PathBuf, String> {
    let expected_len = fs::metadata(downloaded_exe)
        .map_err(|e| format!("Downloaded binary missing: {e}"))?
        .len();
    let staged_same_vol = current_exe.with_extension("exe.new");
    let backup_exe = current_exe.with_extension("exe.old");

    let _ = fs::remove_file(&staged_same_vol);
    let copied = fs::copy(downloaded_exe, &staged_same_vol)
        .map_err(|e| format!("Could not stage new executable beside LPBrower: {e}"))?;
    if copied != expected_len {
        let _ = fs::remove_file(&staged_same_vol);
        return Err(format!(
            "Staged executable size mismatch ({copied} vs {expected_len} bytes)"
        ));
    }

    if backup_exe.exists() {
        let _ = fs::remove_file(&backup_exe);
    }
    if let Err(e) = fs::rename(current_exe, &backup_exe) {
        let _ = fs::remove_file(&staged_same_vol);
        return Err(format!("Could not move current executable aside: {e}"));
    }

    if let Err(e) = fs::rename(&staged_same_vol, current_exe) {
        // Roll back immediately so the user never loses their launcher.
        let _ = fs::rename(&backup_exe, current_exe);
        let _ = fs::remove_file(&staged_same_vol);
        return Err(format!("Could not activate new executable: {e}"));
    }

    Ok(backup_exe)
}

#[cfg(target_os = "macos")]
fn apply_macos_app_bundle(current_exe: &Path, archive_path: &Path) -> Result<PathBuf, String> {
    let current_app = enclosing_macos_app(current_exe)
        .ok_or("Could not locate running .app bundle")?;
    let parent = current_app
        .parent()
        .ok_or("Could not locate .app parent folder")?;
    let extract_dir = archive_path
        .parent()
        .unwrap_or(parent)
        .join("extracted");
    let _ = fs::remove_dir_all(&extract_dir);
    fs::create_dir_all(&extract_dir).map_err(|e| format!("Could not create extract dir: {e}"))?;

    let status = std::process::Command::new("tar")
        .args(["-xzf"])
        .arg(archive_path)
        .arg("-C")
        .arg(&extract_dir)
        .status()
        .map_err(|e| format!("Could not run tar: {e}"))?;
    if !status.success() {
        return Err("Failed to extract macOS update archive".into());
    }

    let extracted_app = fs::read_dir(&extract_dir)
        .map_err(|e| format!("Could not read extracted folder: {e}"))?
        .flatten()
        .map(|e| e.path())
        .find(|p| p.extension().and_then(|e| e.to_str()) == Some("app"))
        .ok_or("Extracted archive did not contain a .app bundle")?;

    let _ = std::process::Command::new("xattr")
        .args(["-cr"])
        .arg(&extracted_app)
        .status();

    let staged_app = parent.join(".LPBrower-update.app");
    let backup_app = current_app.with_extension("app.old");
    let _ = fs::remove_dir_all(&staged_app);
    let _ = fs::remove_dir_all(&backup_app);

    // Move or copy into the same volume as /Applications.
    if fs::rename(&extracted_app, &staged_app).is_err() {
        let cp = std::process::Command::new("ditto")
            .arg(&extracted_app)
            .arg(&staged_app)
            .status()
            .map_err(|e| format!("Could not copy new .app bundle: {e}"))?;
        if !cp.success() {
            return Err("Could not copy new .app bundle into Applications".into());
        }
    }

    if let Err(e) = fs::rename(&current_app, &backup_app) {
        let _ = fs::remove_dir_all(&staged_app);
        return Err(format!("Could not move current .app bundle aside: {e}"));
    }
    if let Err(e) = fs::rename(&staged_app, &current_app) {
        let _ = fs::rename(&backup_app, &current_app);
        let _ = fs::remove_dir_all(&staged_app);
        return Err(format!("Could not activate new .app bundle: {e}"));
    }
    let _ = fs::remove_dir_all(&extract_dir);
    Ok(current_app)
}

/// Apply the downloaded launcher update and relaunch LPBrower.
#[tauri::command]
pub async fn launcher_update_apply(app: AppHandle) -> Result<(), String> {
    if crate::reset::active() {
        return Err(crate::reset::WAIT.to_string());
    }
    let found = FOUND
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone()
        .ok_or("No update was found to install")?;
    let dl = DOWNLOADED
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone()
        .filter(|d| d.version == found.offer.version && d.path.is_file())
        .ok_or("The update has not been downloaded yet")?;

    crate::window_state::flush_now();
    crate::space::open::close_all(crate::space::open::QUIT_WAIT).await;
    crate::space::profile::close_unsealed(crate::space::open::QUIT_WAIT).await;

    match dl.kind {
        InstallKind::PortableExe => {
            #[cfg(target_os = "windows")]
            {
                let current_exe = std::env::current_exe()
                    .map_err(|e| format!("Could not locate current executable: {e}"))?;
                let target = current_exe.clone();
                let source = dl.path.clone();
                tokio::task::spawn_blocking(move || swap_portable_exe(&target, &source))
                    .await
                    .map_err(|e| format!("Swap task failed: {e}"))??;

                std::process::Command::new(&current_exe)
                    .spawn()
                    .map_err(|e| format!("Updated binary was swapped, but relaunch failed: {e}"))?;
                app.exit(0);
                Ok(())
            }
            #[cfg(not(target_os = "windows"))]
            {
                let _ = app;
                Err("Portable .exe update is only supported on Windows".into())
            }
        }
        InstallKind::WindowsInstaller => {
            #[cfg(target_os = "windows")]
            {
                let ext = dl
                    .path
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("")
                    .to_ascii_lowercase();
                if ext == "msi" {
                    std::process::Command::new("msiexec")
                        .arg("/i")
                        .arg(&dl.path)
                        .arg("/passive")
                        .spawn()
                        .map_err(|e| format!("Could not start MSI installer: {e}"))?;
                } else {
                    std::process::Command::new(&dl.path)
                        .spawn()
                        .map_err(|e| format!("Could not start Setup installer: {e}"))?;
                }
                app.exit(0);
                Ok(())
            }
            #[cfg(not(target_os = "windows"))]
            {
                let _ = app;
                Err("Windows installer is only supported on Windows".into())
            }
        }
        InstallKind::MacosApp => {
            #[cfg(target_os = "macos")]
            {
                let current_exe = std::env::current_exe()
                    .map_err(|e| format!("Could not locate current executable: {e}"))?;
                let archive = dl.path.clone();
                let app_bundle = tokio::task::spawn_blocking(move || {
                    apply_macos_app_bundle(&current_exe, &archive)
                })
                .await
                .map_err(|e| format!("macOS update task failed: {e}"))??;

                std::process::Command::new("open")
                    .arg("-n")
                    .arg(&app_bundle)
                    .spawn()
                    .map_err(|e| format!("Updated app was installed, but relaunch failed: {e}"))?;
                app.exit(0);
                Ok(())
            }
            #[cfg(not(target_os = "macos"))]
            {
                let _ = app;
                Err("macOS bundle update is only supported on macOS".into())
            }
        }
        InstallKind::Manual => Err("Please download and install this update manually from the release page.".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_assets() -> Vec<ReleaseAsset> {
        vec![
            ReleaseAsset {
                name: "LPBrower.exe".into(),
                download_url: "https://example.invalid/LPBrower.exe".into(),
                api_url: None,
                size: 45_000_000,
                digest_sha256: None,
            },
            ReleaseAsset {
                name: "LPBrower_Setup_x64.exe".into(),
                download_url: "https://example.invalid/LPBrower_Setup_x64.exe".into(),
                api_url: None,
                size: 16_000_000,
                digest_sha256: None,
            },
            ReleaseAsset {
                name: "LPBrower_Setup_x64.msi".into(),
                download_url: "https://example.invalid/LPBrower_Setup_x64.msi".into(),
                api_url: None,
                size: 20_000_000,
                digest_sha256: None,
            },
            ReleaseAsset {
                name: "LPBrower_Setup_macOS_arm64.dmg".into(),
                download_url: "https://example.invalid/LPBrower_Setup_macOS_arm64.dmg".into(),
                api_url: None,
                size: 24_000_000,
                digest_sha256: None,
            },
            ReleaseAsset {
                name: "LPBrower_Launcher_macOS_arm64.app.tar.gz".into(),
                download_url: "https://example.invalid/LPBrower_Launcher_macOS_arm64.app.tar.gz".into(),
                api_url: None,
                size: 25_000_000,
                digest_sha256: None,
            },
        ]
    }

    #[test]
    fn semver_comparison_handles_v_prefix_and_prerelease() {
        assert!(is_newer("v2.1.6", "2.1.5"));
        assert!(is_newer("2.2.0", "2.1.9"));
        assert!(is_newer("3.0.0", "2.9.9"));
        assert!(!is_newer("v2.1.6", "2.1.6"));
        assert!(!is_newer("2.1.5", "2.1.6"));
    }

    #[test]
    fn asset_selection_matches_portable_vs_installed_and_macos() {
        let assets = sample_assets();
        let (k, a) = select_asset(&assets, "windows", "x86_64", false, true, true);
        assert_eq!(k, InstallKind::PortableExe);
        assert_eq!(a.unwrap().name, "LPBrower.exe");

        let (k, a) = select_asset(&assets, "windows", "x86_64", true, false, true);
        assert_eq!(k, InstallKind::WindowsInstaller);
        assert_eq!(a.unwrap().name, "LPBrower_Setup_x64.exe");

        let (k, a) = select_asset(&assets, "macos", "aarch64", false, true, true);
        assert_eq!(k, InstallKind::MacosApp);
        assert_eq!(a.unwrap().name, "LPBrower_Launcher_macOS_arm64.app.tar.gz");

        let (k, a) = select_asset(&assets, "macos", "aarch64", false, false, true);
        assert_eq!(k, InstallKind::Manual);
        assert_eq!(a.unwrap().name, "LPBrower_Setup_macOS_arm64.dmg");

        let (k, _) = select_asset(&assets, "windows", "x86_64", false, true, false);
        assert_eq!(k, InstallKind::Manual);
    }

    #[test]
    fn portable_exe_swap_replaces_atomically_and_preserves_backup() {
        let tmp = std::env::temp_dir().join(format!("lpb-update-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();
        let cur = tmp.join("LPBrower.exe");
        let dl = tmp.join("downloaded.exe");
        fs::write(&cur, b"MZ-old-binary").unwrap();
        fs::write(&dl, b"MZ-new-binary-v2.1.7").unwrap();

        let backup = swap_portable_exe(&cur, &dl).unwrap();
        assert_eq!(fs::read(&cur).unwrap(), b"MZ-new-binary-v2.1.7");
        assert_eq!(fs::read(&backup).unwrap(), b"MZ-old-binary");
        assert!(!tmp.join("LPBrower.exe.new").exists());

        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn windows_install_detection_distinguishes_portable_from_installed() {
        assert!(!detect_windows_installed(Path::new(r"G:\Vibe Tool\LPBrower\dist-desktop\LPBrower.exe")));
        assert!(!detect_windows_installed(Path::new(r"C:\Users\Admin\Desktop\LPBrower.exe")));
        assert!(detect_windows_installed(Path::new(r"C:\Program Files\LPBrower Launcher\LPBrower Launcher.exe")));
        assert!(detect_windows_installed(Path::new(r"C:\Users\Admin\AppData\Local\LPBrower Launcher\LPBrower Launcher.exe")));
    }
}
