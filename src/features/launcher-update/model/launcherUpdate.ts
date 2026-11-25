import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { openUrl } from "@tauri-apps/plugin-opener";
import { useProfile } from "../../../entities/profile";
import { t } from "../../../shared/i18n";
import { commitFocusedField } from "../../../shared/lib/commitField";
import { fmtBytesProgress } from "../../../shared/lib/format";
import { GH_REPO_URL } from "../../../shared/lib/utils";
import { confirmModal } from "../../../shared/model/confirm";
import { toast } from "../../../shared/model/toast";
import { confirmLeave } from "../../../shared/model/unsaved";
import type { LauncherOffer, RtUpdate } from "../../../shared/types";
import { confirmOpenSessions } from "../../team-sessions";

export type LauncherDownloadProgress = {
  received: number;
  total: number | null;
  percent: number;
};

type LauncherUpdateState = {
  current: string | null;
  offer: LauncherOffer | null;
  checking: boolean;
  checked: boolean;
  offline: boolean;
  downloading: boolean;
  installing: boolean;
  progress: LauncherDownloadProgress | null;
  error: string | null;
  dismissedVersion: string | null;
};

export const useLauncherUpdate = create<LauncherUpdateState>(() => ({
  current: null,
  offer: null,
  checking: false,
  checked: false,
  offline: false,
  downloading: false,
  installing: false,
  progress: null,
  error: null,
  dismissedVersion: null,
}));

let busy = false;

/** Check GitHub Releases (`yamitnb/LPBrower`) for a newer launcher version. */
export async function checkLauncherUpdate(opts?: { silent?: boolean }): Promise<LauncherOffer | null> {
  const silent = opts?.silent ?? false;
  if (useLauncherUpdate.getState().checking) return useLauncherUpdate.getState().offer;
  useLauncherUpdate.setState({ checking: true, error: null });
  try {
    const offer = await invoke<LauncherOffer | null>("launcher_update_offer");
    let current = offer?.current ?? useLauncherUpdate.getState().current;
    if (!current) {
      const info = await invoke<RtUpdate>("launcher_update_check").catch(() => null);
      current = info?.current ?? null;
    }
    useLauncherUpdate.setState({
      current,
      offer,
      checking: false,
      checked: true,
      offline: false,
    });
    if (!silent && !offer) {
      toast.ok(t("settings.aboutUpToDate"));
    }
    return offer;
  } catch (e) {
    let current = useLauncherUpdate.getState().current;
    if (!current) {
      const info = await invoke<RtUpdate>("launcher_update_check").catch(() => null);
      current = info?.current ?? null;
    }
    useLauncherUpdate.setState({
      current,
      checking: false,
      checked: true,
      offline: true,
      error: String(e),
    });
    if (!silent) {
      toast.fail({ title: t("settings.aboutOffline"), detail: e });
    }
    return null;
  }
}

export function dismissLauncherUpdate(): void {
  const v = useLauncherUpdate.getState().offer?.version ?? null;
  if (v) useLauncherUpdate.setState({ dismissedVersion: v });
}

/** Ask the user to confirm updating to `offer`, then download and apply it. */
export async function offerLauncherUpdate(offer: LauncherOffer, asked = false): Promise<void> {
  if (busy) return;
  if (!asked && useLauncherUpdate.getState().dismissedVersion === offer.version) return;
  busy = true;
  try {
    const s = useProfile.getState();
    const running = s.profiles.filter((p) => s.running[p.id]);
    const message = !offer.installable
      ? t("launcherUpdate.helpDownload")
      : running.length > 0 ? t("launcherUpdate.helpRunning") : t("launcherUpdate.help");
    const yes = await confirmModal({
      title: t("launcherUpdate.title", { v: offer.version }),
      message,
      items: offer.installable ? running.map((p) => ({ label: p.name })) : undefined,
      buttons: [
        { label: t("launcherUpdate.notNow"), value: false },
        { label: offer.installable ? t("launcherUpdate.update") : t("launcherUpdate.download"), value: true, primary: true },
      ],
    });
    if (!yes) {
      useLauncherUpdate.setState({ dismissedVersion: offer.version });
      return;
    }
    if (!offer.installable) {
      await openUrl(offer.release_url ?? `${GH_REPO_URL}/releases/latest`).catch(() => {});
      return;
    }
    await downloadAndApply(offer.version);
  } finally {
    busy = false;
  }
}

async function downloadAndApply(v: string): Promise<void> {
  useLauncherUpdate.setState({ downloading: true, installing: false, progress: { received: 0, total: null, percent: 0 }, error: null });
  const p = toast.progress({ title: t("launcherUpdate.downloading", { v }), total: 100 });
  const unlisten = await listen<LauncherDownloadProgress>("launcher-update://progress", (e) => {
    const { received, total, percent } = e.payload;
    useLauncherUpdate.setState({ progress: { received, total, percent } });
    if (total && total > 0) {
      p.update(Math.min(99, percent), t("launcherUpdate.downloadingSize", { v, progress: fmtBytesProgress(received, total) }));
    }
  }).catch(() => () => {});

  try {
    await invoke<string>("launcher_update_download");
  } catch (e) {
    useLauncherUpdate.setState({ downloading: false, progress: null, error: String(e) });
    p.done({ title: t("launcherUpdate.failed"), detail: String(e), failed: 1 });
    return;
  } finally {
    unlisten();
  }
  useLauncherUpdate.setState({ downloading: false });
  p.done({ title: t("launcherUpdate.downloaded", { v }) });

  commitFocusedField();
  if (!(await confirmLeave()) || !(await confirmOpenSessions())) {
    useLauncherUpdate.setState({ progress: null });
    toast.info(t("launcherUpdate.later"));
    return;
  }

  const s = useProfile.getState();
  const ids = Object.keys(s.running).filter((id) => s.running[id]);
  if (ids.length > 0 && (await s.stopMany(ids)).length > 0) {
    useLauncherUpdate.setState({ progress: null });
    toast.fail({ title: t("launcherUpdate.failed"), detail: t("launcherUpdate.stillRunning") });
    return;
  }

  useLauncherUpdate.setState({ installing: true });
  toast.info(t("launcherUpdate.installing", { v }));
  try {
    await invoke("launcher_update_apply");
  } catch (e) {
    useLauncherUpdate.setState({ installing: false, progress: null, error: String(e) });
    toast.fail({ title: t("launcherUpdate.failed"), detail: e });
  }
}

/** Check again and prompt immediately if an update is available (About / What's new). */
export async function askLauncherUpdate(): Promise<void> {
  const existing = useLauncherUpdate.getState().offer;
  const offer = existing ?? await checkLauncherUpdate({ silent: false });
  if (offer) await offerLauncherUpdate(offer, true);
}
