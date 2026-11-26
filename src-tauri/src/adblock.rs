//! Built-in Manifest V3 DeclarativeNetRequest ad & tracker blocker for LPBrowser.
//!
//! Materializes a production-grade unpacked MV3 extension under
//! `<runtime_dir>/lpbrowser-adblock/` and passes it via `--load-extension` only
//! when ad blocking is enabled for the profile being launched.
//!
//! Powered by ~28,500 compiled Chromium DNR rules from:
//! - ABPVN (Vietnamese Adblock list) + curated VN/24h/Admicro/eClick/Adtima rules
//! - uBlock Origin uAssets (quick-fixes, filters, privacy, badware)
//! - Peter Lowe's Ad & Tracking Server list
//! - EasyList + EasyPrivacy
//! - Scoped cosmetic CSS (global, Vietnamese portals, search ads, YouTube UI)
//! - Dedicated YouTube MAIN-world ad-response cleaner & auto-skipper (`youtube.com` only)
//!
//! Anti-detect & marketer safety guarantees:
//! - Priority-100 allowlist exempts major Ads Manager & Seller portals (Google Ads,
//!   Meta Business / Ads Manager, TikTok Ads, Shopee Seller, Amazon Seller).
//! - Priority-90 allowlist protects Auth, Payment, CAPTCHA, and Login SDKs.
//! - Zero `web_accessible_resources` (not probeable via `chrome-extension://`).
//! - Zero JS injected on any non-YouTube domain (keeps anti-detect fingerprints clean).

use flate2::read::GzDecoder;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

/// Bump whenever manifest, bundled filter rules, cosmetic CSS, or YT script changes
/// so existing installs refresh `<runtime_dir>/lpbrowser-adblock/` on next launch.
pub const ADBLOCK_VERSION: &str = "2.2.1";

const EXTENSION_DIR_NAME: &str = "lpbrowser-adblock";
const VERSION_STAMP_FILE: &str = ".lpbrowser-adblock-version";

const ICON_16_PNG: &[u8] = include_bytes!("../assets/adblock/icon16.png");
const ICON_32_PNG: &[u8] = include_bytes!("../assets/adblock/icon32.png");
const ICON_48_PNG: &[u8] = include_bytes!("../assets/adblock/icon48.png");
const ICON_128_PNG: &[u8] = include_bytes!("../assets/adblock/icon128.png");

const RULES_CORE_GZ: &[u8] = include_bytes!("../assets/adblock/rules_core.json.gz");
const RULES_EASYLIST_GZ: &[u8] = include_bytes!("../assets/adblock/rules_easylist.json.gz");
const RULES_PRIVACY_GZ: &[u8] = include_bytes!("../assets/adblock/rules_privacy.json.gz");

const COSMETIC_GLOBAL_CSS_GZ: &[u8] = include_bytes!("../assets/adblock/cosmetic_global.css.gz");
const COSMETIC_VN_CSS: &str = include_str!("../assets/adblock/cosmetic_vn.css");
const COSMETIC_SEARCH_CSS: &str = include_str!("../assets/adblock/cosmetic_search.css");
const COSMETIC_YT_CSS: &str = include_str!("../assets/adblock/cosmetic_yt.css");
const YT_ADBLOCK_JS: &str = include_str!("../assets/adblock/yt_adblock.js");

/// Marketer & Seller portals excluded from cosmetic CSS injection.
const EXCLUDED_MARKETER_DOMAINS: &[&str] = &[
    "ads.google.com",
    "adwords.google.com",
    "analytics.google.com",
    "tagmanager.google.com",
    "adsmanager.facebook.com",
    "business.facebook.com",
    "ads.tiktok.com",
    "business.tiktok.com",
    "seller.shopee.vn",
    "seller.shopee.com",
    "banhang.shopee.vn",
    "sellercentral.amazon.com",
    "advertising.amazon.com",
];

