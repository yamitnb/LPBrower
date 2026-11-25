/* Hallmark · pre-emit critique: P5 H5 E5 S5 R5 V5 */
import { Button, CopyButton, cn } from "@proxyshard/shardx-ui-kit";
import {
  AndroidOsIcon, AppleOsIcon, CheckCircleIcon, FolderIcon, GlobeIcon,
  LinuxOsIcon, RegenerateIcon, RouteIcon, ShardLogo, WindowsOsIcon,
} from "../../../shared/icons";
import { clip } from "../../../shared/lib/clipboard";
import { useT } from "../../../shared/i18n";
import { HOST_OS } from "../../../shared/lib/utils";
import { AUTO_LANG, AUTO_TZ, osIdFor } from "../../../shared/constants";
import { useProfile } from "../../../entities/profile";
import { CountryFlag } from "../../../shared/ui/CountryFlag";
import { randomizeFingerprint, type ProfileFormDraft } from "../model/useProfileForm";

const OS_META: Record<string, { label: string; icon: typeof WindowsOsIcon }> = {
  Windows: { label: "Windows", icon: WindowsOsIcon },
  macOS: { label: "macOS", icon: AppleOsIcon },
  Linux: { label: "Linux", icon: LinuxOsIcon },
  Android: { label: "Android", icon: AndroidOsIcon },
};

