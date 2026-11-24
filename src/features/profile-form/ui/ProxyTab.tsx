/* Hallmark · pre-emit critique: P5 H5 E5 S5 R5 V5 */
import { useMemo, useState } from "react";
import { cn } from "@proxyshard/shardx-ui-kit";
import { patchDraft, patchForm, type ProfileFormDraft } from "../model/useProfileForm";
import {
  AlertCircleIcon, CheckCircleIcon, CloseIcon, EyeIcon, EyeOffIcon,
  GlobeIcon, RouteIcon, SearchIcon, SpinnerIcon,
} from "../../../shared/icons";
import { CountryFlag } from "../../../shared/ui/CountryFlag";
import { proxyFullTest, type ProxyEntry, type ProxyTestSnapshot } from "../../../entities/proxy";
import { useProfile } from "../../../entities/profile";

type ProxyKind = "http" | "socks5" | "none";

function composeLine(kind: ProxyKind, host: string, port: string, username = "", password = ""): string {
  const p = Number(port);
  const h = host.trim();
  if (kind === "none" || !h || !p) return "";
  const auth = username ? `${encodeURIComponent(username)}:${encodeURIComponent(password)}@` : "";
  return `${kind}://${auth}${h}:${p}`;
}

const PROTOCOL_OPTIONS: { value: ProxyKind; label: string; hint: string }[] = [
  { value: "http", label: "HTTP", hint: "Phổ biến" },
  { value: "socks5", label: "SOCKS5", hint: "Hỗ trợ UDP" },
  { value: "none", label: "Không dùng proxy", hint: "IP máy thật" },
];

