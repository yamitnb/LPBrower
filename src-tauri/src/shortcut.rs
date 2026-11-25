//! Desktop shortcut creation & `--launch-profile <id>` command-line parser.
//!
//! Creates a native OS shortcut on the user's Desktop (`<Name>.lnk` on Windows with a
//! multi-resolution custom `.ico`, `<Name>.app` bundle on macOS, `<Name>.desktop` on Linux)
//! that launches a specific profile's browser directly via `--launch-profile <id>`.

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

/// Extract `<profile_id>` from CLI arguments when invoked with `--launch-profile <id>`
/// or `--launch-profile=<id>`.
pub fn extract_launch_profile_arg(args: &[String]) -> Option<String> {
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        let trimmed = arg.trim();
        if trimmed == "--launch-profile" {
            if let Some(val) = iter.next() {
                let id = val.trim().trim_matches('"');
                if !id.is_empty() {
                    return Some(id.to_string());
                }
            }
        } else if let Some(rest) = trimmed.strip_prefix("--launch-profile=") {
            let id = rest.trim().trim_matches('"');
            if !id.is_empty() {
                return Some(id.to_string());
            }
        }
    }
    None
}

/// Sanitize an arbitrary profile name into a safe cross-platform Desktop filename stem
/// while preserving Vietnamese / Unicode characters and spaces.
pub fn sanitize_shortcut_name(raw_name: &str, fallback_id: &str) -> String {
    let cleaned: String = raw_name
        .trim()
        .chars()
        .map(|c| {
            if c.is_control() || matches!(c, '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*') {
                '_'
            } else {
                c
            }
        })
        .collect();

    // Windows forbids trailing spaces or dots on file/directory names.
    let trimmed = cleaned.trim().trim_end_matches(|c: char| c == '.' || c == ' ').trim();
    let stem = if trimmed.is_empty() {
        let short = &fallback_id[..fallback_id.len().min(8)];
        format!("LPBrower Profile {short}")
    } else {
        // Limit stem to 120 chars so Desktop path stays well below MAX_PATH.
        trimmed.chars().take(120).collect::<String>()
    };

    // Guard against Windows reserved DOS device names (CON, PRN, AUX, NUL, COM1..9, LPT1..9).
    let upper = stem.to_ascii_uppercase();
    let base_upper = upper.split('.').next().unwrap_or(&upper);
    let reserved = matches!(
        base_upper,
        "CON"
            | "PRN"
            | "AUX"
            | "NUL"
            | "COM1"
            | "COM2"
            | "COM3"
            | "COM4"
            | "COM5"
            | "COM6"
            | "COM7"
            | "COM8"
            | "COM9"
            | "LPT1"
            | "LPT2"
            | "LPT3"
            | "LPT4"
            | "LPT5"
            | "LPT6"
            | "LPT7"
            | "LPT8"
            | "LPT9"
    );
    if reserved {
        format!("LPB_{stem}")
    } else {
        stem
    }
}

/// Create a Desktop shortcut for `profile_id` and return the created shortcut path.
pub async fn create_desktop_shortcut(profile_id: &str) -> Result<PathBuf> {
    crate::space::ready().await;
    let profiles = tokio::task::spawn_blocking(crate::space::profile::list_all)
        .await
        .context("Không thể đọc danh sách hồ sơ")?
        .context("Không thể đọc danh sách hồ sơ")?;
    let meta = profiles
        .into_iter()
        .find(|p| p.id == profile_id)
        .ok_or_else(|| anyhow::anyhow!("Không tìm thấy hồ sơ ({profile_id})"))?;

    let desktop = dirs::desktop_dir().context("Không tìm thấy thư mục Desktop trên máy tính")?;
    let exe = std::env::current_exe().context("Không xác định được đường dẫn thực thi LPBrower")?;
    let cache_dir = crate::runtime::runtime_dir().or_else(|_| crate::store::profiles_dir())?;

    let id = meta.id.clone();
    let name = meta.name.clone();
    let taskbar_title = meta.taskbar_title.clone();
    let color = meta.color.clone();

    tokio::task::spawn_blocking(move || {
        std::fs::create_dir_all(&desktop).ok();
        let safe_name = sanitize_shortcut_name(&name, &id);
        create_platform_shortcut(
            &desktop,
            &safe_name,
            &exe,
            &cache_dir,
            &id,
            &name,
            &taskbar_title,
            color.as_deref(),
        )
    })
    .await
    .context("Tác vụ tạo shortcut thất bại")?
}