/// Top Vietnamese news & entertainment portals receiving scoped ABPVN cosmetic CSS.
const VN_COSMETIC_MATCHES: &[&str] = &[
    "*://*.24h.com.vn/*",
    "*://*.kenh14.vn/*",
    "*://*.vnexpress.net/*",
    "*://*.dantri.com.vn/*",
    "*://*.cafef.vn/*",
    "*://*.cafebiz.vn/*",
    "*://*.soha.vn/*",
    "*://*.genk.vn/*",
    "*://*.gamek.vn/*",
    "*://*.thanhnien.vn/*",
    "*://*.tuoitre.vn/*",
    "*://*.znews.vn/*",
    "*://*.zingnews.vn/*",
    "*://*.vietnamnet.vn/*",
    "*://*.eva.vn/*",
    "*://*.afamily.vn/*",
    "*://*.vtcnews.vn/*",
    "*://*.laodong.vn/*",
    "*://*.tienphong.vn/*",
    "*://*.baomoi.com/*",
    "*://*.tinhte.vn/*",
    "*://*.voz.vn/*",
    "*://*.nguoiduatin.vn/*",
    "*://*.saostar.vn/*",
    "*://*.2sao.vn/*",
    "*://*.vov.vn/*",
    "*://*.nld.com.vn/*",
    "*://*.sggp.org.vn/*",
    "*://*.plo.vn/*",
    "*://*.vneconomy.vn/*",
    "*://*.baogiaothong.vn/*",
    "*://*.bongdaplus.vn/*",
    "*://*.thethao247.vn/*",
];

const SEARCH_COSMETIC_MATCHES: &[&str] = &[
    "*://*.google.com/*",
    "*://*.google.com.vn/*",
    "*://*.google.co.uk/*",
    "*://*.google.de/*",
    "*://*.google.co.jp/*",
    "*://*.bing.com/*",
    "*://*.duckduckgo.com/*",
];

const YOUTUBE_MATCHES: &[&str] = &["*://*.youtube.com/*"];

/// Resolve whether ad blocking should be active for a profile launch.
/// `profile_override`: `None` = follow global setting, `Some(true)` = force on, `Some(false)` = force off.
pub fn effective_enabled(global_enabled: bool, profile_override: Option<bool>) -> bool {
    profile_override.unwrap_or(global_enabled)
}

fn gunzip_bytes(gz: &[u8]) -> std::io::Result<Vec<u8>> {
    let mut dec = GzDecoder::new(gz);
    let mut out = Vec::new();
    dec.read_to_end(&mut out)?;
    Ok(out)
}

/// Build the MV3 `manifest.json` JSON string.
pub fn manifest_json() -> String {
    let exclude_matches: Vec<String> = EXCLUDED_MARKETER_DOMAINS
        .iter()
        .map(|d| format!("*://{d}/*"))
        .collect();

    let manifest = serde_json::json!({
        "manifest_version": 3,
        "name": "LPBrowser Ad Blocker",
        "short_name": "LP AdBlock",
        "version": ADBLOCK_VERSION,
        "description": "Built-in ad, popup & tracker blocker for LPBrowser (EasyList, EasyPrivacy, uBlock, ABPVN, YouTube).",
        "icons": {
            "16": "icon16.png",
            "32": "icon32.png",
            "48": "icon48.png",
            "128": "icon128.png"
        },
        "action": {
            "default_title": "LPBrowser Ad Blocker",
            "default_icon": {
                "16": "icon16.png",
                "32": "icon32.png",
                "48": "icon48.png",
                "128": "icon128.png"
            }
        },
        "permissions": ["declarativeNetRequest"],
        "declarative_net_request": {
            "rule_resources": [
                {
                    "id": "lpbrowser_core",
                    "enabled": true,
                    "path": "rules_core.json"
                },
                {
                    "id": "lpbrowser_easylist",
                    "enabled": true,
                    "path": "rules_easylist.json"
                },
                {
                    "id": "lpbrowser_privacy",
                    "enabled": true,
                    "path": "rules_privacy.json"
                }
            ]
        },
        "content_scripts": [
            {
                "matches": ["http://*/*", "https://*/*"],
                "exclude_matches": exclude_matches,
                "css": ["cosmetic_global.css"],
                "all_frames": false,
                "run_at": "document_start"
            },
            {
                "matches": VN_COSMETIC_MATCHES,
                "css": ["cosmetic_vn.css"],
                "all_frames": false,
                "run_at": "document_start"
            },
            {
                "matches": SEARCH_COSMETIC_MATCHES,
                "exclude_matches": exclude_matches,
                "css": ["cosmetic_search.css"],
                "all_frames": false,
                "run_at": "document_start"
            },
            {
                "matches": YOUTUBE_MATCHES,
                "css": ["cosmetic_yt.css"],
                "all_frames": false,
                "run_at": "document_start"
            },
            {
                "matches": YOUTUBE_MATCHES,
                "js": ["yt_adblock.js"],
                "world": "MAIN",
                "all_frames": false,
                "run_at": "document_start"
            }
        ]
    });
    serde_json::to_string_pretty(&manifest).expect("manifest json serialization")
}