function parseProxySummary(draft: ProfileFormDraft, savedProxy?: { kind: string; host: string; port: number; country?: string }) {
  if (savedProxy) {
    return {
      mode: savedProxy.kind.toUpperCase(),
      endpoint: `${savedProxy.host}:${savedProxy.port}`,
      country: savedProxy.country || "",
      direct: false,
    };
  }
  const line = draft.newProxyLine.trim();
  if (!line) return { mode: "Direct", endpoint: "IP máy thật", country: "", direct: true };
  const isSocks = line.startsWith("socks5://");
  const clean = line.replace(/^(socks5|https?):\/\//, "");
  const hostPort = clean.includes("@") ? clean.split("@")[1] : clean;
  const [h = "", p = ""] = hostPort.split(":");
  if (!h) return { mode: "Direct", endpoint: "IP máy thật", country: "", direct: true };
  return {
    mode: isSocks ? "SOCKS5" : "HTTP",
    endpoint: p ? `${h}:${p}` : h,
    country: "",
    direct: false,
  };
}

/**
 * Live Profile Blueprint Card: pinned right-hand inspector inside the Create/Edit Profile modal.
 * Shows live `LPB|<Name>` identity, OS, proxy routing, hardware telemetry, and User-Agent.
 */
export function ProfileBlueprintCard({
  draft,
  readOnly,
  saving,
}: {
  draft: ProfileFormDraft;
  readOnly: boolean;
  saving: boolean;
}) {
  const t = useT();
  const f = draft.form;
  const fingerprints = useProfile((s) => s.fingerprints);
  const proxies = useProfile((s) => s.proxies);

  const currentFp = fingerprints.find((x) => x.id === f.gpu_preset_id);
  const currentOs = currentFp ? osIdFor(currentFp.platform as string) : HOST_OS;
  const osInfo = OS_META[currentOs] ?? OS_META.Windows;
  const OsIcon = osInfo.icon;

  const savedProxy = f.proxy_id ? proxies.find((p) => p.id === f.proxy_id) : undefined;
  const proxySummary = parseProxySummary(draft, savedProxy);

  const rawName = f.name.trim() || "Chưa đặt tên";

  const gpuTitle = currentFp?.label || currentFp?.gpu || (f.gpu_preset_id ? f.gpu_preset_id.replace(/^([a-z0-9]+-)+/, "") : "Auto GPU");
  const cpuRam = `${f.hardware_concurrency || 8} nhân · ${f.device_memory || 8} GB`;
  const screenText = f.screen_w && f.screen_h
    ? `${f.screen_w} × ${f.screen_h}${f.refresh_rate ? ` · ${f.refresh_rate}Hz` : ""}`
    : "Theo thiết bị";
  const tzLang = `${f.timezone === AUTO_TZ || !f.timezone ? "Tự động" : f.timezone} · ${f.language === AUTO_LANG || !f.language ? "Auto" : f.language}`;
  const webrtcLabel = f.webrtc === "block" ? "Chặn (Block)" : f.webrtc === "tcp_only" ? "Chỉ TCP" : "Tự động";
  const extCount = f.extensions?.length ?? 0;

  return (
    <aside
      aria-label="Tóm tắt cấu hình hồ sơ"
      className="flex flex-col rounded-panel border border-stroke-soft-200 bg-bg-weak-50/70 overflow-hidden"
    >
      {/* Top Accent Bar matching profile color */}
      <div
        className="h-[3px] w-full shrink-0 bg-primary-base transition-colors"
        style={f.color ? { backgroundColor: f.color } : undefined}
      />

      {/* Identity Block */}
      <div className="flex flex-col gap-2.5 p-3.5 bg-bg-white-0 border-b border-stroke-soft-200">
        <div className="flex items-center justify-between gap-2">
          <span className="inline-flex items-center gap-1.5 text-[11px] font-medium uppercase tracking-wider text-text-soft-400">
            <span className="brand-mark inline-flex"><ShardLogo size={16} /></span>
            Tóm tắt hồ sơ
          </span>
          <span className="inline-flex items-center gap-1 rounded-full bg-bg-weak-50 px-2 py-0.5 text-[11px] font-medium text-text-sub-600 ring-1 ring-inset ring-stroke-soft-200">
            <OsIcon className="size-3 text-text-sub-600" />
            {osInfo.label}
          </span>
        </div>

        {/* Clean Profile Name Preview */}
        <div className="flex flex-col gap-1 min-w-0">
          <div
            className="text-[15px] font-semibold tracking-tight text-text-strong-950 truncate"
            title={rawName}
          >
            {rawName}
          </div>
          <div className="flex flex-wrap items-center gap-1.5 text-[11px] text-text-soft-400">
            <span className="inline-flex items-center gap-1">
              <FolderIcon className="size-3" />
              {draft.folder || "Không có thư mục"}
            </span>
            <span aria-hidden="true">·</span>
            <span className="inline-flex items-center gap-1 text-success-text font-medium">
              <CheckCircleIcon className="size-3 text-success-base" />
              Sẵn sàng
            </span>
            {f.taskbar_title?.trim() && (
              <>
                <span aria-hidden="true">·</span>
                <span
                  title={`Nhãn Taskbar: ${f.taskbar_title.trim()}`}
                  style={f.color ? { backgroundColor: f.color } : undefined}
                  className="inline-flex max-w-[96px] items-center truncate rounded-full bg-primary-base px-2 py-0.5 text-[10px] font-bold text-white"
                >
                  {f.taskbar_title.trim()}
                </span>
              </>
            )}
          </div>
        </div>

        {/* Proxy Routing Pill */}
        <div
          className={cn(
            "flex items-center justify-between gap-2 rounded-control px-2.5 py-2 text-xs border transition-colors",
            proxySummary.direct
              ? "bg-bg-weak-50 border-stroke-soft-200 text-text-sub-600"
              : "bg-primary-base/5 border-primary-base/25 text-text-strong-950",
          )}
        >
          <div className="flex items-center gap-2 min-w-0">
            {proxySummary.country ? (
              <CountryFlag cc={proxySummary.country} height={12} />
            ) : proxySummary.direct ? (
              <GlobeIcon className="size-3.5 shrink-0 text-icon-soft-400" />
            ) : (
              <RouteIcon className="size-3.5 shrink-0 text-primary-base" />
            )}
            <span className="font-mono text-[11px] truncate" title={proxySummary.endpoint}>
              {proxySummary.endpoint}
            </span>
          </div>
          <span
            className={cn(
              "shrink-0 rounded px-1.5 py-0.5 text-[10px] font-semibold uppercase tracking-wide",
              proxySummary.direct
                ? "bg-bg-white-0 text-text-soft-400 ring-1 ring-inset ring-stroke-soft-200"
                : "bg-primary-base/15 text-primary-text",
            )}
          >
            {proxySummary.mode}
          </span>
        </div>
      </div>

      {/* Hardware & Fingerprint Telemetry Rows */}
      <div className="flex flex-col divide-y divide-stroke-soft-200/70 px-3.5 py-1 text-xs">
        <SpecRow label="Taskbar Title" value={f.taskbar_title?.trim() || "Mặc định (Trống)"} title={f.taskbar_title?.trim() || undefined} />
        <SpecRow label="GPU" value={gpuTitle} title={gpuTitle} />
        <SpecRow label="CPU / RAM" value={cpuRam} mono />
        <SpecRow label="Màn hình" value={screenText} mono />
        <SpecRow label="Múi giờ / Ngôn ngữ" value={tzLang} title={tzLang} />
        <SpecRow label="WebRTC" value={webrtcLabel} />
        <SpecRow label="Tiện ích mở rộng" value={extCount > 0 ? `${extCount} tiện ích` : "Mặc định"} />
      </div>

      {/* User-Agent Block */}
      {f.user_agent && (
        <div className="px-3.5 pt-2 pb-3 border-t border-stroke-soft-200/70">
          <div className="flex items-center justify-between mb-1">
            <span className="text-[10px] font-medium uppercase tracking-wider text-text-soft-400">
              User-Agent
            </span>
            <CopyButton
              size="3xsmall"
              label="Sao chép User-Agent"
              onCopy={() => void clip.write(f.user_agent)}
            />
          </div>
          <div className="rounded-control bg-bg-white-0 border border-stroke-soft-200 px-2.5 py-1.5">
            <p
              data-selectable
              title={f.user_agent}
              className="m-0 font-mono text-[10.5px] leading-[1.45] text-text-sub-600 line-clamp-2 break-all"
            >
              {f.user_agent}
            </p>
          </div>
        </div>
      )}

      {/* Randomize Action Footer (Locked to current OS) */}
      <div className="p-3 pt-0 mt-auto">
        <Button
          type="button"
          variant="neutral"
          mode="stroke"
          size="xsmall"
          className="w-full justify-center font-medium bg-bg-white-0"
          disabled={saving || readOnly}
          leftIcon={<RegenerateIcon className="size-3.5" />}
          onClick={() => void randomizeFingerprint()}
          title={`Ngẫu nhiên lại thông số phần cứng & vân tay (giữ nguyên ${osInfo.label})`}
        >
          {t("profileForm.randomizeSpecs") || "Ngẫu nhiên thông số"}
        </Button>
      </div>
    </aside>
  );
}

function SpecRow({
  label,
  value,
  mono,
  title,
}: {
  label: string;
  value: string;
  mono?: boolean;
  title?: string;
}) {
  return (
    <div className="flex items-center justify-between gap-3 py-2 min-w-0">
      <span className="text-[11px] text-text-soft-400 shrink-0">{label}</span>
      <span
        title={title ?? value}
        className={cn(
          "text-[11.5px] font-medium text-text-strong-950 truncate text-right",
          mono && "font-mono tabular-nums",
        )}
      >
        {value}
      </span>
    </div>
  );
}