#[cfg(target_os = "windows")]
mod win_shell_link {
    use anyhow::{bail, Result};
    use std::ffi::{c_void, OsStr};
    use std::os::windows::ffi::OsStrExt;
    use std::path::Path;

    #[repr(C)]
    struct Guid {
        data1: u32,
        data2: u16,
        data3: u16,
        data4: [u8; 8],
    }

    const CLSID_SHELL_LINK: Guid = Guid {
        data1: 0x00021401,
        data2: 0x0000,
        data3: 0x0000,
        data4: [0xC0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46],
    };
    const IID_ISHELL_LINK_W: Guid = Guid {
        data1: 0x000214F9,
        data2: 0x0000,
        data3: 0x0000,
        data4: [0xC0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46],
    };
    const IID_IPERSIST_FILE: Guid = Guid {
        data1: 0x0000010B,
        data2: 0x0000,
        data3: 0x0000,
        data4: [0xC0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46],
    };

    const COINIT_APARTMENTTHREADED: u32 = 0x2;
    const CLSCTX_INPROC_SERVER: u32 = 0x1;
    const RPC_E_CHANGED_MODE: i32 = 0x80010106u32 as i32;

    #[link(name = "ole32")]
    extern "system" {
        fn CoInitializeEx(pv_reserved: *mut c_void, dw_co_init: u32) -> i32;
        fn CoUninitialize();
        fn CoCreateInstance(
            rclsid: *const Guid,
            p_unk_outer: *mut c_void,
            dw_cls_context: u32,
            riid: *const Guid,
            ppv: *mut *mut c_void,
        ) -> i32;
    }

    #[repr(C)]
    struct IShellLinkW {
        vtbl: *const IShellLinkWVtbl,
    }

    #[repr(C)]
    struct IShellLinkWVtbl {
        query_interface: unsafe extern "system" fn(*mut IShellLinkW, *const Guid, *mut *mut c_void) -> i32,
        add_ref: unsafe extern "system" fn(*mut IShellLinkW) -> u32,
        release: unsafe extern "system" fn(*mut IShellLinkW) -> u32,
        get_path: unsafe extern "system" fn(*mut IShellLinkW, *mut u16, i32, *mut c_void, u32) -> i32,
        get_id_list: unsafe extern "system" fn(*mut IShellLinkW, *mut *mut c_void) -> i32,
        set_id_list: unsafe extern "system" fn(*mut IShellLinkW, *const c_void) -> i32,
        get_description: unsafe extern "system" fn(*mut IShellLinkW, *mut u16, i32) -> i32,
        set_description: unsafe extern "system" fn(*mut IShellLinkW, *const u16) -> i32,
        get_working_directory: unsafe extern "system" fn(*mut IShellLinkW, *mut u16, i32) -> i32,
        set_working_directory: unsafe extern "system" fn(*mut IShellLinkW, *const u16) -> i32,
        get_arguments: unsafe extern "system" fn(*mut IShellLinkW, *mut u16, i32) -> i32,
        set_arguments: unsafe extern "system" fn(*mut IShellLinkW, *const u16) -> i32,
        get_hotkey: unsafe extern "system" fn(*mut IShellLinkW, *mut u16) -> i32,
        set_hotkey: unsafe extern "system" fn(*mut IShellLinkW, u16) -> i32,
        get_show_cmd: unsafe extern "system" fn(*mut IShellLinkW, *mut i32) -> i32,
        set_show_cmd: unsafe extern "system" fn(*mut IShellLinkW, i32) -> i32,
        get_icon_location: unsafe extern "system" fn(*mut IShellLinkW, *mut u16, i32, *mut i32) -> i32,
        set_icon_location: unsafe extern "system" fn(*mut IShellLinkW, *const u16, i32) -> i32,
        set_relative_path: unsafe extern "system" fn(*mut IShellLinkW, *const u16, u32) -> i32,
        resolve: unsafe extern "system" fn(*mut IShellLinkW, *mut c_void, u32) -> i32,
        set_path: unsafe extern "system" fn(*mut IShellLinkW, *const u16) -> i32,
    }

