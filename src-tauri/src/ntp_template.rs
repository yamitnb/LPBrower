//! Generates the branded LPBrowser New Tab Page (`chrome://newtab/`) HTML and JS payloads
//! for `resources.pak` (`chrome://new-tab-page/` and `chrome://new-tab-page-third-party/`).
//!
//! IMPORTANT:
//! 1. Never include the literal `$` character in HTML or JS strings here because
//!    Chromium's `WebUIDataSource` runs `ui::ReplaceTemplateExpressions` on WebUI resources.
//! 2. Never use `innerHTML`, `outerHTML`, `insertAdjacentHTML`, or `DOMParser` in JS because
//!    Chromium enforces strict Trusted Types on `chrome://new-tab-page/`. Construct all DOM
//!    and SVG nodes with `document.createElement` / `document.createElementNS`.

use base64::Engine;

const PNG_FAVICON: &[u8] = include_bytes!("../../public/lp-browser-32.png");
const PNG_MAIN: &[u8] = include_bytes!("../assets/ntp/app-main.png");
const PNG_TASKBAR: &[u8] = include_bytes!("../assets/ntp/taskbar.png");
const PNG_PROFILES: &[u8] = include_bytes!("../assets/ntp/multi-profiles.png");
const PNG_GHOST: &[u8] = include_bytes!("../assets/ntp/ghost-mode.png");
const PNG_PROXY: &[u8] = include_bytes!("../assets/ntp/proxy-ip.png");
const PNG_FP: &[u8] = include_bytes!("../assets/ntp/anti-fingerprint.png");
const PNG_COOKIE: &[u8] = include_bytes!("../assets/ntp/cookie-manager.png");
const PNG_UA: &[u8] = include_bytes!("../assets/ntp/user-agent.png");
const PNG_VAULT: &[u8] = include_bytes!("../assets/ntp/encryption-vault.png");

fn data_uri(png_bytes: &[u8]) -> String {
    let encoded = base64::engine::general_purpose::STANDARD.encode(png_bytes);
    format!("data:image/png;base64,{encoded}")
}

