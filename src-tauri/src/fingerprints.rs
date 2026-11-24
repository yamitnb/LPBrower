// User-managed Fingerprint Library.
//
// Each entry is a full FingerprintConfig JSON stored under
// `$CONFIG/shardx-launcher/fingerprints/<id>.json`.  The GPU select in
// the profile editor pulls its options from here — i.e. the user can
// curate which GPUs/devices show up by importing their own JSON files
// (or by deleting bundled ones).
//
// On first run we seed the directory with the bundled gpu presets so
// the launcher is usable out of the box.

use crate::store;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::SystemTime;

/// One row in the library UI; also what the profile editor uses to
/// populate the GPU select.  `payload` is the verbatim FingerprintConfig
/// (without the `_meta` envelope launcher profiles use).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryEntry {
    pub id: String,
    pub label: String,
    pub platform: String,
    pub chrome: String,
    pub gpu: String,
    pub tag_color: String,
    /// True for the bundled starter set — UI marks them so the user
    /// knows they can re-seed by deleting/reimporting.
    #[serde(default)]
    pub builtin: bool,
    /// Full FingerprintConfig JSON.  Stored inline so the frontend can
    /// preview it and the profile editor can read derived fields
    /// (screen, platform, webgl) without a second round-trip.
    pub payload: Value,
}

fn safe_id(id: &str) -> Result<String> {
    if id.is_empty() || id.contains(['/', '\\']) || id.contains("..") {
        anyhow::bail!("invalid fingerprint id");
    }
    Ok(id.to_string())
}

fn path_for(id: &str) -> Result<PathBuf> {
    let id = safe_id(id)?;
    Ok(store::fingerprints_dir()?.join(format!("{id}.json")))
}

/// Seed the library with the bundled GPU presets on the first call
fn tag_color_for(platform: &str) -> String {
    match platform {
        "macOS" => "#8b5cf6".into(),
        "Windows" => "#5dade2".into(),
        "Linux" => "#4ade80".into(),
        // navigator.platform on a phone is "Linux armv8l", so a handset is
        // matched on the prefix rather than on equality.
        p if p.starts_with("Android") || p.starts_with("Linux arm") => "#fb923c".into(),
        _ => "#a78bfa".into(),
    }
}

type FpEntryCache = HashMap<PathBuf, (Option<SystemTime>, u64, Option<LibraryEntry>)>;

fn fp_cache() -> &'static Mutex<FpEntryCache> {
    static CACHE: OnceLock<Mutex<FpEntryCache>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn read_entry_file(path: &Path) -> Option<LibraryEntry> {
    let body = fs::read_to_string(path).ok()?;
    let stem = path.file_stem().and_then(|s| s.to_str()).map(String::from);
    if let Ok(mut e) = serde_json::from_str::<LibraryEntry>(&body) {
        if let Some(stem) = stem {
            e.id = stem;
        }
        Some(e)
    } else if let Ok(payload) = serde_json::from_str::<Value>(&body) {
        let id = stem.unwrap_or_else(|| "imported".to_string());
        Some(wrap_payload(&id, &payload))
    } else {
        None
    }
}

pub fn list_all() -> Result<Vec<LibraryEntry>> {
    // Pure filesystem read with per-file (mtime, size) in-memory caching so
    // repeated listing or preset inference does not re-parse dozens of JSON files.
    let dir = store::fingerprints_dir()?;
    let mut out = Vec::new();
    let mut seen_paths = Vec::new();
    for entry in fs::read_dir(&dir)? {
        let entry = entry?;
        let path = entry.path();
        if store::is_os_litter(&entry.file_name()) || path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }
        let meta = entry.metadata().ok();
        let stamp = meta
            .as_ref()
            .map(|m| (m.modified().ok(), m.len()))
            .unwrap_or((None, 0));
        seen_paths.push(path.clone());

        let cached = fp_cache()
            .lock()
            .ok()
            .and_then(|g| g.get(&path).filter(|(mt, len, _)| (*mt, *len) == stamp).map(|(_, _, e)| e.clone()));

        let item = match cached {
            Some(hit) => hit,
            None => {
                let parsed = read_entry_file(&path);
                if let Ok(mut g) = fp_cache().lock() {
                    g.insert(path, (stamp.0, stamp.1, parsed.clone()));
                }
                parsed
            }
        };
        if let Some(e) = item {
            out.push(e);
        }
    }
    if let Ok(mut g) = fp_cache().lock() {
        let live: std::collections::HashSet<&PathBuf> = seen_paths.iter().collect();
        g.retain(|p, _| !p.starts_with(&dir) || live.contains(p));
    }
    out.sort_by(|a, b| a.label.cmp(&b.label));
    Ok(out)
}