    #[repr(C)]
    struct IPersistFile {
        vtbl: *const IPersistFileVtbl,
    }

    #[repr(C)]
    struct IPersistFileVtbl {
        query_interface: unsafe extern "system" fn(*mut IPersistFile, *const Guid, *mut *mut c_void) -> i32,
        add_ref: unsafe extern "system" fn(*mut IPersistFile) -> u32,
        release: unsafe extern "system" fn(*mut IPersistFile) -> u32,
        get_class_id: unsafe extern "system" fn(*mut IPersistFile, *mut Guid) -> i32,
        is_dirty: unsafe extern "system" fn(*mut IPersistFile) -> i32,
        load: unsafe extern "system" fn(*mut IPersistFile, *const u16, u32) -> i32,
        save: unsafe extern "system" fn(*mut IPersistFile, *const u16, i32) -> i32,
        save_completed: unsafe extern "system" fn(*mut IPersistFile, *const u16) -> i32,
        get_cur_file: unsafe extern "system" fn(*mut IPersistFile, *mut *mut u16) -> i32,
    }

    fn to_wide(s: &OsStr) -> Vec<u16> {
        s.encode_wide().chain(std::iter::once(0)).collect()
    }

    struct ComGuard(bool);
    impl Drop for ComGuard {
        fn drop(&mut self) {
            if self.0 {
                unsafe { CoUninitialize() };
            }
        }
    }

    pub fn write_lnk(
        lnk_path: &Path,
        target_exe: &Path,
        args: &str,
        working_dir: &Path,
        icon_path: &Path,
        description: &str,
    ) -> Result<()> {
        unsafe {
            let hr_init = CoInitializeEx(std::ptr::null_mut(), COINIT_APARTMENTTHREADED);
            if hr_init < 0 && hr_init != RPC_E_CHANGED_MODE {
                bail!("CoInitializeEx failed: 0x{:08x}", hr_init as u32);
            }
            let _com = ComGuard(hr_init >= 0);

            let mut link_ptr: *mut c_void = std::ptr::null_mut();
            let hr = CoCreateInstance(
                &CLSID_SHELL_LINK,
                std::ptr::null_mut(),
                CLSCTX_INPROC_SERVER,
                &IID_ISHELL_LINK_W,
                &mut link_ptr,
            );
            if hr < 0 || link_ptr.is_null() {
                bail!("CoCreateInstance(IShellLinkW) failed: 0x{:08x}", hr as u32);
            }
            let link = link_ptr as *mut IShellLinkW;

            let w_target = to_wide(target_exe.as_os_str());
            let w_args = to_wide(OsStr::new(args));
            let w_workdir = to_wide(working_dir.as_os_str());
            let w_icon = to_wide(icon_path.as_os_str());
            let w_desc = to_wide(OsStr::new(description));
            let w_lnk = to_wide(lnk_path.as_os_str());

            let set_res = (|| -> Result<()> {
                let hr = ((*(*link).vtbl).set_path)(link, w_target.as_ptr());
                if hr < 0 {
                    bail!("IShellLinkW::SetPath failed: 0x{:08x}", hr as u32);
                }
                let hr = ((*(*link).vtbl).set_arguments)(link, w_args.as_ptr());
                if hr < 0 {
                    bail!("IShellLinkW::SetArguments failed: 0x{:08x}", hr as u32);
                }
                if !working_dir.as_os_str().is_empty() {
                    let _ = ((*(*link).vtbl).set_working_directory)(link, w_workdir.as_ptr());
                }
                let _ = ((*(*link).vtbl).set_icon_location)(link, w_icon.as_ptr(), 0);
                let _ = ((*(*link).vtbl).set_description)(link, w_desc.as_ptr());

                let mut pf_ptr: *mut c_void = std::ptr::null_mut();
                let hr = ((*(*link).vtbl).query_interface)(link, &IID_IPERSIST_FILE, &mut pf_ptr);
                if hr < 0 || pf_ptr.is_null() {
                    bail!("QueryInterface(IPersistFile) failed: 0x{:08x}", hr as u32);
                }
                let pf = pf_ptr as *mut IPersistFile;
                let save_hr = ((*(*pf).vtbl).save)(pf, w_lnk.as_ptr(), 1);
                ((*(*pf).vtbl).release)(pf);
                if save_hr < 0 {
                    bail!("IPersistFile::Save failed: 0x{:08x}", save_hr as u32);
                }
                Ok(())
            })();

            ((*(*link).vtbl).release)(link);
            set_res
        }
    }

