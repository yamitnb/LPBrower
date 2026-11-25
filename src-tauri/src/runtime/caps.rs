//! What an engine binary can do, read from the binary itself: a folder copied in
//! by hand or a custom browser path carries no manifest, but its code is there.

use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::SystemTime;

/// Kept in an engine whose --load-extension loads past Enhanced Safe Browsing
/// (components/fingerprint/host_extensions.h in the engine). Never change it.
const LOAD_EXTENSION_MARKER: &[u8] = b"shardx-engine:load-extension-past-gates";

/// Whether the engine at `bin` loads the library without --disable-extensions-except;
/// an older one needs the switch under Enhanced Safe Browsing. Unknown counts as older.
pub fn loads_extensions_alone(bin: &Path) -> bool {
    has_marker(bin, LOAD_EXTENSION_MARKER)
}

type Stamp = Vec<(PathBuf, u64, Option<SystemTime>)>;

#[cfg(not(test))]
#[derive(serde::Serialize, serde::Deserialize)]
struct DiskCapEntry {
    stamp: Vec<(String, u64, Option<u64>)>,
    found: bool,
}

#[cfg(not(test))]
fn disk_stamp(stamp: &Stamp) -> Vec<(String, u64, Option<u64>)> {
    stamp
        .iter()
        .map(|(p, len, mtime)| {
            let secs = mtime
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs());
            (p.to_string_lossy().into_owned(), *len, secs)
        })
        .collect()
}

#[cfg(not(test))]
fn cap_cache_path(bin: &Path) -> Option<PathBuf> {
    bin.parent().map(|d| d.join(".lpbrower-engine-caps.json"))
}

#[cfg(not(test))]
fn load_disk_cap(bin: &Path, marker: &[u8], stamp: &Stamp) -> Option<bool> {
    if marker != LOAD_EXTENSION_MARKER {
        return None;
    }
    let path = cap_cache_path(bin)?;
    let raw = std::fs::read_to_string(path).ok()?;
    let entry: DiskCapEntry = serde_json::from_str(&raw).ok()?;
    if entry.stamp == disk_stamp(stamp) {
        Some(entry.found)
    } else {
        None
    }
}

#[cfg(not(test))]
fn save_disk_cap(bin: &Path, marker: &[u8], stamp: &Stamp, found: bool) {
    if marker != LOAD_EXTENSION_MARKER {
        return;
    }
    if let Some(path) = cap_cache_path(bin) {
        let entry = DiskCapEntry { stamp: disk_stamp(stamp), found };
        if let Ok(body) = serde_json::to_string(&entry) {
            let _ = std::fs::write(path, body);
        }
    }
}

/// One read per engine version: the answer is kept against the size and mtime
/// of every file that was read, and an update changes them.
fn has_marker(bin: &Path, marker: &[u8]) -> bool {
    static SEEN: OnceLock<Mutex<HashMap<(PathBuf, Vec<u8>), (Stamp, bool)>>> = OnceLock::new();
    let files = code_files(bin);
    let stamp: Stamp = files
        .iter()
        .filter_map(|f| {
            let m = std::fs::metadata(f).ok()?;
            Some((f.clone(), m.len(), m.modified().ok()))
        })
        .collect();
    // Held through the read, so launches started together read the engine once.
    let mut seen = SEEN.get_or_init(Default::default).lock().unwrap_or_else(|e| e.into_inner());
    let key = (bin.to_path_buf(), marker.to_vec());
    if let Some((s, found)) = seen.get(&key) {
        if *s == stamp {
            return *found;
        }
    }
    #[cfg(not(test))]
    if !stamp.is_empty() {
        if let Some(found) = load_disk_cap(bin, marker, &stamp) {
            seen.insert(key, (stamp, found));
            return found;
        }
    }
    // Every copy the stub might load must have it: a stale one without it may be the one in use.
    let found = !files.is_empty() && files.iter().all(|f| file_contains(f, marker).unwrap_or(false));
    #[cfg(not(test))]
    if !stamp.is_empty() {
        save_disk_cap(bin, marker, &stamp, found);
    }
    seen.insert(key, (stamp, found));
    found
}