pub fn build_ntp_html(script_src: &str, marker_comment: &str) -> String {
    let ic_favicon = data_uri(PNG_FAVICON);
    let ic_main = data_uri(PNG_MAIN);
    let ic_taskbar = data_uri(PNG_TASKBAR);
    let ic_profiles = data_uri(PNG_PROFILES);
    let ic_ghost = data_uri(PNG_GHOST);
    let ic_proxy = data_uri(PNG_PROXY);
    let ic_fp = data_uri(PNG_FP);
    let ic_cookie = data_uri(PNG_COOKIE);
    let ic_ua = data_uri(PNG_UA);
    let ic_vault = data_uri(PNG_VAULT);

    let template = r##"<!doctype html>
<!-- __MARKER__ -->
<html lang="vi" data-theme="dark">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<meta name="color-scheme" content="dark light">
<link rel="icon" type="image/png" sizes="32x32" href="__IC_FAVICON__">
<link rel="shortcut icon" type="image/png" href="__IC_FAVICON__">
<title>Thẻ mới - LPBrowser</title>
<style>
/* Hallmark · pre-emit critique: P5 H5 E5 S5 R5 V5 */
:root {
  --font-sans: "Inter Variable", "Inter", "Segoe UI", system-ui, -apple-system, BlinkMacSystemFont, Roboto, sans-serif;
  --font-mono: "JetBrains Mono", ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  --bg: #0c1427;
  --bg-glow-a: rgba(34, 211, 238, 0.11);
  --bg-glow-b: rgba(168, 85, 247, 0.08);
  --surface: rgba(19, 30, 56, 0.84);
  --surface-hover: rgba(25, 40, 74, 0.96);
  --surface-modal: #111c36;
  --surface-pill: rgba(255, 255, 255, 0.045);
  --border: rgba(148, 184, 255, 0.13);
  --border-strong: rgba(56, 189, 248, 0.36);
  --ink: #f8fafc;
  --ink-2: #cbd5e1;
  --ink-3: #8597b8;
  --cyan: #22d3ee;
  --violet: #c084fc;
  --emerald: #34d399;
  --emerald-bg: rgba(16, 185, 129, 0.13);
  --danger: #f87171;
  --danger-bg: rgba(239, 68, 68, 0.15);
  --shadow-bar: 0 14px 36px -10px rgba(4, 9, 20, 0.62);
  --shadow-card: 0 6px 20px -6px rgba(4, 9, 20, 0.45);
  --shadow-modal: 0 24px 64px -12px rgba(2, 6, 18, 0.78);
  color-scheme: dark;
}
html[data-theme="light"] {
  --bg: #f4f7fc;
  --bg-glow-a: rgba(14, 165, 233, 0.08);
  --bg-glow-b: rgba(147, 51, 234, 0.05);
  --surface: #ffffff;
  --surface-hover: #f8fafc;
  --surface-modal: #ffffff;
  --surface-pill: #f1f5f9;
  --border: rgba(15, 23, 42, 0.09);
  --border-strong: rgba(2, 132, 199, 0.42);
  --ink: #0f172a;
  --ink-2: #334155;
  --ink-3: #64748b;
  --cyan: #0284c7;
  --violet: #7e22ce;
  --emerald: #059669;
  --emerald-bg: rgba(5, 150, 105, 0.10);
  --danger: #dc2626;
  --danger-bg: rgba(220, 38, 38, 0.10);
  --shadow-bar: 0 12px 28px -10px rgba(15, 23, 42, 0.10);
  --shadow-card: 0 4px 16px -4px rgba(15, 23, 42, 0.07);
  --shadow-modal: 0 22px 54px -14px rgba(15, 23, 42, 0.24);
  color-scheme: light;
}
html[data-theme="light"] .card,
html[data-theme="light"] .search-bar,
html[data-theme="light"] .sc-tile {
  backdrop-filter: none;
}
*, *::before, *::after { box-sizing: border-box; }
html, body {
  margin: 0;
  min-height: 100vh;
  overflow-x: clip;
  background-color: var(--bg);
  background-image:
    radial-gradient(780px 380px at 50% -40px, var(--bg-glow-a), transparent 72%),
    radial-gradient(620px 340px at 82% 8%, var(--bg-glow-b), transparent 70%);
  background-attachment: fixed;
  color: var(--ink);
  font-family: var(--font-sans);
  font-size: 13px;
  line-height: 1.5;
  -webkit-font-smoothing: antialiased;
}
.shell {
  min-height: 100vh;
  max-width: 1060px;
  margin-inline: auto;
  padding: 18px 28px 32px;
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  gap: 22px;
}
.topbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  flex-wrap: wrap;
}
.brand-chip {
  display: inline-flex;
  align-items: center;
  gap: 9px;
}
.brand-chip img {
  width: 26px;
  height: 26px;
  object-fit: contain;
  flex-shrink: 0;
}
.wordmark {
  font-size: 12.5px;
  font-weight: 700;
  font-style: normal;
  letter-spacing: 0.11em;
  text-transform: uppercase;
  color: var(--ink);
}
.status-pill {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 3px 10px;
  border-radius: 999px;
  background: var(--emerald-bg);
  border: 1px solid rgba(52, 211, 153, 0.24);
  color: var(--emerald);
  font-size: 11.5px;
  font-weight: 600;
}
.status-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--emerald);
  box-shadow: 0 0 8px var(--emerald);
}
.meta-group {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}
.meta-pill {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 4px 10px;
  border-radius: 8px;
  background: var(--surface-pill);
  border: 1px solid var(--border);
  color: var(--ink-2);
  font-family: var(--font-mono);
  font-size: 11.5px;
  font-variant-numeric: tabular-nums;
}
.theme-btn {
  appearance: none;
  cursor: pointer;
  padding: 4px 10px;
  border-radius: 8px;
  background: var(--surface-pill);
  border: 1px solid var(--border);
  color: var(--ink-2);
  font-family: var(--font-sans);
  font-size: 11.5px;
  font-weight: 500;
  transition: border-color 150ms ease, color 150ms ease, background 150ms ease;
}
.theme-btn:hover {
  border-color: var(--border-strong);
  color: var(--ink);
  background: var(--surface-hover);
}
.hero {
  width: 100%;
  max-width: 780px;
  margin-inline: auto;
  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;
  gap: 16px;
}
.hero-lockup {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 15px;
}
.hero-icon {
  width: 66px;
  height: 66px;
  object-fit: contain;
  flex-shrink: 0;
  filter: drop-shadow(0 10px 22px rgba(8, 15, 32, 0.55));
}
.hero-copy {
  text-align: left;
}
.hero-copy h1 {
  margin: 0;
  font-size: 28px;
  line-height: 1.12;
  font-weight: 800;
  font-style: normal;
  letter-spacing: -0.025em;
  color: var(--ink);
}
.lp-grad {
  background: linear-gradient(135deg, var(--cyan) 0%, var(--violet) 68%, var(--emerald) 100%);
  -webkit-background-clip: text;
  background-clip: text;
  color: transparent;
}
.hero-sub {
  margin: 4px 0 0;
  font-size: 12.5px;
  color: var(--ink-3);
  letter-spacing: -0.005em;
}
.search-bar {
  width: 100%;
  max-width: 680px;
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 7px 6px 8px;
  border-radius: 14px;
  background: var(--surface);
  border: 1px solid var(--border);
  box-shadow: var(--shadow-bar);
  backdrop-filter: blur(14px);
  transition: border-color 160ms ease, box-shadow 160ms ease;
}
.search-bar:focus-within {
  border-color: var(--border-strong);
  box-shadow: var(--shadow-bar), 0 0 0 3px rgba(34, 211, 238, 0.14);
}
.engine-btn {
  appearance: none;
  cursor: pointer;
  flex-shrink: 0;
  padding: 7px 11px;
  border-radius: 9px;
  border: 1px solid var(--border);
  background: var(--surface-pill);
  color: var(--cyan);
  font-family: var(--font-sans);
  font-size: 12px;
  font-weight: 600;
  transition: background 140ms ease, border-color 140ms ease;
}
.engine-btn:hover {
  background: var(--surface-hover);
  border-color: var(--border-strong);
}
.search-input {
  flex: 1;
  min-width: 0;
  border: none;
  outline: none;
  background: transparent;
  color: var(--ink);
  font-family: var(--font-sans);
  font-size: 14px;
  padding: 8px 6px;
}
.search-input::placeholder {
  color: var(--ink-3);
}
.kbd-hint {
  flex-shrink: 0;
  padding: 2px 7px;
  border-radius: 6px;
  border: 1px solid var(--border);
  background: var(--surface-pill);
  color: var(--ink-3);
  font-family: var(--font-mono);
  font-size: 11px;
}
.search-submit {
  appearance: none;
  cursor: pointer;
  flex-shrink: 0;
  padding: 8px 16px;
  border-radius: 10px;
  border: none;
  background: linear-gradient(135deg, #06b6d4 0%, #6366f1 55%, #8b5cf6 100%);
  color: #ffffff;
  font-family: var(--font-sans);
  font-size: 12.5px;
  font-weight: 600;
  transition: filter 140ms ease, transform 120ms ease;
}
.search-submit:hover { filter: brightness(1.09); }
.search-submit:active { transform: scale(0.98); }

/* Frequently Used Websites (Shortcuts Dock with Add / Edit / Delete) */
.sc-section {
  width: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
}
.sc-grid {
  width: 100%;
  display: flex;
  flex-wrap: wrap;
  justify-content: center;
  gap: 10px;
}
.sc-tile {
  position: relative;
  width: 98px;
  border-radius: 14px;
  background: var(--surface);
  border: 1px solid var(--border);
  box-shadow: var(--shadow-card);
  backdrop-filter: blur(10px);
  transition: border-color 150ms ease, transform 150ms ease;
}
.sc-tile:hover,
.sc-tile:focus-within {
  border-color: var(--border-strong);
  background: var(--surface-hover);
  transform: translateY(-2px);
}
.sc-link {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  padding: 13px 8px 10px;
  text-decoration: none;
  color: var(--ink);
  border-radius: 14px;
  outline: none;
}
.sc-link:focus-visible {
  box-shadow: 0 0 0 2px var(--cyan);
}
.sc-icon-wrap {
  width: 44px;
  height: 44px;
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  box-shadow: 0 6px 14px -4px rgba(2, 6, 23, 0.36);
  overflow: hidden;
}
.sc-icon-wrap svg {
  width: 44px;
  height: 44px;
  display: block;
}
.sc-label {
  max-width: 82px;
  font-size: 12px;
  font-weight: 600;
  line-height: 1.25;
  color: var(--ink-2);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.sc-tile:hover .sc-label {
  color: var(--ink);
}
.sc-actions {
  position: absolute;
  top: 5px;
  right: 5px;
  display: flex;
  align-items: center;
  gap: 3px;
  opacity: 0;
  pointer-events: none;
  transition: opacity 130ms ease;
}
.sc-tile:hover .sc-actions,
.sc-tile:focus-within .sc-actions {
  opacity: 1;
  pointer-events: auto;
}
.sc-act-btn {
  appearance: none;
  border: 1px solid var(--border);
  background: var(--surface-modal);
  color: var(--ink-2);
  width: 21px;
  height: 21px;
  padding: 0;
  border-radius: 6px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: background 120ms ease, color 120ms ease, border-color 120ms ease;
}
.sc-act-btn svg {
  width: 12px;
  height: 12px;
  display: block;
}
.sc-act-btn:hover {
  border-color: var(--border-strong);
  color: var(--ink);
  background: var(--surface-hover);
}
.sc-act-btn--del:hover {
  border-color: rgba(239, 68, 68, 0.45);
  color: var(--danger);
  background: var(--danger-bg);
}
.sc-add-btn {
  appearance: none;
  cursor: pointer;
  width: 98px;
  padding: 13px 8px 10px;
  border-radius: 14px;
  background: var(--surface-pill);
  border: 1px dashed var(--border);
  color: var(--ink-3);
  font-family: var(--font-sans);
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  transition: border-color 150ms ease, background 150ms ease, color 150ms ease, transform 150ms ease;
}
.sc-add-btn:hover {
  border-color: var(--border-strong);
  background: var(--surface-hover);
  color: var(--ink);
  transform: translateY(-2px);
}
.sc-add-circle {
  width: 44px;
  height: 44px;
  border-radius: 12px;
  border: 1px solid var(--border);
  background: var(--surface);
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--cyan);
}
.sc-add-circle svg {
  width: 20px;
  height: 20px;
  display: block;
}
.sc-reset-btn {
  appearance: none;
  border: none;
  background: transparent;
  color: var(--ink-3);
  font-family: var(--font-sans);
  font-size: 11.5px;
  cursor: pointer;
  padding: 2px 8px;
  border-radius: 6px;
  transition: color 130ms ease, background 130ms ease;
}
.sc-reset-btn:hover {
  color: var(--cyan);
  background: var(--surface-pill);
}

/* Shortcut Add/Edit Modal */
.sc-backdrop {
  position: fixed;
  inset: 0;
  z-index: 90;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 20px;
  background: rgba(5, 10, 22, 0.62);
  backdrop-filter: blur(6px);
}
.sc-backdrop[hidden] {
  display: none;
}
.sc-modal {
  width: 100%;
  max-width: 410px;
  border-radius: 16px;
  background: var(--surface-modal);
  border: 1px solid var(--border-strong);
  box-shadow: var(--shadow-modal);
  padding: 20px 20px 18px;
  text-align: left;
  display: flex;
  flex-direction: column;
  gap: 14px;
}
.sc-modal-head {
  display: flex;
  align-items: center;
  gap: 12px;
}
.sc-preview {
  width: 44px;
  height: 44px;
  border-radius: 12px;
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
  box-shadow: 0 6px 14px -4px rgba(2, 6, 23, 0.35);
}
.sc-preview svg {
  width: 44px;
  height: 44px;
  display: block;
}
.sc-modal-title-wrap {
  min-width: 0;
  flex: 1;
}
.sc-modal-title {
  margin: 0;
  font-size: 15px;
  font-weight: 700;
  color: var(--ink);
}
.sc-modal-sub {
  margin: 2px 0 0;
  font-size: 11.5px;
  color: var(--ink-3);
}
.sc-field {
  display: flex;
  flex-direction: column;
  gap: 5px;
}
.sc-field label {
  font-size: 12px;
  font-weight: 600;
  color: var(--ink-2);
}
.sc-field input {
  width: 100%;
  padding: 9px 11px;
  border-radius: 10px;
  border: 1px solid var(--border);
  background: var(--surface-pill);
  color: var(--ink);
  font-family: var(--font-sans);
  font-size: 13px;
  outline: none;
  transition: border-color 140ms ease, box-shadow 140ms ease;
}
.sc-field input:focus {
  border-color: var(--border-strong);
  box-shadow: 0 0 0 3px rgba(34, 211, 238, 0.14);
}
.sc-error {
  margin: 0;
  font-size: 12px;
  color: var(--danger);
  min-height: 16px;
}
.sc-modal-foot {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  margin-top: 2px;
}
.sc-foot-right {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-left: auto;
}
.sc-btn {
  appearance: none;
  cursor: pointer;
  padding: 8px 14px;
  border-radius: 9px;
  border: 1px solid var(--border);
  background: var(--surface-pill);
  color: var(--ink-2);
  font-family: var(--font-sans);
  font-size: 12.5px;
  font-weight: 600;
  transition: background 130ms ease, border-color 130ms ease, color 130ms ease, filter 130ms ease;
}
.sc-btn:hover {
  border-color: var(--border-strong);
  background: var(--surface-hover);
  color: var(--ink);
}
.sc-btn--primary {
  border: none;
  background: linear-gradient(135deg, #06b6d4 0%, #6366f1 55%, #8b5cf6 100%);
  color: #ffffff;
}
.sc-btn--primary:hover {
  filter: brightness(1.09);
  color: #ffffff;
}
.sc-btn--danger {
  border-color: rgba(239, 68, 68, 0.35);
  background: var(--danger-bg);
  color: var(--danger);
}
.sc-btn--danger:hover {
  border-color: rgba(239, 68, 68, 0.6);
  background: rgba(239, 68, 68, 0.22);
  color: var(--danger);
}

.quick-links {
  display: flex;
  align-items: center;
  justify-content: center;
  flex-wrap: wrap;
  gap: 8px;
}
.qlink {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 4px 11px;
  border-radius: 999px;
  background: var(--surface-pill);
  border: 1px solid var(--border);
  color: var(--ink-2);
  font-size: 11.5px;
  font-weight: 500;
  text-decoration: none;
  transition: border-color 140ms ease, color 140ms ease, background 140ms ease;
}
.qlink:hover {
  border-color: var(--border-strong);
  background: var(--surface-hover);
  color: var(--ink);
}
.qdot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--cyan);
}
.qdot--v { background: var(--violet); }
.qdot--e { background: var(--emerald); }