    #[cfg(test)]
    pub fn read_lnk_args_and_icon(lnk_path: &Path) -> Result<(String, String)> {
        unsafe {
            let hr_init = CoInitializeEx(std::ptr::null_mut(), COINIT_APARTMENTTHREADED);
            let _com = ComGuard(hr_init >= 0);
            let mut link_ptr: *mut c_void = std::ptr::null_mut();
            let hr = CoCreateInstance(
                &CLSID_SHELL_LINK,
                std::ptr::null_mut(),
                CLSCTX_INPROC_SERVER,
                &IID_ISHELL_LINK_W,
                &mut link_ptr,
            );
            if hr < 0 || link_ptr.is_null() {
                bail!("CoCreateInstance failed");
            }
            let link = link_ptr as *mut IShellLinkW;
            let mut pf_ptr: *mut c_void = std::ptr::null_mut();
            let hr = ((*(*link).vtbl).query_interface)(link, &IID_IPERSIST_FILE, &mut pf_ptr);
            if hr < 0 || pf_ptr.is_null() {
                ((*(*link).vtbl).release)(link);
                bail!("QI IPersistFile failed");
            }
            let pf = pf_ptr as *mut IPersistFile;
            let w_lnk = to_wide(lnk_path.as_os_str());
            let load_hr = ((*(*pf).vtbl).load)(pf, w_lnk.as_ptr(), 0);
            ((*(*pf).vtbl).release)(pf);
            if load_hr < 0 {
                ((*(*link).vtbl).release)(link);
                bail!("IPersistFile::Load failed: 0x{:08x}", load_hr as u32);
            }
            let mut args_buf = [0u16; 512];
            let mut icon_buf = [0u16; 512];
            let mut icon_idx: i32 = 0;
            let _ = ((*(*link).vtbl).get_arguments)(link, args_buf.as_mut_ptr(), args_buf.len() as i32);
            let _ = ((*(*link).vtbl).get_icon_location)(
                link,
                icon_buf.as_mut_ptr(),
                icon_buf.len() as i32,
                &mut icon_idx,
            );
            ((*(*link).vtbl).release)(link);
            let nul_trim = |b: &[u16]| {
                let end = b.iter().position(|&c| c == 0).unwrap_or(b.len());
                String::from_utf16_lossy(&b[..end])
            };
            Ok((nul_trim(&args_buf), nul_trim(&icon_buf)))
        }
    }
}