/// Every file the engine may run its code from: each version of the Chromium framework
/// (`{stub} Framework.framework` or `<Product> Framework.framework` when the outer stub
/// was renamed from `ShardX` to `LPBrowser`), each Windows chrome.dll beside chrome.exe
/// or in a version folder, else `bin`.
fn code_files(bin: &Path) -> Vec<PathBuf> {
    let dir = bin.parent().unwrap_or(Path::new(""));
    if let (Some(contents), Some(stub)) = (
        dir.parent().filter(|_| dir.file_name().is_some_and(|n| n == "MacOS")),
        bin.file_name(),
    ) {
        let frameworks = contents.join("Frameworks");
        let stub_name = format!("{} Framework", stub.to_string_lossy());
        let collect_fw = |fw: &Path, name: &str| -> Vec<PathBuf> {
            let mut out: Vec<PathBuf> =
                read_dir(&fw.join("Versions")).into_iter().map(|v| v.join(name)).collect();
            if out.is_empty() {
                out.push(fw.join(name));
            }
            out.retain(|p| p.is_file());
            out
        };
        let mut out = collect_fw(&frameworks.join(format!("{stub_name}.framework")), &stub_name);
        if out.is_empty() {
            for fw in read_dir(&frameworks) {
                if fw.extension().and_then(|e| e.to_str()) != Some("framework") {
                    continue;
                }
                let Some(stem) = fw.file_stem().map(|s| s.to_string_lossy().into_owned()) else {
                    continue;
                };
                if stem.ends_with(" Framework") {
                    out.extend(collect_fw(&fw, &stem));
                }
            }
        }
        if !out.is_empty() {
            return out;
        }
    }
    let mut out: Vec<PathBuf> = std::iter::once(dir.join("chrome.dll"))
        .chain(read_dir(dir).into_iter().map(|sub| sub.join("chrome.dll")))
        .filter(|p| p.is_file())
        .collect();
    if out.is_empty() && bin.is_file() {
        out.push(bin.to_path_buf());
    }
    out
}

/// Real subfolders only: a symlink such as the framework's Versions/Current is not followed.
fn read_dir(dir: &Path) -> Vec<PathBuf> {
    let Ok(rd) = std::fs::read_dir(dir) else { return Vec::new() };
    let mut v: Vec<PathBuf> =
        rd.flatten().filter(|e| e.file_type().is_ok_and(|t| t.is_dir())).map(|e| e.path()).collect();
    v.sort();
    v
}

/// Streams the file, so a 300 MB engine never sits in memory at once.
fn file_contains(path: &Path, needle: &[u8]) -> std::io::Result<bool> {
    file_contains_with(path, needle, 8 << 20)
}

fn file_contains_with(path: &Path, needle: &[u8], chunk: usize) -> std::io::Result<bool> {
    if needle.is_empty() {
        return Ok(true);
    }
    let mut f = File::open(path)?;
    let keep = needle.len() - 1;
    let chunk = chunk.max(needle.len());
    let mut buf = vec![0u8; chunk + keep];
    let mut have = 0;
    loop {
        let n = f.read(&mut buf[have..have + chunk])?;
        if n == 0 {
            return Ok(false);
        }
        let end = have + n;
        if contains(&buf[..end], needle) {
            return Ok(true);
        }
        // The tail that could still start a match carries over to the next read.
        let k = keep.min(end);
        buf.copy_within(end - k..end, 0);
        have = k;
    }
}

