import { create } from "zustand";
import { settingsGet, settingsSave } from "./api";
import { toast } from "../../../shared/model/toast";
import { t } from "../../../shared/i18n";

/**
 * Global Ad Blocker state read once at app start and kept in sync with
 * Settings saves and the quick toggle button in the Profiles toolbar.
 */
export const useAdblock = create<{ on: boolean; busy: boolean }>(() => ({
  on: false,
  busy: false,
}));

let asked = false;

/** Reads the global adblock switch once; retries next time if the first read failed. */
export function loadAdblock(): void {
  if (asked) return;
  asked = true;
  void settingsGet().then(
    (s) => useAdblock.setState({ on: s.adblock_enabled === true }),
    () => {
      asked = false;
    },
  );
}

/** Toggles global ad blocking on/off immediately and persists it to settings.json. */
export async function toggleAdblock(): Promise<void> {
  if (useAdblock.getState().busy) return;
  const next = !useAdblock.getState().on;
  useAdblock.setState({ busy: true, on: next });
  try {
    const current = await settingsGet();
    await settingsSave({ ...current, adblock_enabled: next });
    toast.saved(next ? t("adblock.toastOn") : t("adblock.toastOff"));
  } catch (e) {
    useAdblock.setState({ on: !next });
    toast.fail({ title: t("settings.saveFailed"), detail: e });
  } finally {
    useAdblock.setState({ busy: false });
  }
}

/** Current global adblock status (non-reactive). */
export const adblockOn = (): boolean => useAdblock.getState().on;