#[cfg(target_os = "windows")]
fn create_platform_shortcut(
    desktop: &Path,
    safe_name: &str,
    exe: &Path,
    cache_dir: &Path,
    profile_id: &str,
    profile_name: &str,
    taskbar_title: &str,
    color: Option<&str>,
) -> Result<PathBuf> {
    let ico_path =
        crate::profile_icon::ensure_ico(cache_dir, taskbar_title, profile_name, color)
            .context("Không thể tạo icon cho shortcut")?;
    let lnk_path = desktop.join(format!("{safe_name}.lnk"));
    let workdir = exe.parent().unwrap_or_else(|| Path::new(""));
    let args = format!("--launch-profile \"{profile_id}\"");
    let desc = format!("Mở hồ sơ trình duyệt LPBrower: {profile_name}");

    win_shell_link::write_lnk(&lnk_path, exe, &args, workdir, &ico_path, &desc)
        .context("Không thể lưu file shortcut (.lnk) trên Desktop")?;

    Ok(lnk_path)
}

#[cfg(target_os = "macos")]
fn create_platform_shortcut(
    desktop: &Path,
    safe_name: &str,
    exe: &Path,
    cache_dir: &Path,
    profile_id: &str,
    profile_name: &str,
    taskbar_title: &str,
    color: Option<&str>,
) -> Result<PathBuf> {
    use std::os::unix::fs::PermissionsExt;

    let app_dir = desktop.join(format!("{safe_name}.app"));
    let contents_dir = app_dir.join("Contents");
    let macos_dir = contents_dir.join("MacOS");
    let res_dir = contents_dir.join("Resources");
    std::fs::create_dir_all(&macos_dir).context("create .app MacOS dir")?;
    std::fs::create_dir_all(&res_dir).context("create .app Resources dir")?;

    let xml_escape = |s: &str| {
        s.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
    };
    let sh_escape = |s: &str| s.replace('\'', "'\\''");

    let plist = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleExecutable</key>
  <string>launch</string>
  <key>CFBundleIdentifier</key>
  <string>com.lpbrower.shortcut.{}</string>
  <key>CFBundleName</key>
  <string>{}</string>
  <key>CFBundleDisplayName</key>
  <string>{}</string>
  <key>CFBundlePackageType</key>
  <string>APPL</string>
  <key>CFBundleIconFile</key>
  <string>shortcut.png</string>
  <key>LSUIElement</key>
  <true/>
</dict>
</plist>
"#,
        xml_escape(profile_id),
        xml_escape(profile_name),
        xml_escape(profile_name),
    );
    std::fs::write(contents_dir.join("Info.plist"), plist).context("write Info.plist")?;

    if let Ok(png_path) =
        crate::profile_icon::ensure_icon(cache_dir, taskbar_title, profile_name, color)
    {
        let _ = std::fs::copy(&png_path, res_dir.join("shortcut.png"));
    }

    let script = format!(
        "#!/bin/sh\nexec '{}' --launch-profile '{}' >/dev/null 2>&1 &\n",
        sh_escape(&exe.display().to_string()),
        sh_escape(profile_id)
    );
    let launch_bin = macos_dir.join("launch");
    std::fs::write(&launch_bin, script).context("write launcher script")?;
    std::fs::set_permissions(&launch_bin, std::fs::Permissions::from_mode(0o755))
        .context("chmod launcher script")?;

    Ok(app_dir)
}

#[cfg(all(unix, not(target_os = "macos")))]
fn create_platform_shortcut(
    desktop: &Path,
    safe_name: &str,
    exe: &Path,
    cache_dir: &Path,
    profile_id: &str,
    profile_name: &str,
    taskbar_title: &str,
    color: Option<&str>,
) -> Result<PathBuf> {
    use std::os::unix::fs::PermissionsExt;

    let icon_path =
        crate::profile_icon::ensure_icon(cache_dir, taskbar_title, profile_name, color)?;
    let desktop_file = desktop.join(format!("{safe_name}.desktop"));
    let body = format!(
        "[Desktop Entry]\nType=Application\nName={}\nComment=LPBrower Profile\nExec=\"{}\" --launch-profile \"{}\"\nIcon={}\nTerminal=false\nCategories=Network;WebBrowser;\n",
        profile_name.replace('\n', " "),
        exe.display(),
        profile_id,
        icon_path.display()
    );
    std::fs::write(&desktop_file, body).context("write .desktop shortcut")?;
    std::fs::set_permissions(&desktop_file, std::fs::Permissions::from_mode(0o755))
        .context("chmod .desktop shortcut")?;
    Ok(desktop_file)
}