fn contains(hay: &[u8], needle: &[u8]) -> bool {
    let (first, rest) = (needle[0], &needle[1..]);
    let mut from = 0;
    while hay.len() - from >= needle.len() {
        let last = hay.len() - needle.len();
        match hay[from..=last].iter().position(|&b| b == first) {
            None => return false,
            Some(p) => {
                let at = from + p;
                if &hay[at + 1..at + needle.len()] == rest {
                    return true;
                }
                from = at + 1;
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A scratch folder, removed when the test is done with it.
    struct Scratch(PathBuf);
    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    impl std::ops::Deref for Scratch {
        type Target = Path;
        fn deref(&self) -> &Path {
            &self.0
        }
    }

    fn dir(name: &str) -> Scratch {
        let d = std::env::temp_dir().join(format!("shardx-caps-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        Scratch(d)
    }

    fn code(with_marker: bool) -> Vec<u8> {
        let mut v = vec![0x5a_u8; 4096];
        if with_marker {
            v.extend_from_slice(LOAD_EXTENSION_MARKER);
        }
        v.extend_from_slice(&[0u8; 512]);
        v
    }

    fn write(p: &Path, bytes: &[u8]) -> PathBuf {
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, bytes).unwrap();
        p.to_path_buf()
    }

    #[test]
    fn a_single_binary_engine_is_read_whole() {
        let d = dir("linux");
        assert!(loads_extensions_alone(&write(&d.join("new/chrome"), &code(true))));
        assert!(!loads_extensions_alone(&write(&d.join("old/chrome"), &code(false))));
    }

    #[test]
    fn a_mac_engine_is_read_in_its_framework_not_its_stub() {
        let d = dir("mac");
        let app = d.join("ShardX-Mac-arm64/ShardX.app/Contents");
        let stub = write(&app.join("MacOS/ShardX"), &code(false));
        let fw = app.join("Frameworks/ShardX Framework.framework");
        let v1 = write(&fw.join("Versions/156.0.8078.48/ShardX Framework"), &code(false));
        #[cfg(unix)]
        std::os::unix::fs::symlink("156.0.8078.48", fw.join("Versions/Current")).unwrap();
        // Another framework of the app is not the engine's.
        write(&app.join("Frameworks/Other.framework/Versions/A/Other"), &code(false));
        assert!(!loads_extensions_alone(&stub));
        // A new framework copied over the old one (size changes): read again.
        write(&v1, &code(true));
        assert!(loads_extensions_alone(&stub));
        // An old version left beside it: the stub may be the one that loads it.
        write(&fw.join("Versions/155.0.7000.1/ShardX Framework"), &code(false));
        assert!(!loads_extensions_alone(&stub));
    }

    #[test]
    fn a_rebranded_mac_stub_still_reads_shardx_framework() {
        let d = dir("mac-rebranded");
        let app = d.join("ShardX-Mac-arm64/LPBrowser.app/Contents");
        let stub = write(&app.join("MacOS/LPBrowser"), &code(false));
        let fw = app.join("Frameworks/ShardX Framework.framework");
        write(&fw.join("Versions/158.0.8127.36/ShardX Framework"), &code(true));
        write(&app.join("Frameworks/Other.framework/Versions/A/Other"), &code(false));
        assert!(loads_extensions_alone(&stub));
    }

    #[test]
    fn a_windows_engine_is_read_in_chrome_dll_beside_or_in_its_version_folder() {
        let d = dir("win");
        let flat = write(&d.join("flat/chrome.exe"), &code(false));
        write(&d.join("flat/chrome.dll"), &code(true));
        assert!(loads_extensions_alone(&flat));
        let versioned = write(&d.join("versioned/chrome.exe"), &code(false));
        write(&d.join("versioned/156.0.8078.48/chrome.dll"), &code(true));
        assert!(loads_extensions_alone(&versioned));
        let old = write(&d.join("old/chrome.exe"), &code(false));
        write(&d.join("old/chrome.dll"), &code(false));
        assert!(!loads_extensions_alone(&old));
        // A new flat chrome.dll over an old version folder: chrome.exe looks there first.
        let mixed = write(&d.join("mixed/chrome.exe"), &code(false));
        write(&d.join("mixed/chrome.dll"), &code(true));
        write(&d.join("mixed/152.0.7000.1/chrome.dll"), &code(false));
        assert!(!loads_extensions_alone(&mixed));
    }

    #[test]
    fn no_engine_is_an_old_engine() {
        assert!(!loads_extensions_alone(&dir("none").join("missing/chrome")));
    }

    #[test]
    fn a_marker_across_two_reads_is_found() {
        let d = dir("chunks");
        for split in 1..LOAD_EXTENSION_MARKER.len() {
            let mut bytes = vec![b's'; 64 - split];
            bytes.extend_from_slice(LOAD_EXTENSION_MARKER);
            bytes.extend_from_slice(b"tail");
            let p = write(&d.join(format!("f{split}")), &bytes);
            assert!(file_contains_with(&p, LOAD_EXTENSION_MARKER, 64).unwrap(), "split {split}");
            // One byte short of the whole marker is not the marker.
            let cut = write(&d.join(format!("c{split}")), &bytes[..64 - split + LOAD_EXTENSION_MARKER.len() - 1]);
            assert!(!file_contains_with(&cut, LOAD_EXTENSION_MARKER, 64).unwrap(), "cut {split}");
        }
    }
}