.matrix-wrap {
  width: 100%;
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.matrix-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding-inline: 2px;
}
.matrix-title {
  margin: 0;
  font-size: 11px;
  font-weight: 700;
  font-style: normal;
  letter-spacing: 0.09em;
  text-transform: uppercase;
  color: var(--ink-3);
}
.matrix-summary {
  font-family: var(--font-mono);
  font-size: 11px;
  color: var(--ink-3);
  font-variant-numeric: tabular-nums;
}
.matrix {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 11px;
}
@media (max-width: 920px) {
  .matrix { grid-template-columns: repeat(2, minmax(0, 1fr)); }
}
@media (max-width: 540px) {
  .shell { padding: 16px; }
  .hero-lockup { flex-direction: column; text-align: center; }
  .hero-copy { text-align: center; }
  .matrix { grid-template-columns: minmax(0, 1fr); }
  .kbd-hint { display: none; }
}
.card {
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding: 13px 14px;
  border-radius: 13px;
  background: var(--surface);
  border: 1px solid var(--border);
  box-shadow: var(--shadow-card);
  backdrop-filter: blur(10px);
  transition: border-color 160ms ease, transform 160ms ease;
}
.card:hover {
  border-color: var(--border-strong);
  background: var(--surface-hover);
  transform: translateY(-1px);
}
.card-top {
  display: flex;
  align-items: center;
  gap: 11px;
  min-width: 0;
}
.card-icon {
  width: 38px;
  height: 38px;
  object-fit: contain;
  flex-shrink: 0;
}
.card-titles {
  min-width: 0;
  flex: 1;
}
.card-titles h2 {
  margin: 0;
  font-size: 13px;
  line-height: 1.25;
  font-weight: 700;
  font-style: normal;
  letter-spacing: -0.01em;
  color: var(--ink);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.card-sub {
  margin: 2px 0 0;
  font-size: 11.5px;
  line-height: 1.3;
  color: var(--ink-3);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.card-val {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 6px;
  padding: 5px 9px;
  border-radius: 8px;
  background: var(--surface-pill);
  border: 1px solid var(--border);
  font-family: var(--font-mono);
  font-size: 11px;
  color: var(--ink-2);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  font-variant-numeric: tabular-nums;
}
.foot {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  flex-wrap: wrap;
  font-size: 11.5px;
  color: var(--ink-3);
  padding-inline: 2px;
}
.foot a {
  color: var(--ink-3);
  text-decoration: none;
  transition: color 140ms ease;
}
.foot a:hover {
  color: var(--cyan);
}
</style>
</head>
<body>
<main class="shell">
  <header class="topbar">
    <div class="brand-chip">
      <img src="__IC_TASKBAR__" alt="">
      <span class="wordmark">LPBrowser</span>
      <span class="status-pill"><span class="status-dot"></span>Phiên độc lập</span>
    </div>
    <div class="meta-group">
      <span class="meta-pill" id="lp-clock">--:-- · Local</span>
      <span class="meta-pill" id="lp-locale">--</span>
      <button type="button" class="theme-btn" id="lp-theme-btn" title="Chuyển chế độ Sáng / Tối">Sáng / Tối</button>
    </div>
  </header>

  <section class="hero" aria-label="Tìm kiếm và lối tắt truy cập nhanh">
    <div class="hero-lockup">
      <img class="hero-icon" src="__IC_MAIN__" alt="LPBrowser">
      <div class="hero-copy">
        <h1><span class="lp-grad">LP</span>Browser</h1>
        <p class="hero-sub">Trình duyệt đa hồ sơ chống nhận diện · Bảo mật danh tính &amp; kết nối độc lập</p>
      </div>
    </div>

    <form id="lp-search-form" class="search-bar" autocomplete="off">
      <button type="button" id="lp-engine-btn" class="engine-btn" title="Nhấp để đổi công cụ tìm kiếm">Google</button>
      <input
        id="lp-search-input"
        class="search-input"
        type="text"
        placeholder="Tìm kiếm Google hoặc nhập địa chỉ trang web..."
        autofocus
        spellcheck="false"
      >
      <kbd class="kbd-hint" aria-hidden="true">/</kbd>
      <button type="submit" class="search-submit">Truy cập</button>
    </form>

    <div class="sc-section" aria-label="Trang web thường dùng">
      <div id="lp-shortcuts-grid" class="sc-grid"></div>
      <button type="button" id="lp-sc-reset" class="sc-reset-btn" hidden>Khôi phục lối tắt mặc định</button>
    </div>

    <nav class="quick-links" aria-label="Công cụ kiểm tra nhanh">
      <a class="qlink" href="https://iphey.com"><span class="qdot qdot--e"></span>Kiểm tra Iphey</a>
      <a class="qlink" href="https://whoer.net"><span class="qdot"></span>Kiểm tra IP (Whoer)</a>
      <a class="qlink" href="https://browserleaks.com"><span class="qdot qdot--v"></span>BrowserLeaks</a>
      <a class="qlink" href="https://pixelscan.net"><span class="qdot"></span>Pixelscan</a>
    </nav>
  </section>

  <section class="matrix-wrap" aria-label="Hệ thống nhận diện và bảo vệ phiên">
    <div class="matrix-head">
      <h2 class="matrix-title">Hệ sinh thái bảo vệ LPBrowser</h2>
      <span class="matrix-summary" id="lp-env-summary">Đang đồng bộ thông số thiết bị...</span>
    </div>

    <div class="matrix">
      <article class="card">
        <div class="card-top">
          <img class="card-icon" src="__IC_PROFILES__" alt="">
          <div class="card-titles">
            <h2>Đa hồ sơ</h2>
            <p class="card-sub">Môi trường phiên tách biệt</p>
          </div>
        </div>
        <div class="card-val" id="stat-tz">Múi giờ: Tự động</div>
      </article>

      <article class="card">
        <div class="card-top">
          <img class="card-icon" src="__IC_PROXY__" alt="">
          <div class="card-titles">
            <h2>Proxy &amp; IP</h2>
            <p class="card-sub">Định tuyến &amp; chống rò UDP</p>
          </div>
        </div>
        <div class="card-val" id="stat-net">WebRTC Shield · Sẵn sàng</div>
      </article>

      <article class="card">
        <div class="card-top">
          <img class="card-icon" src="__IC_FP__" alt="">
          <div class="card-titles">
            <h2>Chống vân tay</h2>
            <p class="card-sub">Canvas · WebGL · Audio · Fonts</p>
          </div>
        </div>
        <div class="card-val" id="stat-fp">Đã ghim vân tay hồ sơ</div>
      </article>

      <article class="card">
        <div class="card-top">
          <img class="card-icon" src="__IC_UA__" alt="">
          <div class="card-titles">
            <h2>Giả lập thiết bị</h2>
            <p class="card-sub">User-Agent &amp; phần cứng</p>
          </div>
        </div>
        <div class="card-val" id="stat-hw">Đang kiểm tra...</div>
      </article>

      <article class="card">
        <div class="card-top">
          <img class="card-icon" src="__IC_COOKIE__" alt="">
          <div class="card-titles">
            <h2>Quản lý Cookie</h2>
            <p class="card-sub">Cookie · Cache · IndexedDB</p>
          </div>
        </div>
        <div class="card-val" id="stat-cookie">Lưu trữ cô lập riêng</div>
      </article>

      <article class="card">
        <div class="card-top">
          <img class="card-icon" src="__IC_GHOST__" alt="">
          <div class="card-titles">
            <h2>Chế độ Ẩn danh</h2>
            <p class="card-sub">Chống theo dõi chéo tài khoản</p>
          </div>
        </div>
        <div class="card-val" id="stat-ghost">Ngăn rò rỉ danh tính</div>
      </article>

      <article class="card">
        <div class="card-top">
          <img class="card-icon" src="__IC_VAULT__" alt="">
          <div class="card-titles">
            <h2>Bảo mật Vault</h2>
            <p class="card-sub">Mã hóa phiên &amp; mật khẩu</p>
          </div>
        </div>
        <div class="card-val" id="stat-vault">Khóa cục bộ an toàn</div>
      </article>

      <article class="card">
        <div class="card-top">
          <img class="card-icon" src="__IC_TASKBAR__" alt="">
          <div class="card-titles">
            <h2>LP Browser Core</h2>
            <p class="card-sub">Đa cửa sổ &amp; nhãn Taskbar</p>
          </div>
        </div>
        <div class="card-val" id="stat-core">Chromium Engine</div>
      </article>
    </div>
  </section>

  <footer class="foot">
    <span>LPBrowser Anti-Detect · Mỗi hồ sơ là một thiết bị độc lập</span>
    <a href="https://github.com/yamitnb/LPBrower">github.com/yamitnb/LPBrower</a>
  </footer>
</main>

<div id="lp-sc-backdrop" class="sc-backdrop" hidden role="dialog" aria-modal="true" aria-labelledby="lp-sc-modal-title">
  <form id="lp-sc-form" class="sc-modal" autocomplete="off">
    <div class="sc-modal-head">
      <div id="lp-sc-preview" class="sc-preview" aria-hidden="true"></div>
      <div class="sc-modal-title-wrap">
        <h2 id="lp-sc-modal-title" class="sc-modal-title">Thêm lối tắt mới</h2>
        <p class="sc-modal-sub">Tự động nhận diện icon cho các trang phổ biến</p>
      </div>
    </div>
    <div class="sc-field">
      <label for="lp-sc-name">Tên trang web</label>
      <input id="lp-sc-name" type="text" maxlength="32" placeholder="VD: Facebook, Threads, YouTube...">
    </div>
    <div class="sc-field">
      <label for="lp-sc-url">Địa chỉ URL</label>
      <input id="lp-sc-url" type="text" maxlength="512" placeholder="VD: https://www.facebook.com" required>
    </div>
    <p id="lp-sc-error" class="sc-error" role="alert"></p>
    <div class="sc-modal-foot">
      <button type="button" id="lp-sc-delete" class="sc-btn sc-btn--danger" hidden>Xóa</button>
      <div class="sc-foot-right">
        <button type="button" id="lp-sc-cancel" class="sc-btn">Hủy</button>
        <button type="submit" id="lp-sc-save" class="sc-btn sc-btn--primary">Lưu lối tắt</button>
      </div>
    </div>
  </form>
</div>

<script type="module" src="__SCRIPT_SRC__"></script>
</body>
</html>
"##;

    template
        .replace("__MARKER__", marker_comment)
        .replace("__SCRIPT_SRC__", script_src)
        .replace("__IC_FAVICON__", &ic_favicon)
        .replace("__IC_MAIN__", &ic_main)
        .replace("__IC_TASKBAR__", &ic_taskbar)
        .replace("__IC_PROFILES__", &ic_profiles)
        .replace("__IC_GHOST__", &ic_ghost)
        .replace("__IC_PROXY__", &ic_proxy)
        .replace("__IC_FP__", &ic_fp)
        .replace("__IC_COOKIE__", &ic_cookie)
        .replace("__IC_UA__", &ic_ua)
        .replace("__IC_VAULT__", &ic_vault)
}

pub fn build_ntp_js(marker_comment: &str) -> String {
    let js = r##"/* __MARKER__ */
(function () {
  var SVG_NS = "http://www.w3.org/2000/svg";
  var STORAGE_KEY = "lpb_ntp_shortcuts_v1";
  var ENGINES = [
    { name: "Google", url: "https://www.google.com/search?q=" },
    { name: "DuckDuckGo", url: "https://duckduckgo.com/?q=" },
    { name: "Bing", url: "https://www.bing.com/search?q=" }
  ];
  var DEFAULT_SHORTCUTS = [
    { id: "fb", name: "Facebook", url: "https://www.facebook.com" },
    { id: "threads", name: "Threads", url: "https://www.threads.net" },
    { id: "ig", name: "Instagram", url: "https://www.instagram.com" },
    { id: "gmail", name: "Gmail", url: "https://mail.google.com" }
  ];

  var root = document.documentElement;
  try {
    var savedTheme = localStorage.getItem("lpb_ntp_theme");
    if (savedTheme === "light" || savedTheme === "dark") {
      root.setAttribute("data-theme", savedTheme);
    }
  } catch (_e) {}

  var engineIdx = 0;
  try {
    var savedEng = Number(localStorage.getItem("lpb_ntp_engine"));
    if (savedEng >= 0 && savedEng < ENGINES.length) {
      engineIdx = savedEng;
    }
  } catch (_e) {}

  var engineBtn = document.getElementById("lp-engine-btn");
  var searchInput = document.getElementById("lp-search-input");
  var searchForm = document.getElementById("lp-search-form");
  var themeBtn = document.getElementById("lp-theme-btn");

  function syncEngineUi() {
    var eng = ENGINES[engineIdx];
    if (engineBtn) engineBtn.textContent = eng.name;
    if (searchInput) {
      searchInput.placeholder = "Tìm kiếm " + eng.name + " hoặc nhập địa chỉ trang web...";
    }
  }
  syncEngineUi();

  if (engineBtn) {
    engineBtn.addEventListener("click", function () {
      engineIdx = (engineIdx + 1) % ENGINES.length;
      try { localStorage.setItem("lpb_ntp_engine", String(engineIdx)); } catch (_e) {}
      syncEngineUi();
      if (searchInput) searchInput.focus();
    });
  }

  if (themeBtn) {
    themeBtn.addEventListener("click", function () {
      var next = root.getAttribute("data-theme") === "light" ? "dark" : "light";
      root.setAttribute("data-theme", next);
      try { localStorage.setItem("lpb_ntp_theme", next); } catch (_e) {}
    });
  }

  function resolveDestination(raw) {
    var q = raw.trim();
    if (!q) return null;
    if (/^(https?:\/\/|file:\/\/)/i.test(q)) {
      return q;
    }
    var noSpace = q.indexOf(" ") === -1;
    if (noSpace && /^(localhost(:\d+)?|\d{1,3}(\.\d{1,3}){3}(:\d+)?)(\/.*)?(?!.)/i.test(q)) {
      return "http://" + q;
    }
    if (noSpace && /^[a-z0-9-]+(\.[a-z0-9-]+)+([/:?#].*)?(?!.)/i.test(q)) {
      return "https://" + q;
    }
    return ENGINES[engineIdx].url + encodeURIComponent(q);
  }

  if (searchForm && searchInput) {
    searchForm.addEventListener("submit", function (ev) {
      ev.preventDefault();
      var dest = resolveDestination(searchInput.value);
      if (dest) {
        window.location.assign(dest);
      }
    });
  }

  // ---- Pure SVG helpers (100% Trusted Types compliant DOM construction) ----
  function svgNode(tag, attrs, children) {
    var el = document.createElementNS(SVG_NS, tag);
    if (attrs) {
      var keys = Object.keys(attrs);
      for (var i = 0; i < keys.length; i++) {
        el.setAttribute(keys[i], String(attrs[keys[i]]));
      }
    }
    if (children) {
      for (var j = 0; j < children.length; j++) {
        el.appendChild(children[j]);
      }
    }
    return el;
  }

  function iconPlus() {
    return svgNode("svg", { viewBox: "0 0 24 24", fill: "none", stroke: "currentColor", "stroke-width": "2.4", "stroke-linecap": "round" }, [
      svgNode("path", { d: "M12 5v14M5 12h14" })
    ]);
  }

  function iconEdit() {
    return svgNode("svg", { viewBox: "0 0 24 24", fill: "none", stroke: "currentColor", "stroke-width": "2.2", "stroke-linecap": "round", "stroke-linejoin": "round" }, [
      svgNode("path", { d: "M12 20h9" }),
      svgNode("path", { d: "M16.5 3.5a2.12 2.12 0 0 1 3 3L7 19l-4 1 1-4Z" })
    ]);
  }

  function iconClose() {
    return svgNode("svg", { viewBox: "0 0 24 24", fill: "none", stroke: "currentColor", "stroke-width": "2.4", "stroke-linecap": "round" }, [
      svgNode("path", { d: "M18 6L6 18M6 6l12 12" })
    ]);
  }

  var gradSeq = 0;

  function extractHost(rawUrl) {
    try {
      var u = new URL(normalizeShortcutUrl(rawUrl) || "https://example.com");
      return u.hostname.toLowerCase().replace(/^www\./, "");
    } catch (_e) {
      return "";
    }
  }

  function brandKeyFor(rawUrl) {
    var h = extractHost(rawUrl);
    if (!h) return "";
    if (h === "fb.com" || h.indexOf("facebook.com") !== -1) return "facebook";
    if (h.indexOf("threads.net") !== -1 || h.indexOf("threads.com") !== -1) return "threads";
    if (h.indexOf("instagram.com") !== -1 || h === "instagr.am") return "instagram";
    if (h.indexOf("mail.google.com") !== -1 || h.indexOf("gmail.com") !== -1) return "gmail";
    if (h.indexOf("youtube.com") !== -1 || h === "youtu.be") return "youtube";
    if (h === "x.com" || h.indexOf("twitter.com") !== -1) return "x";
    if (h.indexOf("tiktok.com") !== -1) return "tiktok";
    if (h.indexOf("telegram.org") !== -1 || h === "t.me") return "telegram";
    if (h.indexOf("zalo.me") !== -1 || h.indexOf("zalo.vn") !== -1) return "zalo";
    if (h.indexOf("shopee.") !== -1) return "shopee";
    if (h.indexOf("github.com") !== -1) return "github";
    if (h.indexOf("chatgpt.com") !== -1 || h.indexOf("openai.com") !== -1) return "chatgpt";
    if (h.indexOf("google.") !== -1) return "google";
    if (h.indexOf("linkedin.com") !== -1) return "linkedin";
    if (h.indexOf("discord.com") !== -1 || h === "discord.gg") return "discord";
    if (h.indexOf("whatsapp.com") !== -1) return "whatsapp";
    if (h.indexOf("reddit.com") !== -1) return "reddit";
    if (h.indexOf("canva.com") !== -1) return "canva";
    if (h.indexOf("netflix.com") !== -1) return "netflix";
    return "";
  }

  var MONOGRAM_COLORS = [
    ["#1d4ed8", "#3b82f6"],
    ["#0f766e", "#14b8a6"],
    ["#6d28d9", "#8b5cf6"],
    ["#be123c", "#f43f5e"],
    ["#c2410c", "#f97316"],
    ["#047857", "#10b981"],
    ["#0369a1", "#0ea5e9"],
    ["#4338ca", "#6366f1"]
  ];

  function hashStr(s) {
    var h = 2166136261;
    for (var i = 0; i < s.length; i++) {
      h ^= s.charCodeAt(i);
      h = Math.imul(h, 16777619);
    }
    return h >>> 0;
  }

  function buildBrandIcon(name, rawUrl) {
    var key = brandKeyFor(rawUrl);
    var gid = "lpg" + (++gradSeq);

    if (key === "facebook") {
      return svgNode("svg", { viewBox: "0 0 48 48" }, [
        svgNode("defs", {}, [
          svgNode("linearGradient", { id: gid, x1: "0", y1: "0", x2: "1", y2: "1" }, [
            svgNode("stop", { offset: "0%", "stop-color": "#1877F2" }),
            svgNode("stop", { offset: "100%", "stop-color": "#0C5DC7" })
          ])
        ]),
        svgNode("rect", { width: "48", height: "48", rx: "12", fill: "url(#" + gid + ")" }),
        svgNode("path", {
          fill: "#FFFFFF",
          d: "M31.4 30.8l1.1-7.2h-6.9v-4.7c0-2 1-3.9 4.1-3.9h3.2V8.9s-2.9-.5-5.6-.5c-5.8 0-9.5 3.5-9.5 9.8v5.4h-6.4v7.2h6.4V48h7.8V30.8h5.8z"
        })
      ]);
    }

    if (key === "threads") {
      return svgNode("svg", { viewBox: "0 0 48 48" }, [
        svgNode("rect", { width: "48", height: "48", rx: "12", fill: "#101218" }),
        svgNode("rect", { x: "0.75", y: "0.75", width: "46.5", height: "46.5", rx: "11.25", fill: "none", stroke: "rgba(255,255,255,0.16)", "stroke-width": "1.5" }),
        svgNode("path", {
          fill: "none",
          stroke: "#FFFFFF",
          "stroke-width": "3.1",
          "stroke-linecap": "round",
          "stroke-linejoin": "round",
          d: "M33.2 21.8c-.4-6.4-4.2-10.3-10.1-10.3-6.6 0-11.2 4.9-11.2 12.5 0 7.7 4.6 12.6 11.4 12.6 4.7 0 8.2-2.1 10-5.4"
        }),
        svgNode("path", {
          fill: "none",
          stroke: "#FFFFFF",
          "stroke-width": "3.1",
          "stroke-linecap": "round",
          "stroke-linejoin": "round",
          d: "M19.8 19.6c1.1-1.3 2.7-2 4.6-2 3.2 0 5.4 2 5.4 5.5 0 4.2-2.4 7.3-6.1 7.3-2.6 0-4.4-1.5-4.4-3.7 0-2.4 1.9-3.9 5.2-3.9 2.2 0 4.3.5 6.1 1.6 2.7 1.6 4.2 3.8 4.2 6.5"
        })
      ]);
    }

    if (key === "instagram") {
      return svgNode("svg", { viewBox: "0 0 48 48" }, [
        svgNode("defs", {}, [
          svgNode("radialGradient", { id: gid, cx: "28%", cy: "105%", r: "130%" }, [
            svgNode("stop", { offset: "0%", "stop-color": "#FFDD55" }),
            svgNode("stop", { offset: "26%", "stop-color": "#FF543E" }),
            svgNode("stop", { offset: "62%", "stop-color": "#C837AB" }),
            svgNode("stop", { offset: "100%", "stop-color": "#4E54D8" })
          ])
        ]),
        svgNode("rect", { width: "48", height: "48", rx: "12", fill: "url(#" + gid + ")" }),
        svgNode("rect", { x: "11.5", y: "11.5", width: "25", height: "25", rx: "7", fill: "none", stroke: "#FFFFFF", "stroke-width": "3" }),
        svgNode("circle", { cx: "24", cy: "24", r: "6.2", fill: "none", stroke: "#FFFFFF", "stroke-width": "3" }),
        svgNode("circle", { cx: "31.5", cy: "16.5", r: "1.9", fill: "#FFFFFF" })
      ]);
    }

    if (key === "gmail") {
      return svgNode("svg", { viewBox: "0 0 48 48" }, [
        svgNode("rect", { width: "48", height: "48", rx: "12", fill: "#FFFFFF" }),
        svgNode("path", { fill: "#4285F4", d: "M9 35.5V20.2l6.2 4.7v11.6h-4.7c-.8 0-1.5-.7-1.5-1.5z" }),
        svgNode("path", { fill: "#34A853", d: "M32.8 36.5V24.9L39 20.2v15.3c0 .8-.7 1.5-1.5 1.5h-4.7z" }),
        svgNode("path", { fill: "#FBBC04", d: "M32.8 14.4v10.5L39 20.2v-3.5c0-2.8-3.2-4.4-5.4-2.7l-.8.4z" }),
        svgNode("path", { fill: "#EA4335", d: "M15.2 24.9V14.4L24 21l8.8-6.6v10.5L24 31.5l-8.8-6.6z" }),
        svgNode("path", { fill: "#C5221F", d: "M9 16.7v3.5l6.2 4.7V14.4l-.8-.4C12.2 12.3 9 13.9 9 16.7z" })
      ]);
    }

    if (key === "youtube") {
      return svgNode("svg", { viewBox: "0 0 48 48" }, [
        svgNode("rect", { width: "48", height: "48", rx: "12", fill: "#FF0000" }),
        svgNode("polygon", { points: "20,16 34,24 20,32", fill: "#FFFFFF" })
      ]);
    }

    if (key === "x") {
      return svgNode("svg", { viewBox: "0 0 48 48" }, [
        svgNode("rect", { width: "48", height: "48", rx: "12", fill: "#0F1419" }),
        svgNode("rect", { x: "0.75", y: "0.75", width: "46.5", height: "46.5", rx: "11.25", fill: "none", stroke: "rgba(255,255,255,0.16)", "stroke-width": "1.5" }),
        svgNode("path", { fill: "#FFFFFF", d: "M27.2 21.8L37.4 10h-2.4l-8.9 10.3L19 10h-8.2l10.7 15.6L10.8 38h2.4l9.4-10.9L30.1 38h8.2L27.2 21.8zm-3.3 3.9l-1.1-1.6-8.6-12.3h3.7l7 10 1.1 1.6 9.1 13h-3.7l-7.4-10.7z" })
      ]);
    }

    if (key === "tiktok") {
      return svgNode("svg", { viewBox: "0 0 48 48" }, [
        svgNode("rect", { width: "48", height: "48", rx: "12", fill: "#0B0B0F" }),
        svgNode("path", { fill: "#25F4EE", d: "M27.8 11.5v16.8a5.8 5.8 0 1 1-4.6-5.7v4.1a1.8 1.8 0 1 0 1.3 1.7V11.5h3.3z" }),
        svgNode("path", { fill: "#FE2C55", d: "M29.4 13c1.1 2.7 3.5 4.6 6.4 4.9v3.6c-2.6-.1-4.9-1-6.4-2.3v9.7a7.3 7.3 0 1 1-6.4-7.2v3.7a3.6 3.6 0 1 0 2.7 3.5V13h3.7z" }),
        svgNode("path", { fill: "#FFFFFF", d: "M28.6 12.2c.9 2.7 3.3 4.7 6.3 5v3.5c-2.6-.1-4.8-1-6.3-2.3v9.6a7.1 7.1 0 1 1-6.2-7v3.7a3.4 3.4 0 1 0 2.6 3.3V12.2h3.6z" })
      ]);
    }

    if (key === "telegram") {
      return svgNode("svg", { viewBox: "0 0 48 48" }, [
        svgNode("rect", { width: "48", height: "48", rx: "12", fill: "#229ED9" }),
        svgNode("path", { fill: "#FFFFFF", d: "M36.8 13.4l-4.5 21.4c-.3 1.5-1.2 1.9-2.5 1.2l-6.9-5.1-3.3 3.2c-.4.4-.7.7-1.4.7l.5-7 12.8-11.6c.6-.5-.1-.8-.9-.3L14.8 26l-6.8-2.1c-1.5-.5-1.5-1.5.3-2.2l26.6-10.3c1.2-.4 2.3.3 1.9 2z" })
      ]);
    }

    if (key === "zalo") {
      var zaloTxt = svgNode("text", { x: "24", y: "30", "text-anchor": "middle", fill: "#FFFFFF", "font-family": "Segoe UI, system-ui, sans-serif", "font-size": "15", "font-weight": "800" });
      zaloTxt.textContent = "Zalo";
      return svgNode("svg", { viewBox: "0 0 48 48" }, [
        svgNode("rect", { width: "48", height: "48", rx: "12", fill: "#0068FF" }),
        zaloTxt
      ]);
    }

    if (key === "shopee") {
      var spTxt = svgNode("text", { x: "24", y: "34", "text-anchor": "middle", fill: "#EE4D2D", "font-family": "Segoe UI, system-ui, sans-serif", "font-size": "17", "font-weight": "800" });
      spTxt.textContent = "S";
      return svgNode("svg", { viewBox: "0 0 48 48" }, [
        svgNode("rect", { width: "48", height: "48", rx: "12", fill: "#EE4D2D" }),
        svgNode("path", { fill: "none", stroke: "#FFFFFF", "stroke-width": "2.6", d: "M18.5 18v-2.2a5.5 5.5 0 0 1 11 0V18" }),
        svgNode("rect", { x: "12", y: "17.5", width: "24", height: "20.5", rx: "4", fill: "#FFFFFF" }),
        spTxt
      ]);
    }

    if (key === "google") {
      return svgNode("svg", { viewBox: "0 0 48 48" }, [
        svgNode("rect", { width: "48", height: "48", rx: "12", fill: "#FFFFFF" }),
        svgNode("path", { fill: "#EA4335", d: "M24 16.2c2.2 0 4.2.8 5.7 2.2l4.3-4.3C31.4 11.7 27.9 10 24 10c-5.5 0-10.2 3.1-12.5 7.7l5 3.9c1.2-3.2 4.1-5.4 7.5-5.4z" }),
        svgNode("path", { fill: "#4285F4", d: "M37.4 24.3c0-1-.1-2-.3-2.9H24v5.6h7.5c-.3 1.7-1.3 3.2-2.8 4.2l4.5 3.5c2.6-2.4 4.2-6 4.2-10.4z" }),
        svgNode("path", { fill: "#FBBC05", d: "M16.5 26.4c-.3-.8-.5-1.6-.5-2.4s.2-1.6.5-2.4l-5-3.9C10.5 19.6 10 21.7 10 24s.5 4.4 1.5 6.3l5-3.9z" }),
        svgNode("path", { fill: "#34A853", d: "M24 38c3.8 0 7-1.3 9.3-3.4l-4.5-3.5c-1.3.9-2.9 1.4-4.8 1.4-3.4 0-6.3-2.2-7.5-5.4l-5 3.9C13.8 34.9 18.5 38 24 38z" })
      ]);
    }

    if (key === "github") {
      return svgNode("svg", { viewBox: "0 0 48 48" }, [
        svgNode("rect", { width: "48", height: "48", rx: "12", fill: "#181717" }),
        svgNode("path", { fill: "#FFFFFF", d: "M24 10c-7.7 0-14 6.3-14 14 0 6.2 4 11.4 9.6 13.3.7.1 1-.3 1-.7v-2.4c-3.9.8-4.7-1.9-4.7-1.9-.6-1.6-1.6-2-1.6-2-1.3-.9.1-.9.1-.9 1.4.1 2.2 1.5 2.2 1.5 1.3 2.2 3.3 1.5 4.1 1.2.1-.9.5-1.5.9-1.9-3.1-.4-6.4-1.6-6.4-6.9 0-1.5.5-2.8 1.4-3.8-.1-.4-.6-1.8.1-3.7 0 0 1.2-.4 3.9 1.4 1.1-.3 2.3-.5 3.5-.5s2.4.2 3.5.5c2.7-1.8 3.9-1.4 3.9-1.4.8 1.9.3 3.3.1 3.7.9 1 1.4 2.2 1.4 3.8 0 5.4-3.3 6.5-6.4 6.9.5.4 1 1.3 1 2.6v3.8c0 .4.3.8 1 .7A14 14 0 0 0 38 24c0-7.7-6.3-14-14-14z" })
      ]);
    }

    if (key === "chatgpt") {
      return svgNode("svg", { viewBox: "0 0 48 48" }, [
        svgNode("rect", { width: "48", height: "48", rx: "12", fill: "#10A37F" }),
        svgNode("circle", { cx: "24", cy: "24", r: "10", fill: "none", stroke: "#FFFFFF", "stroke-width": "2.6" }),
        svgNode("path", { fill: "none", stroke: "#FFFFFF", "stroke-width": "2.4", "stroke-linecap": "round", d: "M24 14v20M15.3 19l17.4 10M32.7 19L15.3 29" })
      ]);
    }

    if (key === "linkedin") {
      var liTxt = svgNode("text", { x: "24", y: "31", "text-anchor": "middle", fill: "#FFFFFF", "font-family": "Segoe UI, system-ui, sans-serif", "font-size": "21", "font-weight": "800" });
      liTxt.textContent = "in";
      return svgNode("svg", { viewBox: "0 0 48 48" }, [
        svgNode("rect", { width: "48", height: "48", rx: "12", fill: "#0A66C2" }),
        liTxt
      ]);
    }

    if (key === "discord") {
      return svgNode("svg", { viewBox: "0 0 48 48" }, [
        svgNode("rect", { width: "48", height: "48", rx: "12", fill: "#5865F2" }),
        svgNode("path", { fill: "#FFFFFF", d: "M33.6 15.6a22 22 0 0 0-5.4-1.7l-.7 1.4a20 20 0 0 0-7 0l-.7-1.4a22 22 0 0 0-5.4 1.7c-3.4 5.1-4.3 10.1-3.9 15a22 22 0 0 0 6.6 3.3l1.4-2.3a14 14 0 0 1-2.2-1.1l.5-.4a15.4 15.4 0 0 0 14.4 0l.5.4a14 14 0 0 1-2.2 1.1l1.4 2.3a22 22 0 0 0 6.6-3.3c.5-5.7-.9-10.6-3.9-15z" }),
        svgNode("circle", { cx: "19.5", cy: "24.6", r: "2.3", fill: "#5865F2" }),
        svgNode("circle", { cx: "28.5", cy: "24.6", r: "2.3", fill: "#5865F2" })
      ]);
    }

    if (key === "whatsapp") {
      return svgNode("svg", { viewBox: "0 0 48 48" }, [
        svgNode("rect", { width: "48", height: "48", rx: "12", fill: "#25D366" }),
        svgNode("path", { fill: "none", stroke: "#FFFFFF", "stroke-width": "2.8", "stroke-linejoin": "round", d: "M24 11.5a12 12 0 0 0-10.3 18.2L12 36l6.5-1.7A12 12 0 1 0 24 11.5z" }),
        svgNode("path", { fill: "#FFFFFF", d: "M20.3 18.8c-.4-.8-.8-.8-1.2-.8h-1c-.4 0-1 .2-1.4.7-.5.6-1.8 1.8-1.8 4.3s1.8 5 2.1 5.3c.3.4 3.5 5.6 8.7 7.6 4.3 1.7 5.2 1.4 6.1 1.3.9-.1 3-1.2 3.4-2.4.4-1.2.4-2.2.3-2.4-.1-.2-.5-.4-1-.6l-3.1-1.5c-.4-.2-.8-.3-1.1.2-.3.5-1.3 1.6-1.6 1.9-.3.3-.6.4-1.1.1-2.4-1.1-4.2-2.6-5.7-5-.4-.6.1-.9.4-1.3l.9-1c.2-.3.3-.6.1-1l-1.5-3.4z" })
      ]);
    }

    if (key === "reddit") {
      return svgNode("svg", { viewBox: "0 0 48 48" }, [
        svgNode("rect", { width: "48", height: "48", rx: "12", fill: "#FF4500" }),
        svgNode("ellipse", { cx: "24", cy: "27.5", rx: "11.5", ry: "8.2", fill: "#FFFFFF" }),
        svgNode("circle", { cx: "19.8", cy: "26.5", r: "1.9", fill: "#FF4500" }),
        svgNode("circle", { cx: "28.2", cy: "26.5", r: "1.9", fill: "#FF4500" }),
        svgNode("path", { fill: "none", stroke: "#FFFFFF", "stroke-width": "2.4", "stroke-linecap": "round", d: "M24 19.3l2.2-6.3 5.2 1.2" }),
        svgNode("circle", { cx: "32.5", cy: "14.4", r: "2.2", fill: "#FFFFFF" })
      ]);
    }

    if (key === "canva") {
      var cvTxt = svgNode("text", { x: "24", y: "31", "text-anchor": "middle", fill: "#FFFFFF", "font-family": "Segoe UI, system-ui, sans-serif", "font-size": "21", "font-weight": "800" });
      cvTxt.textContent = "C";
      return svgNode("svg", { viewBox: "0 0 48 48" }, [
        svgNode("defs", {}, [
          svgNode("linearGradient", { id: gid, x1: "0", y1: "0", x2: "1", y2: "1" }, [
            svgNode("stop", { offset: "0%", "stop-color": "#00C4CC" }),
            svgNode("stop", { offset: "100%", "stop-color": "#7D2AE8" })
          ])
        ]),
        svgNode("rect", { width: "48", height: "48", rx: "12", fill: "url(#" + gid + ")" }),
        cvTxt
      ]);
    }

    if (key === "netflix") {
      var nfTxt = svgNode("text", { x: "24", y: "33", "text-anchor": "middle", fill: "#E50914", "font-family": "Segoe UI, system-ui, sans-serif", "font-size": "26", "font-weight": "900" });
      nfTxt.textContent = "N";
      return svgNode("svg", { viewBox: "0 0 48 48" }, [
        svgNode("rect", { width: "48", height: "48", rx: "12", fill: "#141414" }),
        nfTxt
      ]);
    }

    // Crisp deterministic monogram icon for any custom website
    var host = extractHost(rawUrl);
    var seed = host || name || "W";
    var pal = MONOGRAM_COLORS[hashStr(seed) % MONOGRAM_COLORS.length];
    var cleanLabel = (name || host || "W").trim();
    var letters = cleanLabel.replace(/[^a-zA-Z0-9\u00C0-\u024F\u1E00-\u1EFF]/g, "").slice(0, 2).toUpperCase() || "W";
    var monoTxt = svgNode("text", {
      x: "24",
      y: "30.5",
      "text-anchor": "middle",
      fill: "#FFFFFF",
      "font-family": "Segoe UI, system-ui, sans-serif",
      "font-size": letters.length > 1 ? "17" : "21",
      "font-weight": "800"
    });
    monoTxt.textContent = letters;

    return svgNode("svg", { viewBox: "0 0 48 48" }, [
      svgNode("defs", {}, [
        svgNode("linearGradient", { id: gid, x1: "0", y1: "0", x2: "1", y2: "1" }, [
          svgNode("stop", { offset: "0%", "stop-color": pal[0] }),
          svgNode("stop", { offset: "100%", "stop-color": pal[1] })
        ])
      ]),
      svgNode("rect", { width: "48", height: "48", rx: "12", fill: "url(#" + gid + ")" }),
      monoTxt
    ]);
  }

  // ---- Shortcut CRUD (Add / Edit / Delete + localStorage persistence) ----
  function normalizeShortcutUrl(raw) {
    var s = String(raw || "").trim();
    if (!s) return null;
    if (/^(javascript|data|vbscript):/i.test(s)) return null;
    if (!/^https?:\/\//i.test(s)) {
      s = "https://" + s;
    }
    try {
      var parsed = new URL(s);
      if (parsed.protocol !== "http:" && parsed.protocol !== "https:") return null;
      if (!parsed.hostname || parsed.hostname.indexOf(".") === -1 && parsed.hostname !== "localhost") return null;
      return parsed.href;
    } catch (_e) {
      return null;
    }
  }

  function inferSiteName(url) {
    var h = extractHost(url);
    if (!h) return "Website";
    var base = h.split(".")[0] || h;
    return base.charAt(0).toUpperCase() + base.slice(1);
  }

  function cloneDefaults() {
    return DEFAULT_SHORTCUTS.map(function (item) {
      return { id: item.id, name: item.name, url: item.url };
    });
  }

  function loadShortcuts() {
    try {
      var raw = localStorage.getItem(STORAGE_KEY);
      if (!raw) return cloneDefaults();
      var parsed = JSON.parse(raw);
      if (!Array.isArray(parsed)) return cloneDefaults();
      var valid = [];
      for (var i = 0; i < parsed.length; i++) {
        var it = parsed[i];
        if (!it || typeof it.url !== "string") continue;
        var norm = normalizeShortcutUrl(it.url);
        if (!norm) continue;
        valid.push({
          id: typeof it.id === "string" && it.id ? it.id : ("sc_" + i + "_" + Date.now()),
          name: typeof it.name === "string" && it.name.trim() ? it.name.trim() : inferSiteName(norm),
          url: norm
        });
      }
      return valid;
    } catch (_e) {
      return cloneDefaults();
    }
  }

  function saveShortcuts(list) {
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(list));
    } catch (_e) {}
  }

  function isDefaultList(list) {
    if (list.length !== DEFAULT_SHORTCUTS.length) return false;
    for (var i = 0; i < list.length; i++) {
      if (list[i].name !== DEFAULT_SHORTCUTS[i].name || list[i].url !== DEFAULT_SHORTCUTS[i].url) {
        return false;
      }
    }
    return true;
  }

  var shortcuts = loadShortcuts();
  var editingId = null;

  var gridEl = document.getElementById("lp-shortcuts-grid");
  var resetBtn = document.getElementById("lp-sc-reset");
  var backdropEl = document.getElementById("lp-sc-backdrop");
  var modalForm = document.getElementById("lp-sc-form");
  var modalTitle = document.getElementById("lp-sc-modal-title");
  var previewEl = document.getElementById("lp-sc-preview");
  var nameInput = document.getElementById("lp-sc-name");
  var urlInput = document.getElementById("lp-sc-url");
  var errorEl = document.getElementById("lp-sc-error");
  var deleteBtn = document.getElementById("lp-sc-delete");
  var cancelBtn = document.getElementById("lp-sc-cancel");

  function clearChildren(node) {
    while (node && node.firstChild) {
      node.removeChild(node.firstChild);
    }
  }

  function updateModalPreview() {
    if (!previewEl) return;
    clearChildren(previewEl);
    var nm = nameInput ? nameInput.value : "";
    var ur = urlInput ? urlInput.value : "";
    previewEl.appendChild(buildBrandIcon(nm || inferSiteName(ur), ur));
  }

  function openShortcutModal(item) {
    if (!backdropEl) return;
    editingId = item ? item.id : null;
    if (modalTitle) {
      modalTitle.textContent = item ? "Chỉnh sửa lối tắt" : "Thêm lối tắt mới";
    }
    if (nameInput) nameInput.value = item ? item.name : "";
    if (urlInput) urlInput.value = item ? item.url : "";
    if (errorEl) errorEl.textContent = "";
    if (deleteBtn) deleteBtn.hidden = !item;
    updateModalPreview();
    backdropEl.hidden = false;
    if (item && nameInput) {
      nameInput.focus();
      nameInput.select();
    } else if (urlInput) {
      urlInput.focus();
    }
  }

  function closeShortcutModal() {
    if (!backdropEl) return;
    backdropEl.hidden = true;
    editingId = null;
    if (errorEl) errorEl.textContent = "";
  }

  function removeShortcutById(id) {
    shortcuts = shortcuts.filter(function (s) { return s.id !== id; });
    saveShortcuts(shortcuts);
    renderShortcuts();
  }

  function renderShortcuts() {
    if (!gridEl) return;
    clearChildren(gridEl);

    for (var i = 0; i < shortcuts.length; i++) {
      (function (item) {
        var tile = document.createElement("div");
        tile.className = "sc-tile";
        tile.setAttribute("data-sc-id", item.id);

        var link = document.createElement("a");
        link.className = "sc-link";
        link.href = item.url;
        link.title = item.name + " (" + item.url + ")";

        var iconBox = document.createElement("span");
        iconBox.className = "sc-icon-wrap";
        iconBox.setAttribute("aria-hidden", "true");
        iconBox.appendChild(buildBrandIcon(item.name, item.url));

        var label = document.createElement("span");
        label.className = "sc-label";
        label.textContent = item.name;

        link.appendChild(iconBox);
        link.appendChild(label);
        tile.appendChild(link);

        var actions = document.createElement("div");
        actions.className = "sc-actions";

        var editBtn = document.createElement("button");
        editBtn.type = "button";
        editBtn.className = "sc-act-btn";
        editBtn.title = "Sửa lối tắt";
        editBtn.setAttribute("aria-label", "Sửa " + item.name);
        editBtn.appendChild(iconEdit());
        editBtn.addEventListener("click", function (ev) {
          ev.preventDefault();
          ev.stopPropagation();
          openShortcutModal(item);
        });

        var delBtn = document.createElement("button");
        delBtn.type = "button";
        delBtn.className = "sc-act-btn sc-act-btn--del";
        delBtn.title = "Xóa lối tắt";
        delBtn.setAttribute("aria-label", "Xóa " + item.name);
        delBtn.appendChild(iconClose());
        delBtn.addEventListener("click", function (ev) {
          ev.preventDefault();
          ev.stopPropagation();
          removeShortcutById(item.id);
        });

        actions.appendChild(editBtn);
        actions.appendChild(delBtn);
        tile.appendChild(actions);
        gridEl.appendChild(tile);
      })(shortcuts[i]);
    }

    // "+ Thêm lối tắt" button tile
    var addBtn = document.createElement("button");
    addBtn.type = "button";
    addBtn.id = "lp-sc-add";
    addBtn.className = "sc-add-btn";
    addBtn.title = "Thêm trang web thường dùng";

    var addCircle = document.createElement("span");
    addCircle.className = "sc-add-circle";
    addCircle.setAttribute("aria-hidden", "true");
    addCircle.appendChild(iconPlus());

    var addLabel = document.createElement("span");
    addLabel.className = "sc-label";
    addLabel.textContent = "Thêm lối tắt";

    addBtn.appendChild(addCircle);
    addBtn.appendChild(addLabel);
    addBtn.addEventListener("click", function () {
      openShortcutModal(null);
    });
    gridEl.appendChild(addBtn);

    if (resetBtn) {
      resetBtn.hidden = isDefaultList(shortcuts);
    }
  }

  if (nameInput) nameInput.addEventListener("input", updateModalPreview);
  if (urlInput) urlInput.addEventListener("input", updateModalPreview);
  if (cancelBtn) cancelBtn.addEventListener("click", closeShortcutModal);
  if (deleteBtn) {
    deleteBtn.addEventListener("click", function () {
      if (editingId) {
        removeShortcutById(editingId);
      }
      closeShortcutModal();
    });
  }
  if (backdropEl) {
    backdropEl.addEventListener("click", function (ev) {
      if (ev.target === backdropEl) closeShortcutModal();
    });
  }
  if (modalForm) {
    modalForm.addEventListener("submit", function (ev) {
      ev.preventDefault();
      var normUrl = normalizeShortcutUrl(urlInput ? urlInput.value : "");
      if (!normUrl) {
        if (errorEl) errorEl.textContent = "Vui lòng nhập địa chỉ URL hợp lệ (VD: facebook.com hoặc https://...)";
        if (urlInput) urlInput.focus();
        return;
      }
      var finalName = (nameInput ? nameInput.value.trim() : "") || inferSiteName(normUrl);
      if (editingId) {
        for (var i = 0; i < shortcuts.length; i++) {
          if (shortcuts[i].id === editingId) {
            shortcuts[i] = { id: editingId, name: finalName, url: normUrl };
            break;
          }
        }
      } else {
        shortcuts.push({
          id: "sc_" + Date.now() + "_" + Math.floor(Math.random() * 10000),
          name: finalName,
          url: normUrl
        });
      }
      saveShortcuts(shortcuts);
      renderShortcuts();
      closeShortcutModal();
    });
  }
  if (resetBtn) {
    resetBtn.addEventListener("click", function () {
      shortcuts = cloneDefaults();
      saveShortcuts(shortcuts);
      renderShortcuts();
    });
  }

  renderShortcuts();

  document.addEventListener("keydown", function (ev) {
    if (ev.key === "Escape" && backdropEl && !backdropEl.hidden) {
      ev.preventDefault();
      closeShortcutModal();
      return;
    }
    var tag = document.activeElement ? document.activeElement.tagName : "";
    var inField = tag === "INPUT" || tag === "TEXTAREA";
    if (ev.key === "/" && !inField && (!backdropEl || backdropEl.hidden) && !ev.ctrlKey && !ev.metaKey && !ev.altKey) {
      ev.preventDefault();
      if (searchInput) {
        searchInput.focus();
        searchInput.select();
      }
    }
  });

  function detectOs(ua) {
    if (ua.indexOf("Windows") !== -1) return "Windows";
    if (ua.indexOf("Mac OS X") !== -1 || ua.indexOf("Macintosh") !== -1) return "macOS";
    if (ua.indexOf("Android") !== -1) return "Android";
    if (ua.indexOf("Linux") !== -1) return "Linux";
    return "Desktop";
  }

  function detectChromiumMajor(ua) {
    var m = /Chrome\/(\d+)/.exec(ua);
    return m ? "Chromium " + m[1] : "Chromium Core";
  }

  function setText(id, val) {
    var el = document.getElementById(id);
    if (el) el.textContent = val;
  }

  function updateClockAndTelemetry() {
    var tz = "UTC";
    try {
      tz = Intl.DateTimeFormat().resolvedOptions().timeZone || "Local";
    } catch (_e) {}

    var now = new Date();
    var hh = String(now.getHours()).padStart(2, "0");
    var mm = String(now.getMinutes()).padStart(2, "0");
    setText("lp-clock", hh + ":" + mm + " · " + tz);
    setText("stat-tz", "Múi giờ: " + tz);

    var lang = navigator.language || "vi-VN";
    setText("lp-locale", lang);

    var ua = navigator.userAgent || "";
    var os = detectOs(ua);
    var cores = navigator.hardwareConcurrency ? navigator.hardwareConcurrency + " nhân" : "Đa nhân";
    var scr = window.screen ? (window.screen.width + "×" + window.screen.height) : "";
    setText("stat-hw", os + " · " + cores + (scr ? " · " + scr : ""));

    var mem = navigator.deviceMemory ? " · " + navigator.deviceMemory + "GB RAM" : "";
    setText("lp-env-summary", os + " · " + cores + mem + (scr ? " · " + scr : ""));

    var ck = navigator.cookieEnabled ? "Cookies bật · Cô lập" : "Cookies tắt";
    setText("stat-cookie", ck);

    var netState = navigator.onLine ? "Trực tuyến · WebRTC bảo vệ" : "Ngoại tuyến";
    setText("stat-net", netState);

    setText("stat-core", detectChromiumMajor(ua) + " · " + lang);
  }

  updateClockAndTelemetry();
  setInterval(updateClockAndTelemetry, 15000);
})();
"##;
    js.replace("__MARKER__", marker_comment)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ntp_payloads_never_contain_dollar_or_inner_html() {
        let html = build_ntp_html("new_tab_page.js", "TEST_HTML");
        let js = build_ntp_js("TEST_JS");
        assert!(!html.contains('$'), "HTML must not contain literal '$'");
        assert!(!js.contains('$'), "JS must not contain literal '$'");
        assert!(!js.contains("innerHTML"), "JS must not use innerHTML under Trusted Types");
        assert!(html.contains("rel=\"icon\""), "HTML must embed Blank Tab favicon link");
        assert!(js.contains("https://www.facebook.com"));
        assert!(js.contains("https://www.threads.net"));
        assert!(js.contains("https://www.instagram.com"));
        assert!(js.contains("https://mail.google.com"));
    }
}
