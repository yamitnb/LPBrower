import { useEffect, useRef } from "react";
import { Button, CopyButton, cn } from "@proxyshard/shardx-ui-kit";
import { AlertCircleIcon, InfoIcon } from "../../shared/icons";
import { useT } from "../../shared/i18n";
import { clip } from "../../shared/lib/clipboard";
import { fmtBytesProgress } from "../../shared/lib/format";
import { useRunningCount } from "../../entities/profile";
import {
  dismissLauncherUpdate,
  offerLauncherUpdate,
  useLauncherUpdate,
} from "../../features/launcher-update";
import { HAS_CAPTIONS } from "../WindowControls";
import {
  dismissEngineUpdate,
  joined,
  reason,
  retryEngineUpdate,
  startEngineUpdate,
  stopAndFinish,
  useEngineUpdate,
} from "./engineUpdate";

/// Shows one slim top banner when:
/// 1. An engine update stopped with the previous engine kept (#122), or
/// 2. A Core Chromium update is downloading/applying, or
/// 3. Background check found a newer Core Chromium engine, or
/// 4. Background check found a newer LPBrowser launcher release.
export function EngineUpdateBanner() {
  const t = useT();
  const error = useEngineUpdate((s) => s.error);
  const staged = useEngineUpdate((s) => s.staged);
  const retrying = useEngineUpdate((s) => s.retrying);
  const progress = useEngineUpdate((s) => s.progress);
  const status = useEngineUpdate((s) => s.status);
  const dismissedEngine = useEngineUpdate((s) => s.dismissedTarget);

  const launcherOffer = useLauncherUpdate((s) => s.offer);
  const launcherDownloading = useLauncherUpdate((s) => s.downloading);
  const launcherInstalling = useLauncherUpdate((s) => s.installing);
  const launcherProgress = useLauncherUpdate((s) => s.progress);
  const dismissedLauncher = useLauncherUpdate((s) => s.dismissedVersion);

  const running = useRunningCount();
  const before = useRef(running);
  useEffect(() => {
    const was = before.current;
    before.current = running;
    if (!(was > 0 && running === 0)) return;
    // A browser may still be exiting when its profile shows stopped: a few tries, further apart.
    let gone = false;
    void (async () => {
      for (const ms of [800, 2500, 6000]) {
        await new Promise((r) => setTimeout(r, ms));
        if (gone || useEngineUpdate.getState().error?.stage !== "busy") return;
        await retryEngineUpdate(true);
      }
    })();
    return () => { gone = true; };
  }, [running]);

  // Priority 1: Stopped/failed engine update with previous engine kept.
  if (error) {
    const title = retrying ? t("engineUpdate.updatingTitle") : t("engineUpdate.title");
    const detail = !retrying
      ? [staged ? t("engineUpdate.staged") : "", reason(t, error), t("engineUpdate.kept")].filter(Boolean).reduce(joined)
      : staged ? t("engineUpdate.finishing")
        : progress?.phase === "download" ? t("engineUpdate.downloaded", { progress: fmtBytesProgress(progress.received, progress.total) })
          : progress?.phase === "branding" ? t("firstRunGate.branding")
            : progress ? t("firstRunGate.extracting") : t("firstRunGate.contactingCdn");
    const stopFirst = error.stage === "busy" && running > 0 && !retrying;
    return (
      <div
        data-engine-update={error.stage}
        className={cn("mx-gutter mt-2.5 flex shrink-0 items-center gap-2 rounded-control bg-warning-tint px-3 py-1 text-secondary text-text-strong-950 forced-edge", HAS_CAPTIONS && "me-[114px]")}
      >
        <div className="flex min-w-0 flex-1 items-start gap-2 py-1">
          <AlertCircleIcon aria-hidden="true" className="mt-0.5 size-3.5 shrink-0 text-warning-text" />
          <p role="status" className="m-0 min-w-0 [overflow-wrap:anywhere]">
            <strong className="font-medium">{title}</strong>
            {/[。！？]$/.test(title) ? "" : " "}
            <span className="text-text-sub-600">{detail}</span>
          </p>
        </div>
        {error.message && <CopyButton size="2xsmall" className="shrink-0" label={t("kit.copyDetails")} onCopy={async () => { await clip.write(error.message); }} />}
        {stopFirst ? (
          <Button variant="neutral" mode="stroke" size="2xsmall" className="shrink-0" onClick={() => void stopAndFinish()}>
            {running === 1 ? t("extensions.stopAndUpdateOne", { n: running }) : t("extensions.stopAndUpdateMany", { n: running })}
          </Button>
        ) : (
          <Button variant="neutral" mode="stroke" size="2xsmall" className="shrink-0" isLoading={retrying} onClick={() => void retryEngineUpdate()}>
            {t("firstRunGate.retry")}
          </Button>
        )}
      </div>
    );
  }

  // Priority 2: Active Core Chromium update triggered from Settings or banner.
  if (retrying) {
    const title = t("engineUpdate.updatingTitle");
    const detail = progress?.phase === "download"
      ? t("engineUpdate.downloaded", { progress: fmtBytesProgress(progress.received, progress.total) })
      : progress?.phase === "branding"
        ? t("firstRunGate.branding")
        : progress ? t("firstRunGate.extracting") : t("firstRunGate.contactingCdn");
    return (
      <div
        data-engine-update="updating"
        className={cn("mx-gutter mt-2.5 flex shrink-0 items-center gap-2 rounded-control bg-primary-tint px-3 py-1 text-secondary text-text-strong-950 forced-edge", HAS_CAPTIONS && "me-[114px]")}
      >
        <div className="flex min-w-0 flex-1 items-start gap-2 py-1">
          <InfoIcon aria-hidden="true" className="mt-0.5 size-3.5 shrink-0 text-primary-text" />
          <p role="status" className="m-0 min-w-0 [overflow-wrap:anywhere]">
            <strong className="font-medium">{title}</strong>
            {/[。！？]$/.test(title) ? "" : " "}
            <span className="text-text-sub-600">{detail}</span>
          </p>
        </div>
      </div>
    );
  }

  // Priority 3: Background check found a newer Core Chromium engine.
  const engineTarget = status?.remote_chromium_version ?? status?.remote_browser_etag ?? null;
  if (status?.installed && status.update_available && engineTarget && dismissedEngine !== engineTarget) {
    const title = t("engineUpdate.availableTitle");
    const detail = t("engineUpdate.availableDetail", { v: status.remote_chromium_version ?? "" });
    return (
      <div
        data-engine-update="available"
        className={cn("mx-gutter mt-2.5 flex shrink-0 items-center gap-2 rounded-control bg-primary-tint px-3 py-1 text-secondary text-text-strong-950 forced-edge", HAS_CAPTIONS && "me-[114px]")}
      >
        <div className="flex min-w-0 flex-1 items-start gap-2 py-1">
          <InfoIcon aria-hidden="true" className="mt-0.5 size-3.5 shrink-0 text-primary-text" />
          <p role="status" className="m-0 min-w-0 [overflow-wrap:anywhere]">
            <strong className="font-medium">{title}</strong>
            {/[。！？]$/.test(title) ? "" : " "}
            <span className="text-text-sub-600">{detail}</span>
          </p>
        </div>
        <Button variant="neutral" mode="ghost" size="2xsmall" className="shrink-0" onClick={dismissEngineUpdate}>
          {t("launcherUpdate.notNow")}
        </Button>
        <Button variant="primary" mode="filled" size="2xsmall" className="shrink-0" onClick={() => void startEngineUpdate()}>
          {t("engineUpdate.updateNow")}
        </Button>
      </div>
    );
  }

  // Priority 4: Background check found a newer LPBrowser launcher release.
  if (launcherOffer && (launcherDownloading || launcherInstalling || dismissedLauncher !== launcherOffer.version)) {
    const title = launcherInstalling
      ? t("launcherUpdate.installing", { v: launcherOffer.version })
      : launcherDownloading
        ? (launcherProgress?.total
          ? t("launcherUpdate.downloadingSize", { v: launcherOffer.version, progress: fmtBytesProgress(launcherProgress.received, launcherProgress.total) })
          : t("launcherUpdate.downloading", { v: launcherOffer.version }))
        : t("launcherUpdate.bannerAvailable", { v: launcherOffer.version });
    return (
      <div
        data-launcher-update={launcherDownloading ? "downloading" : launcherInstalling ? "installing" : "available"}
        className={cn("mx-gutter mt-2.5 flex shrink-0 items-center gap-2 rounded-control bg-primary-tint px-3 py-1 text-secondary text-text-strong-950 forced-edge", HAS_CAPTIONS && "me-[114px]")}
      >
        <div className="flex min-w-0 flex-1 items-start gap-2 py-1">
          <InfoIcon aria-hidden="true" className="mt-0.5 size-3.5 shrink-0 text-primary-text" />
          <p role="status" className="m-0 min-w-0 [overflow-wrap:anywhere]">
            <strong className="font-medium">{title}</strong>
          </p>
        </div>
        {!launcherDownloading && !launcherInstalling && (
          <>
            <Button variant="neutral" mode="ghost" size="2xsmall" className="shrink-0" onClick={dismissLauncherUpdate}>
              {t("launcherUpdate.notNow")}
            </Button>
            <Button variant="primary" mode="filled" size="2xsmall" className="shrink-0" onClick={() => void offerLauncherUpdate(launcherOffer, true)}>
              {launcherOffer.installable ? t("launcherUpdate.update") : t("launcherUpdate.download")}
            </Button>
          </>
        )}
      </div>
    );
  }

  return null;
}