/// Materialize (or refresh when version changed) the unpacked MV3 ad-blocker
/// extension directory inside `runtime_dir` and return its path.
pub fn ensure_extension(runtime_dir: &Path) -> std::io::Result<PathBuf> {
    let dir = runtime_dir.join(EXTENSION_DIR_NAME);
    let stamp = dir.join(VERSION_STAMP_FILE);

    let required_files = [
        "manifest.json",
        "rules_core.json",
        "rules_easylist.json",
        "rules_privacy.json",
        "cosmetic_global.css",
        "cosmetic_vn.css",
        "cosmetic_search.css",
        "cosmetic_yt.css",
        "yt_adblock.js",
        "icon16.png",
        "icon32.png",
        "icon48.png",
        "icon128.png",
    ];

    let up_to_date = fs::read_to_string(&stamp)
        .map(|s| s.trim() == ADBLOCK_VERSION)
        .unwrap_or(false)
        && required_files.iter().all(|f| dir.join(f).is_file());

    if up_to_date {
        return Ok(dir);
    }

    fs::create_dir_all(&dir)?;

    // Drop stale files & Chromium's cached indexed rulesets from previous versions
    // so Chromium cleanly compiles the new DNR rulesets on next launch.
    let _ = fs::remove_dir_all(dir.join("_metadata"));
    let _ = fs::remove_file(dir.join("rules.json"));
    let _ = fs::remove_file(dir.join("cosmetic.css"));
    let _ = fs::remove_file(dir.join("icon64.png"));

    fs::write(dir.join("manifest.json"), manifest_json())?;
    fs::write(dir.join("rules_core.json"), gunzip_bytes(RULES_CORE_GZ)?)?;
    fs::write(dir.join("rules_easylist.json"), gunzip_bytes(RULES_EASYLIST_GZ)?)?;
    fs::write(dir.join("rules_privacy.json"), gunzip_bytes(RULES_PRIVACY_GZ)?)?;
    fs::write(dir.join("cosmetic_global.css"), gunzip_bytes(COSMETIC_GLOBAL_CSS_GZ)?)?;
    fs::write(dir.join("cosmetic_vn.css"), COSMETIC_VN_CSS)?;
    fs::write(dir.join("cosmetic_search.css"), COSMETIC_SEARCH_CSS)?;
    fs::write(dir.join("cosmetic_yt.css"), COSMETIC_YT_CSS)?;
    fs::write(dir.join("yt_adblock.js"), YT_ADBLOCK_JS)?;
    fs::write(dir.join("icon16.png"), ICON_16_PNG)?;
    fs::write(dir.join("icon32.png"), ICON_32_PNG)?;
    fs::write(dir.join("icon48.png"), ICON_48_PNG)?;
    fs::write(dir.join("icon128.png"), ICON_128_PNG)?;
    fs::write(&stamp, ADBLOCK_VERSION)?;

    Ok(dir)
}