export function ProxyTab({ draft }: { draft: ProfileFormDraft }) {
  const proxies = useProfile((s) => s.proxies);
  const savedProxy = draft.form.proxy_id ? proxies.find((p) => p.id === draft.form.proxy_id) : undefined;

  const [proxyKind, setProxyKind] = useState<ProxyKind>(() => {
    if (savedProxy) return savedProxy.kind === "socks5" ? "socks5" : "http";
    if (draft.newProxyLine) {
      return draft.newProxyLine.startsWith("socks5://") ? "socks5" : "http";
    }
    return "http";
  });

  const [host, setHost] = useState<string>(() => {
    if (savedProxy) return savedProxy.host;
    if (draft.newProxyLine) {
      const clean = draft.newProxyLine.replace(/^(socks5|https?):\/\//, "");
      const hp = clean.includes("@") ? clean.split("@")[1] : clean;
      return hp.split(":")[0] || "";
    }
    return "";
  });

  const [port, setPort] = useState<string>(() => {
    if (savedProxy) return String(savedProxy.port);
    if (draft.newProxyLine) {
      const clean = draft.newProxyLine.replace(/^(socks5|https?):\/\//, "");
      const hp = clean.includes("@") ? clean.split("@")[1] : clean;
      return hp.split(":")[1] || "";
    }
    return "";
  });

  const [username, setUsername] = useState<string>(() => {
    if (savedProxy) return savedProxy.username;
    if (draft.newProxyLine) {
      const clean = draft.newProxyLine.replace(/^(socks5|https?):\/\//, "");
      if (clean.includes("@")) {
        const auth = clean.split("@")[0];
        return decodeURIComponent(auth.split(":")[0] || "");
      }
      const parts = clean.split(":");
      if (parts.length >= 3) return parts[2] || "";
    }
    return "";
  });

  const [password, setPassword] = useState<string>(() => {
    if (savedProxy) return savedProxy.password;
    if (draft.newProxyLine) {
      const clean = draft.newProxyLine.replace(/^(socks5|https?):\/\//, "");
      if (clean.includes("@")) {
        const auth = clean.split("@")[0];
        return decodeURIComponent(auth.split(":")[1] || "");
      }
      const parts = clean.split(":");
      if (parts.length >= 4) return parts[3] || "";
    }
    return "";
  });

  const [showPass, setShowPass] = useState(false);
  const [showLibrary, setShowLibrary] = useState(false);
  const [libSearch, setLibSearch] = useState("");
  const [testing, setTesting] = useState(false);
  const [testResult, setTestResult] = useState<ProxyTestSnapshot | null>(null);
  const [testError, setTestError] = useState<string | null>(null);

  const syncToDraft = (newKind: ProxyKind, newHost: string, newPort: string, newUser: string, newPass: string) => {
    if (newKind === "none") {
      patchDraft({ newProxyLine: "" });
      patchForm({ proxy_id: null });
      return;
    }
    const line = composeLine(newKind, newHost, newPort, newUser, newPass);
    patchDraft({ newProxyLine: line });
    patchForm({ proxy_id: null });
  };

  const handleKindChange = (k: ProxyKind) => {
    setProxyKind(k);
    setTestResult(null);
    setTestError(null);
    if (k === "none") {
      patchDraft({ newProxyLine: "" });
      patchForm({ proxy_id: null });
    } else {
      syncToDraft(k, host, port, username, password);
    }
  };

  const parseAndApplyProxyString = (raw: string) => {
    const trimmed = raw.trim();
    if (!trimmed) {
      setHost("");
      syncToDraft(proxyKind, "", port, username, password);
      return;
    }

    if (trimmed.includes("://") || trimmed.includes(":") || trimmed.includes("@")) {
      let rest = trimmed;
      let detectedKind: ProxyKind = proxyKind === "none" ? "http" : proxyKind;

      if (rest.startsWith("socks5://")) {
        detectedKind = "socks5";
        rest = rest.slice(9);
      } else if (rest.startsWith("http://")) {
        detectedKind = "http";
        rest = rest.slice(7);
      } else if (rest.startsWith("https://")) {
        detectedKind = "http";
        rest = rest.slice(8);
      }

      let newHost = "";
      let newPort = "";
      let newUser = "";
      let newPass = "";

      if (rest.includes("@")) {
        const [auth, hp] = rest.split("@");
        const [u, p = ""] = auth.split(":");
        newUser = decodeURIComponent(u || "");
        newPass = decodeURIComponent(p || "");
        const [h, pt = ""] = hp.split(":");
        newHost = h || "";
        newPort = pt || "";
      } else if (rest.includes(":")) {
        const parts = rest.split(":");
        newHost = parts[0] || "";
        newPort = parts[1] || "";
        newUser = parts[2] || "";
        newPass = parts[3] || "";
      }

      if (newHost) {
        setProxyKind(detectedKind);
        setHost(newHost);
        setPort(newPort);
        setUsername(newUser);
        setPassword(newPass);
        setTestResult(null);
        setTestError(null);
        syncToDraft(detectedKind, newHost, newPort, newUser, newPass);
        return;
      }
    }

    setHost(raw);
    syncToDraft(proxyKind, raw, port, username, password);
  };

  const handleRunTest = async () => {
    if (proxyKind === "none" || !host.trim()) return;
    const entry: ProxyEntry = {
      id: draft.form.proxy_id || "",
      name: `${host}:${port || 0}`,
      kind: proxyKind === "socks5" ? "socks5" : "http",
      host: host.trim(),
      port: Number(port) || 0,
      username: username.trim(),
      password: password.trim(),
      country: "",
      notes: "",
    };

    setTesting(true);
    setTestResult(null);
    setTestError(null);

    try {
      const snap = await proxyFullTest(entry);
      if (snap && snap.ip) {
        setTestResult(snap);
      } else {
        setTestError(snap?.geo_error || snap?.udp_error || "Không thể kết nối đến máy chủ Proxy.");
      }
    } catch (e) {
      setTestError(String(e) || "Lỗi kiểm tra kết nối Proxy.");
    } finally {
      setTesting(false);
    }
  };

  const handleSelectFromLibrary = (p: ProxyEntry) => {
    const k: ProxyKind = p.kind === "socks5" ? "socks5" : "http";
    setProxyKind(k);
    setHost(p.host);
    setPort(String(p.port));
    setUsername(p.username || "");
    setPassword(p.password || "");
    setShowLibrary(false);
    setTestResult(null);
    setTestError(null);
    patchDraft({ newProxyLine: "" });
    patchForm({ proxy_id: p.id });
  };

  const filteredLibrary = useMemo(() => {
    if (!libSearch.trim()) return proxies;
    const q = libSearch.toLowerCase();
    return proxies.filter((p) =>
      p.host.toLowerCase().includes(q) ||
      p.name.toLowerCase().includes(q) ||
      (p.country && p.country.toLowerCase().includes(q))
    );
  }, [proxies, libSearch]);

  const isNone = proxyKind === "none";

  const fieldCls = cn(
    "h-[34px] w-full rounded-control border px-2.5 text-xs outline-hidden transition-all",
    isNone
      ? "bg-bg-weak-50 border-stroke-soft-200 text-text-soft-400 cursor-not-allowed"
      : "bg-bg-white-0 border-stroke-soft-200 text-text-strong-950 placeholder:text-text-soft-400 focus:border-primary-base focus:ring-2 focus:ring-primary-base/15",
  );

  return (
    <section data-tour="form-proxy" className="flex flex-col gap-3 rounded-panel border border-stroke-soft-200 bg-bg-white-0 p-3.5">
      {/* Header */}
      <div className="flex items-center justify-between gap-2">
        <div className="flex items-center gap-2">
          <RouteIcon className="size-3.5 text-primary-base" />
          <h3 className="m-0 text-xs font-semibold text-text-strong-950">
            Kết nối mạng &amp; Proxy
          </h3>
        </div>
        <button
          type="button"
          onClick={() => setShowLibrary((v) => !v)}
          className={cn(
            "inline-flex items-center gap-1.5 rounded-control px-2.5 py-1 text-[11px] font-medium transition-colors cursor-pointer border",
            showLibrary
              ? "border-primary-base/30 bg-primary-base/10 text-primary-text"
              : "border-stroke-soft-200 bg-bg-weak-50 text-text-sub-600 hover:text-text-strong-950 hover:border-stroke-sub-300",
          )}
        >
          <GlobeIcon className="size-3" />
          {showLibrary ? "Đóng thư viện" : `Thư viện proxy (${proxies.length})`}
        </button>
      </div>

      {/* Saved Proxy Library Drawer */}
      {showLibrary && (
        <div className="flex flex-col gap-2 rounded-control border border-primary-base/25 bg-bg-weak-50/80 p-2.5">
          <div className="relative">
            <SearchIcon className="pointer-events-none absolute left-2.5 top-1/2 -translate-y-1/2 size-3.5 text-icon-soft-400" />
            <input
              type="text"
              placeholder="Tìm theo IP, tên hoặc mã quốc gia..."
              value={libSearch}
              onChange={(e) => setLibSearch(e.target.value)}
              className="h-[30px] w-full rounded-control border border-stroke-soft-200 bg-bg-white-0 pl-7 pr-7 text-xs text-text-strong-950 placeholder:text-text-soft-400 outline-hidden focus:border-primary-base"
            />
            {libSearch && (
              <button
                type="button"
                onClick={() => setLibSearch("")}
                className="absolute right-2 top-1/2 -translate-y-1/2 text-icon-soft-400 hover:text-text-strong-950 cursor-pointer"
              >
                <CloseIcon className="size-3" />
              </button>
            )}
          </div>

          <div className="max-h-40 overflow-y-auto flex flex-col gap-1">
            {filteredLibrary.length === 0 ? (
              <p className="m-0 py-3 text-center text-xs text-text-soft-400">
                {proxies.length === 0 ? "Chưa có proxy nào trong thư viện." : "Không tìm thấy proxy phù hợp."}
              </p>
            ) : (
              filteredLibrary.map((p) => {
                const isSelected = draft.form.proxy_id === p.id;
                return (
                  <button
                    key={p.id}
                    type="button"
                    onClick={() => handleSelectFromLibrary(p)}
                    className={cn(
                      "flex items-center justify-between gap-2 rounded-control px-2.5 py-1.5 text-left text-xs transition-colors cursor-pointer",
                      isSelected
                        ? "bg-primary-base/10 text-primary-text font-medium ring-1 ring-inset ring-primary-base/30"
                        : "bg-bg-white-0 hover:bg-bg-hover text-text-strong-950 border border-stroke-soft-200/70",
                    )}
                  >
                    <div className="flex items-center gap-2 min-w-0">
                      {p.country ? <CountryFlag cc={p.country} height={12} /> : <GlobeIcon className="size-3.5 text-icon-soft-400" />}
                      <span className="font-mono text-[11.5px] truncate">{p.host}:{p.port}</span>
                      <span className="rounded bg-bg-weak-50 px-1.5 py-0.5 text-[10px] font-semibold uppercase text-text-soft-400">
                        {p.kind}
                      </span>
                    </div>
                    {p.name && p.name !== `${p.host}:${p.port}` && (
                      <span className="text-[11px] text-text-soft-400 truncate max-w-[130px]">{p.name}</span>
                    )}
                  </button>
                );
              })
            )}
          </div>
        </div>
      )}

      {/* 3-Segment Protocol Switcher */}
      <div
        role="radiogroup"
        aria-label="Loại proxy"
        className="grid grid-cols-3 gap-1 rounded-control bg-bg-weak-50 p-1 border border-stroke-soft-200"
      >
        {PROTOCOL_OPTIONS.map((opt) => {
          const active = proxyKind === opt.value;
          return (
            <button
              key={opt.value}
              type="button"
              role="radio"
              aria-checked={active}
              onClick={() => handleKindChange(opt.value)}
              className={cn(
                "flex items-center justify-center gap-1.5 rounded-[5px] px-2.5 py-1.5 text-xs font-medium transition-all cursor-pointer outline-hidden",
                active
                  ? "bg-bg-white-0 text-text-strong-950 shadow-2xs ring-1 ring-stroke-soft-200"
                  : "text-text-sub-600 hover:text-text-strong-950",
              )}
            >
              <span>{opt.label}</span>
            </button>
          );
        })}
      </div>

      {/* Fields Grid (or Direct Hint) */}
      {isNone ? (
        <div className="flex items-center gap-2.5 rounded-control border border-stroke-soft-200 bg-bg-weak-50/70 px-3 py-2.5 text-xs text-text-sub-600">
          <GlobeIcon className="size-4 shrink-0 text-icon-sub-600" />
          <span>Hồ sơ sẽ kết nối trực tiếp bằng mạng Internet của thiết bị này (Direct).</span>
        </div>
      ) : (
        <>
          <div className="grid grid-cols-1 sm:grid-cols-[1fr_130px] gap-2.5">
            <div className="flex flex-col gap-1">
              <label className="text-[11px] font-medium text-text-sub-600">
                Host / Địa chỉ IP <span className="font-normal text-text-soft-400">(dán nhanh host:port:user:pass)</span>
              </label>
              <input
                type="text"
                value={host}
                placeholder="192.168.1.1 hoặc dán chuỗi proxy..."
                onChange={(e) => parseAndApplyProxyString(e.target.value)}
                className={cn(fieldCls, "font-mono")}
              />
            </div>

            <div className="flex flex-col gap-1">
              <label className="text-[11px] font-medium text-text-sub-600">Port</label>
              <input
                type="text"
                inputMode="numeric"
                value={port}
                placeholder="8080"
                onChange={(e) => {
                  const cleaned = e.target.value.replace(/\D/g, "").slice(0, 5);
                  setPort(cleaned);
                  setTestResult(null);
                  setTestError(null);
                  syncToDraft(proxyKind, host, cleaned, username, password);
                }}
                className={cn(fieldCls, "font-mono tabular-nums")}
              />
            </div>
          </div>

          <div className="grid grid-cols-1 sm:grid-cols-2 gap-2.5">
            <div className="flex flex-col gap-1">
              <label className="text-[11px] font-medium text-text-sub-600">Username</label>
              <input
                type="text"
                value={username}
                placeholder="Tùy chọn"
                onChange={(e) => {
                  setUsername(e.target.value);
                  setTestResult(null);
                  setTestError(null);
                  syncToDraft(proxyKind, host, port, e.target.value, password);
                }}
                className={fieldCls}
              />
            </div>

            <div className="flex flex-col gap-1">
              <label className="text-[11px] font-medium text-text-sub-600">Password</label>
              <div className="relative">
                <input
                  type={showPass ? "text" : "password"}
                  value={password}
                  placeholder="Tùy chọn"
                  onChange={(e) => {
                    setPassword(e.target.value);
                    setTestResult(null);
                    setTestError(null);
                    syncToDraft(proxyKind, host, port, username, e.target.value);
                  }}
                  className={cn(fieldCls, "pr-8")}
                />
                <button
                  type="button"
                  onClick={() => setShowPass((v) => !v)}
                  aria-label={showPass ? "Ẩn mật khẩu" : "Hiện mật khẩu"}
                  className="absolute right-2 top-1/2 -translate-y-1/2 text-icon-soft-400 hover:text-text-strong-950 cursor-pointer"
                >
                  {showPass ? <EyeOffIcon className="size-3.5" /> : <EyeIcon className="size-3.5" />}
                </button>
              </div>
            </div>
          </div>

          {/* Action Row: Kiểm tra kết nối */}
          <div className="flex items-center justify-between pt-0.5">
            <button
              type="button"
              disabled={!host.trim() || testing}
              onClick={() => void handleRunTest()}
              className="inline-flex items-center gap-1.5 rounded-control border border-stroke-soft-200 bg-bg-weak-50 px-2.5 py-1 text-xs font-medium text-text-strong-950 hover:border-primary-base/40 hover:text-primary-text transition-colors cursor-pointer disabled:opacity-40 disabled:cursor-not-allowed"
            >
              {testing ? (
                <SpinnerIcon className="size-3.5 animate-spin text-primary-base" />
              ) : (
                <CheckCircleIcon className="size-3.5 text-primary-base" />
              )}
              {testing ? "Đang kiểm tra kết nối..." : "Kiểm tra proxy"}
            </button>

            {savedProxy && (
              <span className="text-[11px] text-text-soft-400">
                Đang liên kết từ thư viện
              </span>
            )}
          </div>
        </>
      )}

      {/* Test Success Strip */}
      {testResult && (
        <div className="flex flex-wrap items-center justify-between gap-2 rounded-control bg-success-base/10 border border-success-base/25 px-3 py-2 text-xs">
          <div className="flex items-center gap-2 min-w-0">
            <CheckCircleIcon className="size-3.5 shrink-0 text-success-base" />
            <span className="font-mono font-semibold text-text-strong-950">{testResult.ip}</span>
            {testResult.country_code && <CountryFlag cc={testResult.country_code} height={12} />}
            <span className="text-text-sub-600 truncate">
              {testResult.country || testResult.country_code}{testResult.isp ? ` · ${testResult.isp}` : ""}
            </span>
          </div>
          {testResult.tcp_ms != null && (
            <span className="font-mono text-[11px] font-semibold text-success-text tabular-nums">
              {testResult.tcp_ms} ms
            </span>
          )}
        </div>
      )}

      {/* Test Error Strip */}
      {testError && (
        <div className="flex items-start gap-2 rounded-control bg-error-base/10 border border-error-base/25 px-3 py-2 text-xs text-error-text">
          <AlertCircleIcon className="size-3.5 shrink-0 mt-0.5 text-error-base" />
          <div className="flex flex-col gap-0.5 min-w-0">
            <span className="font-semibold">Kết nối Proxy thất bại</span>
            <span className="text-[11px] opacity-90 break-words">{testError}</span>
          </div>
        </div>
      )}
    </section>
  );
}