/// Pull a label / platform / chrome / GPU description out of a raw
/// FingerprintConfig.  Used both at import time and when listing
/// bare-JSON files in the library dir.
fn wrap_payload(id: &str, p: &Value) -> LibraryEntry {
    let label = p
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or(id)
        .to_string();
    let platform = p
        .get("navigator")
        .and_then(|n| n.get("platform"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let chrome = p
        .get("client_hints")
        .and_then(|c| c.get("brand_version"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let gpu = p
        .get("webgl")
        .and_then(|w| w.get("renderer"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    LibraryEntry {
        id: id.to_string(),
        label,
        platform: platform.clone(),
        chrome,
        gpu,
        tag_color: tag_color_for(&platform),
        builtin: false,
        payload: p.clone(),
    }
}

pub fn get(id: &str) -> Result<Option<LibraryEntry>> {
    let Ok(path) = path_for(id) else {
        return Ok(None);
    };
    let Ok(meta) = fs::metadata(&path) else {
        return Ok(None);
    };
    let stamp = (meta.modified().ok(), meta.len());
    if let Ok(g) = fp_cache().lock() {
        if let Some((mt, len, entry)) = g.get(&path) {
            if (*mt, *len) == stamp {
                return Ok(entry.clone());
            }
        }
    }
    let parsed = read_entry_file(&path);
    if let Ok(mut g) = fp_cache().lock() {
        g.insert(path, (stamp.0, stamp.1, parsed.clone()));
    }
    Ok(parsed)
}

/// Import a raw FingerprintConfig JSON.  Accepts the user's text and
/// returns the saved entry.  If `id` is empty a slug is derived from
/// `payload.name` (or a UUID if no name).
pub fn import(json_text: &str, id_hint: Option<String>) -> Result<LibraryEntry> {
    let payload: Value =
        serde_json::from_str(json_text).context("not a valid JSON FingerprintConfig")?;
    let raw_id = id_hint
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| {
            payload
                .get("name")
                .and_then(|v| v.as_str())
                .map(slugify)
                .unwrap_or_else(|| uuid::Uuid::new_v4().to_string())
        });
    let id = ensure_unique_id(&raw_id)?;
    let entry = wrap_payload(&id, &payload);
    let path = path_for(&id)?;
    fs::write(path, serde_json::to_string_pretty(&entry)?)?;
    Ok(entry)
}

fn slugify(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
        } else if c == ' ' || c == '_' || c == '-' || c == '.' {
            out.push('-');
        }
    }
    let trimmed = out.trim_matches('-').to_string();
    if trimmed.is_empty() { uuid::Uuid::new_v4().to_string() } else { trimmed }
}

fn ensure_unique_id(base: &str) -> Result<String> {
    let dir = store::fingerprints_dir()?;
    if !dir.join(format!("{base}.json")).exists() {
        return Ok(base.into());
    }
    for n in 2..1000 {
        let cand = format!("{base}-{n}");
        if !dir.join(format!("{cand}.json")).exists() {
            return Ok(cand);
        }
    }
    Ok(format!("{base}-{}", uuid::Uuid::new_v4()))
}

pub fn delete(id: &str) -> Result<()> {
    let path = path_for(id)?;
    if path.exists() {
        fs::remove_file(path)?;
    }
    Ok(())
}