/// Remove any persisted `lpbrowser-adblock` entry from `<udd>/Default/Preferences`
/// (`extensions.settings`) when ad blocking is disabled, so toggling Ad Blocker
/// off guarantees the built-in extension never lingers across restarts.
pub fn purge_from_profile_prefs(udd: &Path) {
    let prefs_path = udd.join("Default").join("Preferences");
    let Ok(raw) = fs::read_to_string(&prefs_path) else {
        return;
    };
    if !raw.contains(EXTENSION_DIR_NAME) {
        return;
    }
    let Ok(mut root) = serde_json::from_str::<serde_json::Value>(&raw) else {
        return;
    };
    let Some(settings) = root
        .get_mut("extensions")
        .and_then(|e| e.get_mut("settings"))
        .and_then(|s| s.as_object_mut())
    else {
        return;
    };
    let to_remove: Vec<String> = settings
        .iter()
        .filter_map(|(ext_id, entry)| {
            let p = entry.get("path").and_then(|v| v.as_str())?;
            let tail = Path::new(p).file_name().and_then(|n| n.to_str())?;
            if tail.eq_ignore_ascii_case(EXTENSION_DIR_NAME) {
                Some(ext_id.clone())
            } else {
                None
            }
        })
        .collect();
    if to_remove.is_empty() {
        return;
    }
    for k in to_remove {
        settings.remove(&k);
    }
    if let Ok(updated) = serde_json::to_string(&root) {
        let _ = fs::write(&prefs_path, updated);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn effective_enabled_respects_override_over_global() {
        assert!(!effective_enabled(false, None));
        assert!(effective_enabled(true, None));
        assert!(effective_enabled(false, Some(true)));
        assert!(!effective_enabled(true, Some(false)));
    }

    #[test]
    fn manifest_is_valid_mv3_and_scopes_js_only_to_youtube() {
        let v: serde_json::Value = serde_json::from_str(&manifest_json()).expect("valid manifest");
        assert_eq!(v["manifest_version"], 3);
        assert_eq!(v["permissions"], serde_json::json!(["declarativeNetRequest"]));
        assert!(v.get("web_accessible_resources").is_none());

        let rulesets = v["declarative_net_request"]["rule_resources"]
            .as_array()
            .expect("rule_resources array");
        assert_eq!(rulesets.len(), 3);

        let scripts = v["content_scripts"].as_array().expect("content_scripts array");
        for cs in scripts {
            if cs.get("js").is_some() {
                assert_eq!(cs["matches"], serde_json::json!(["*://*.youtube.com/*"]));
                assert_eq!(cs["world"], "MAIN");
            }
        }
    }

    #[test]
    fn bundled_rulesets_decompress_validate_and_protect_marketer_portals() {
        let mut total_rules = 0usize;
        for (name, gz) in [
            ("core", RULES_CORE_GZ),
            ("easylist", RULES_EASYLIST_GZ),
            ("privacy", RULES_PRIVACY_GZ),
        ] {
            let bytes = gunzip_bytes(gz).expect("gunzip");
            let arr: Vec<serde_json::Value> = serde_json::from_slice(&bytes).expect("valid json");
            assert!(!arr.is_empty(), "{name} ruleset should not be empty");
            let mut seen_ids = HashSet::new();
            for rule in &arr {
                let id = rule["id"].as_u64().expect("numeric id");
                assert!(id >= 1);
                assert!(seen_ids.insert(id), "duplicate id {id} in {name}");
                let act = rule["action"]["type"].as_str().expect("action type");
                assert!(
                    act == "block" || act == "allow" || act == "allowAllRequests",
                    "unexpected action type {act} in {name}"
                );
            }
            if name == "core" {
                // Rule 1 must be the priority-100 allowlist for Ads Manager / Seller portals
                assert_eq!(arr[0]["priority"], 100);
                assert_eq!(arr[0]["action"]["type"], "allow");
                let inits = arr[0]["condition"]["initiatorDomains"].as_array().unwrap();
                for portal in [
                    "ads.google.com",
                    "adsmanager.facebook.com",
                    "ads.tiktok.com",
                    "seller.shopee.vn",
                ] {
                    assert!(inits.iter().any(|d| d == portal), "missing portal {portal}");
                }
            }
            total_rules += arr.len();
        }
        assert!(total_rules >= 20_000 && total_rules <= 30_000, "unexpected total_rules={total_rules}");
    }

    #[test]
    fn ensure_extension_materializes_upgrades_and_is_idempotent() {
        let tmp = std::env::temp_dir().join(format!("lpbrowser-adblock-v2-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&tmp);

        // Simulate stale v1.0.0 artifacts + Chromium _metadata cache
        let old_dir = tmp.join(EXTENSION_DIR_NAME);
        fs::create_dir_all(old_dir.join("_metadata")).unwrap();
        fs::write(old_dir.join("rules.json"), "[]").unwrap();
        fs::write(old_dir.join(VERSION_STAMP_FILE), "1.0.0").unwrap();

        let dir = ensure_extension(&tmp).expect("ensure_extension upgrade");
        assert!(!dir.join("_metadata").exists(), "stale _metadata must be purged on upgrade");
        assert!(!dir.join("rules.json").exists(), "stale v1 rules.json must be removed");
        assert!(dir.join("manifest.json").is_file());
        assert!(dir.join("rules_core.json").is_file());
        assert!(dir.join("rules_easylist.json").is_file());
        assert!(dir.join("rules_privacy.json").is_file());
        assert!(dir.join("cosmetic_global.css").is_file());
        assert!(dir.join("cosmetic_vn.css").is_file());
        assert!(dir.join("cosmetic_search.css").is_file());
        assert!(dir.join("cosmetic_yt.css").is_file());
        assert!(dir.join("yt_adblock.js").is_file());
        assert!(dir.join("icon16.png").is_file());
        assert!(dir.join("icon32.png").is_file());
        assert!(dir.join("icon48.png").is_file());
        assert!(dir.join("icon128.png").is_file());
        assert!(!dir.join("icon64.png").exists(), "stale icon64.png must be removed");
        assert_eq!(fs::read_to_string(dir.join(VERSION_STAMP_FILE)).unwrap(), ADBLOCK_VERSION);

        // Second call hits fast path without rewriting.
        let dir2 = ensure_extension(&tmp).expect("ensure_extension second run");
        assert_eq!(dir, dir2);
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn purge_from_profile_prefs_removes_only_adblock_extension() {
        let tmp = std::env::temp_dir().join(format!("lpbrowser-adblock-prefs-{}", std::process::id()));
        let _ = fs::remove_dir_all(&tmp);
        let default_dir = tmp.join("Default");
        fs::create_dir_all(&default_dir).unwrap();
        let prefs = default_dir.join("Preferences");
        fs::write(
            &prefs,
            serde_json::json!({
                "extensions": {
                    "settings": {
                        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa": {
                            "path": "C:\\Users\\Test\\runtime\\lpbrowser-adblock"
                        },
                        "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb": {
                            "path": "C:\\Users\\Test\\extensions\\my-wallet"
                        }
                    }
                }
            })
            .to_string(),
        )
        .unwrap();

        purge_from_profile_prefs(&tmp);

        let after: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&prefs).unwrap()).unwrap();
        let map = after["extensions"]["settings"].as_object().unwrap();
        assert!(!map.contains_key("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"));
        assert!(map.contains_key("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"));
        let _ = fs::remove_dir_all(&tmp);
    }
}