#[cfg(test)]
mod tests {
    use super::{extract_launch_profile_arg, sanitize_shortcut_name};

    #[test]
    fn extracts_launch_profile_flag_variants() {
        let a = vec![
            "LPBrower.exe".to_string(),
            "--launch-profile".to_string(),
            "prof-123".to_string(),
        ];
        assert_eq!(extract_launch_profile_arg(&a).as_deref(), Some("prof-123"));

        let b = vec![
            "LPBrower.exe".to_string(),
            "--launch-profile=\"prof-456\"".to_string(),
        ];
        assert_eq!(extract_launch_profile_arg(&b).as_deref(), Some("prof-456"));

        let c = vec!["LPBrower.exe".to_string()];
        assert_eq!(extract_launch_profile_arg(&c), None);
    }

    #[test]
    fn sanitizes_windows_illegal_and_reserved_names() {
        assert_eq!(
            sanitize_shortcut_name("Tài khoản FB #1: VIP?", "id123456"),
            "Tài khoản FB #1_ VIP_"
        );
        assert_eq!(sanitize_shortcut_name("Shop...", "id123456"), "Shop");
        assert_eq!(sanitize_shortcut_name("CON", "id123456"), "LPB_CON");
        assert_eq!(sanitize_shortcut_name("   ", "abcdef123456"), "LPBrower Profile abcdef12");
    }

    #[test]
    fn generates_valid_multi_size_ico_with_and_without_taskbar_title() {
        let tmp = std::env::temp_dir().join(format!("lpb-ico-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();

        let plain = crate::profile_icon::ensure_ico(&tmp, "", "Profile 1", None).unwrap();
        let badged =
            crate::profile_icon::ensure_ico(&tmp, "FB-01", "Profile 1", Some("#2563eb")).unwrap();
        assert_ne!(plain, badged, "badged and plain icons must not collide");

        for p in [&plain, &badged] {
            let bytes = std::fs::read(p).unwrap();
            assert!(bytes.len() > 6 + 16 * 5);
            // ICONDIR header: reserved=0, type=1, count=5
            assert_eq!(&bytes[0..6], &[0, 0, 1, 0, 5, 0]);
            // Directory widths: 0 (256), 64, 48, 32, 16
            assert_eq!(bytes[6], 0);
            assert_eq!(bytes[6 + 16], 64);
            assert_eq!(bytes[6 + 32], 48);
            assert_eq!(bytes[6 + 48], 32);
            assert_eq!(bytes[6 + 64], 16);
            // First payload (256px) starts with PNG magic
            let off0 = u32::from_le_bytes(bytes[18..22].try_into().unwrap()) as usize;
            assert_eq!(&bytes[off0..off0 + 8], b"\x89PNG\r\n\x1a\n");
        }

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn creates_real_windows_lnk_with_unicode_name_and_custom_ico() {
        let tmp = std::env::temp_dir().join(format!("lpb-lnk-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();

        let exe = std::env::current_exe().unwrap();
        let safe_name = sanitize_shortcut_name("Tài khoản Shopee #01", "prof-vn-01");
        let lnk = super::create_platform_shortcut(
            &tmp,
            &safe_name,
            &exe,
            &tmp,
            "prof-vn-01",
            "Tài khoản Shopee #01",
            "SP-01",
            Some("#16a34a"),
        )
        .unwrap();

        assert!(lnk.is_file(), "expected .lnk file at {}", lnk.display());
        assert!(std::fs::metadata(&lnk).unwrap().len() > 64);
        let (saved_args, saved_icon) = super::win_shell_link::read_lnk_args_and_icon(&lnk).unwrap();
        assert_eq!(saved_args, "--launch-profile \"prof-vn-01\"");
        assert!(saved_icon.ends_with(".ico"), "unexpected icon: {saved_icon}");

        let _ = std::fs::remove_dir_all(&tmp);
    }
}
