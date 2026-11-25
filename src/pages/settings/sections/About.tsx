import { useEffect, useRef } from "react";
import { openUrl } from "@tauri-apps/plugin-opener";
import { Button, ProgressBar } from "@proxyshard/shardx-ui-kit";
import { GH_REPO_URL } from "../../../shared/lib/utils";
import { fmtBytesProgress } from "../../../shared/lib/format";
import { useWhatsNew } from "../../../shared/model/whatsNew";
import { confirmModal } from "../../../shared/model/confirm";
import { toast } from "../../../shared/model/toast";
import { useOnboarding } from "../../../features/onboarding";
import { takeTheTour } from "../../../features/tour";
import {
  askLauncherUpdate,
  checkLauncherUpdate,
  useLauncherUpdate,
} from "../../../features/launcher-update";
import {
  checkEngineUpdate,
  startEngineUpdate,
  useEngineUpdate,
} from "../../../widgets/FirstRunGate/engineUpdate";
import { useT } from "../../../shared/i18n";
import { DetailRow, DetailsCard } from "../../../shared/ui/DetailsCard";

export function AboutSection() {
  const t = useT();
  const openWhatsNew = useWhatsNew((s) => s.setOpen);
  const tipsOff = useOnboarding((s) => s.tipsOff);
  const replayRef = useRef<HTMLButtonElement>(null);

  // App Launcher update state
  const current = useLauncherUpdate((s) => s.current);
  const offer = useLauncherUpdate((s) => s.offer);
  const checking = useLauncherUpdate((s) => s.checking);
  const checked = useLauncherUpdate((s) => s.checked);
  const offline = useLauncherUpdate((s) => s.offline);
  const downloading = useLauncherUpdate((s) => s.downloading);
  const installing = useLauncherUpdate((s) => s.installing);
  const appProgress = useLauncherUpdate((s) => s.progress);

  // Core Chromium engine state
  const engineStatus = useEngineUpdate((s) => s.status);
  const engineChecking = useEngineUpdate((s) => s.checking);
  const engineUpdating = useEngineUpdate((s) => s.retrying);
  const engineProgress = useEngineUpdate((s) => s.progress);

  // Refresh both App Launcher & Core Chromium remote status when About opens.
  useEffect(() => {
    void checkLauncherUpdate({ silent: true });
    void checkEngineUpdate({ silent: true });
  }, []);

  const appStatus = checking && !checked
    ? t("settings.aboutChecking")
    : installing && offer
      ? t("launcherUpdate.installing", { v: offer.version })
      : downloading && offer
        ? (appProgress?.total
          ? t("launcherUpdate.downloadingSize", { v: offer.version, progress: fmtBytesProgress(appProgress.received, appProgress.total) })
          : t("launcherUpdate.downloading", { v: offer.version }))
        : offer
          ? t("settings.aboutUpdate", { v: offer.version })
          : offline
            ? t("settings.aboutOffline")
            : checked
              ? t("settings.aboutUpToDate")
              : t("settings.aboutChecking");

  const installedEngineVer = engineStatus?.installed_chromium_version ?? null;
  const installedBuild = engineStatus?.installed_engine_build ?? null;
  const engineVersionText = !engineStatus
    ? "…"
    : !engineStatus.installed
      ? t("settings.engineNotInstalled")
      : installedEngineVer ?? "…";

  const engineUpdateText = engineUpdating
    ? (engineProgress?.phase === "download"
      ? t("engineUpdate.downloaded", { progress: fmtBytesProgress(engineProgress.received, engineProgress.total) })
      : engineProgress?.phase === "branding"
        ? t("firstRunGate.branding")
        : engineProgress ? t("firstRunGate.extracting") : t("firstRunGate.contactingCdn"))
    : engineChecking && !engineStatus?.remote_checked
      ? t("settings.aboutChecking")
      : !engineStatus
        ? t("settings.aboutChecking")
        : !engineStatus.installed
          ? t("settings.engineNotInstalled")
          : engineStatus.needs_newer_launcher
            ? t("engineUpdate.needsNewerLauncher", { v: engineStatus.min_launcher_version ?? "" })
            : engineStatus.update_available
              ? t("settings.engineUpdateAvailable", { v: engineStatus.remote_chromium_version ?? "" })
              : engineStatus.fingerprints_update_available
                ? t("settings.engineFingerprintsUpdate")
                : engineStatus.remote_checked && !engineStatus.manifest_reachable
                  ? t("settings.aboutOffline")
                  : engineStatus.remote_checked
                    ? t("engineUpdate.upToDate")
                    : t("settings.aboutChecking");

  const hasEngineUpdate = Boolean(
    engineStatus && (!engineStatus.installed || engineStatus.update_available || engineStatus.fingerprints_update_available || engineStatus.update_staged),
  );

  const tipsOn = () => {
    const onboarding = useOnboarding.getState();
    onboarding.markPageSeen("settings");
    onboarding.setTipsOff(false);
    toast.ok(t("settings.aboutTipsOn"));
    replayRef.current?.focus();
  };

  const confirmReinstallEngine = async () => {
    const ok = await confirmModal({
      title: t("settings.engineReinstallTitle"),
      message: t("settings.engineReinstallHelp"),
      confirmLabel: t("settings.engineReinstall"),
    });
    if (ok) await startEngineUpdate({ force: true });
  };

  return (
    <div className="flex flex-col gap-[14px]">
      {/* App Launcher Update Card */}
      <div className="flex flex-col gap-[10px]">
        <DetailsCard id="about" title="LPBrowser">
          <DetailRow id="version" label={t("settings.aboutVersionLabel")} copy={current ? { value: current } : undefined}>
            {current ?? "…"}
          </DetailRow>
          <DetailRow id="updates" label={t("settings.aboutUpdatesLabel")}>
            <span className={offer ? "text-warning-text" : undefined}>{appStatus}</span>
          </DetailRow>
          {downloading && appProgress && (
            <div className="pb-2 pt-0.5">
              <ProgressBar aria-label={t("launcherUpdate.downloading", { v: offer?.version ?? "" })} value={appProgress.percent} color="primary" />
            </div>
          )}
          <DetailRow id="website" label={t("settings.aboutWebsiteLabel")}>
            <a
              href={GH_REPO_URL}
              className="text-primary-text underline underline-offset-2"
              onClick={(e) => { e.preventDefault(); openUrl(GH_REPO_URL).catch(() => {}); }}
            >
              {GH_REPO_URL}
            </a>
          </DetailRow>
        </DetailsCard>
        <div className="flex flex-wrap items-center gap-2">
          {offer && (
            <Button
              variant="primary"
              mode="filled"
              size="xsmall"
              isLoading={downloading || installing}
              onClick={() => void askLauncherUpdate()}
            >
              {t("settings.aboutGetUpdate")}
            </Button>
          )}
          <Button
            variant="neutral"
            mode="stroke"
            size="xsmall"
            isLoading={checking}
            disabled={downloading || installing}
            onClick={() => void checkLauncherUpdate({ silent: false })}
          >
            {t("settings.aboutCheckUpdates")}
          </Button>
          <Button variant="neutral" mode="stroke" size="xsmall" onClick={() => openWhatsNew(true)}>
            {t("whatsNew.title")}
          </Button>
          <Button ref={replayRef} variant="neutral" mode="stroke" size="xsmall" onClick={() => void takeTheTour()}>
            {t("settings.aboutReplayTour")}
          </Button>
          {tipsOff && (
            <Button variant="neutral" mode="ghost" size="xsmall" onClick={tipsOn}>
              {t("settings.aboutTipsAgain")}
            </Button>
          )}
          <Button variant="neutral" mode="ghost" size="xsmall" onClick={() => void openUrl(GH_REPO_URL).catch(() => {})}>
            GitHub
          </Button>
        </div>
      </div>

      {/* Core Chromium Engine Update Card */}
      <div className="flex flex-col gap-[10px]">
        <DetailsCard id="engine" title={t("settings.engineCardTitle")}>
          <DetailRow
            id="engine-version"
            label={t("settings.engineVersionLabel")}
            copy={installedEngineVer ? { value: installedEngineVer } : undefined}
          >
            {engineVersionText}
          </DetailRow>
          {installedBuild && (
            <DetailRow id="engine-build" label={t("settings.engineBuildLabel")}>
              {installedBuild}
            </DetailRow>
          )}
          <DetailRow id="engine-updates" label={t("settings.aboutUpdatesLabel")}>
            <span className={hasEngineUpdate || engineStatus?.needs_newer_launcher ? "text-warning-text" : undefined}>
              {engineUpdateText}
            </span>
          </DetailRow>
          {engineUpdating && engineProgress && (
            <div className="pb-2 pt-0.5">
              <ProgressBar aria-label={t("firstRunGate.downloadBar")} value={engineProgress.percent} color="primary" />
            </div>
          )}
        </DetailsCard>
        <div className="flex flex-wrap items-center gap-2">
          {hasEngineUpdate && !engineStatus?.needs_newer_launcher && (
            <Button
              variant="primary"
              mode="filled"
              size="xsmall"
              isLoading={engineUpdating}
              onClick={() => void startEngineUpdate()}
            >
              {t("engineUpdate.updateNow")}
            </Button>
          )}
          <Button
            variant="neutral"
            mode="stroke"
            size="xsmall"
            isLoading={engineChecking}
            disabled={engineUpdating}
            onClick={() => void checkEngineUpdate({ silent: false })}
          >
            {t("settings.engineCheckUpdates")}
          </Button>
          {engineStatus?.installed && !engineStatus.needs_newer_launcher && (
            <Button
              variant="neutral"
              mode="ghost"
              size="xsmall"
              disabled={engineUpdating || engineChecking}
              onClick={() => void confirmReinstallEngine()}
            >
              {t("settings.engineReinstall")}
            </Button>
          )}
        </div>
      </div>
    </div>
  );
}
