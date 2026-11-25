//! Generates the branded LPBrowser New Tab Page (`chrome://newtab/`) HTML and JS payloads
//! for `resources.pak` (`chrome://new-tab-page/` and `chrome://new-tab-page-third-party/`).
//!
//! IMPORTANT: Never include the literal `$` character in HTML or JS strings here because
//! Chromium's `WebUIDataSource` runs `ui::ReplaceTemplateExpressions` on WebUI resources.

use base64::Engine;

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
    let ic_main = data_uri(PNG_MAIN);
    let ic_taskbar = data_uri(PNG_TASKBAR);
    let ic_profiles = data_uri(PNG_PROFILES);
    let ic_ghost = data_uri(PNG_GHOST);
    let ic_proxy = data_uri(PNG_PROXY);
    let ic_fp = data_uri(PNG_FP);
    let ic_cookie = data_uri(PNG_COOKIE);
    let ic_ua = data_uri(PNG_UA);
    let ic_vault = data_uri(PNG_VAULT);

    let template = r#"<!doctype html>
<!-- __MARKER__ -->
<html lang="vi" data-theme="dark">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<meta name="color-scheme" content="dark light">
<title>Thẻ mới - LPBrowser</title>
<style>
/* Hallmark · pre-emit critique: P5 H5 E5 S5 R5 V5 */
:root {
  --font-sans: "Inter Variable", "Inter", "Segoe UI", system-ui, -apple-system, BlinkMacSystemFont, Roboto, sans-serif;
  --font-mono: "JetBrains Mono", ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  --bg: #0c1427;
  --bg-glow-a: rgba(34, 211, 238, 0.11);
  --bg-glow-b: rgba(168, 85, 247, 0.08);
  --surface: rgba(19, 30, 56, 0.82);
  --surface-hover: rgba(25, 40, 74, 0.96);
  --surface-pill: rgba(255, 255, 255, 0.045);
  --border: rgba(148, 184, 255, 0.13);
  --border-strong: rgba(56, 189, 248, 0.34);
  --ink: #f8fafc;
  --ink-2: #cbd5e1;
  --ink-3: #8597b8;
  --cyan: #22d3ee;
  --violet: #c084fc;
  --emerald: #34d399;
  --emerald-bg: rgba(16, 185, 129, 0.13);
  --shadow-bar: 0 14px 36px -10px rgba(4, 9, 20, 0.62);
  --shadow-card: 0 6px 20px -6px rgba(4, 9, 20, 0.45);
  color-scheme: dark;
}
html[data-theme="light"] {
  --bg: #f4f7fc;
  --bg-glow-a: rgba(14, 165, 233, 0.08);
  --bg-glow-b: rgba(147, 51, 234, 0.05);
  --surface: #ffffff;
  --surface-hover: #f8fafc;
  --surface-pill: #f1f5f9;
  --border: rgba(15, 23, 42, 0.09);
  --border-strong: rgba(2, 132, 199, 0.40);
  --ink: #0f172a;
  --ink-2: #334155;
  --ink-3: #64748b;
  --cyan: #0284c7;
  --violet: #7e22ce;
  --emerald: #059669;
  --emerald-bg: rgba(5, 150, 105, 0.10);
  --shadow-bar: 0 12px 28px -10px rgba(15, 23, 42, 0.10);
  --shadow-card: 0 4px 16px -4px rgba(15, 23, 42, 0.07);
  color-scheme: light;
}
html[data-theme="light"] .card,
html[data-theme="light"] .search-bar {
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
  padding: 20px 28px 36px;
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  gap: 24px;
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
  max-width: 700px;
  margin-inline: auto;
  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;
  gap: 18px;
}
.hero-lockup {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 16px;
}
.hero-icon {
  width: 72px;
  height: 72px;
  object-fit: contain;
  flex-shrink: 0;
  filter: drop-shadow(0 10px 22px rgba(8, 15, 32, 0.55));
}
.hero-copy {
  text-align: left;
}
.hero-copy h1 {
  margin: 0;
  font-size: 30px;
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
  margin: 5px 0 0;
  font-size: 13px;
  color: var(--ink-3);
  letter-spacing: -0.005em;
}
.search-bar {
  width: 100%;
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
  padding: 5px 12px;
  border-radius: 999px;
  background: var(--surface-pill);
  border: 1px solid var(--border);
  color: var(--ink-2);
  font-size: 12px;
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
  gap: 11px;
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
  transition: border-color 160ms ease, background 160ms ease, transform 160ms ease;
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

  <section class="hero" aria-label="Tìm kiếm và điều hướng">
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

    <nav class="quick-links" aria-label="Công cụ kiểm tra nhanh">
      <a class="qlink" href="https://iphey.com"><span class="qdot qdot--e"></span>Kiểm tra Iphey</a>
      <a class="qlink" href="https://whoer.net"><span class="qdot"></span>Kiểm tra IP (Whoer)</a>
      <a class="qlink" href="https://browserleaks.com"><span class="qdot qdot--v"></span>BrowserLeaks</a>
      <a class="qlink" href="https://pixelscan.net"><span class="qdot"></span>Pixelscan</a>
      <a class="qlink" href="https://www.google.com"><span class="qdot qdot--e"></span>Google</a>
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
<script type="module" src="__SCRIPT_SRC__"></script>
</body>
</html>
"#;

    template
        .replace("__MARKER__", marker_comment)
        .replace("__SCRIPT_SRC__", script_src)
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
    let js = r#"/* __MARKER__ */
(function () {
  var ENGINES = [
    { name: "Google", url: "https://www.google.com/search?q=" },
    { name: "DuckDuckGo", url: "https://duckduckgo.com/?q=" },
    { name: "Bing", url: "https://www.bing.com/search?q=" }
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

  document.addEventListener("keydown", function (ev) {
    if (ev.key === "/" && document.activeElement !== searchInput && !ev.ctrlKey && !ev.metaKey && !ev.altKey) {
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
"#;
    js.replace("__MARKER__", marker_comment)
}
