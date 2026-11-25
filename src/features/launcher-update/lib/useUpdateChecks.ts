import { useEffect } from "react";
import { checkEngineUpdate } from "../../../widgets/FirstRunGate/engineUpdate";
import { checkLauncherUpdate } from "../model/launcherUpdate";

/** Run the first background check after initial paint and profile rows settle. */
const FIRST_CHECK_MS = 4_000;
/** Re-check every 6 hours while the launcher stays open. */
const RECHECK_MS = 6 * 60 * 60 * 1000;

/** Mount once in the Shell: checks for both App Launcher and Chromium engine
 *  updates in the background without delaying startup or interrupting dialogs. */
export function useUpdateChecks(): void {
  useEffect(() => {
    if (!("__TAURI_INTERNALS__" in window)) return;
    let gone = false;
    let timer: ReturnType<typeof setTimeout> | undefined;

    const run = async () => {
      if (gone) return;
      await Promise.allSettled([
        checkEngineUpdate({ silent: true }),
        checkLauncherUpdate({ silent: true }),
      ]);
      if (!gone) timer = setTimeout(() => void run(), RECHECK_MS);
    };

    timer = setTimeout(() => void run(), FIRST_CHECK_MS);
    return () => {
      gone = true;
      clearTimeout(timer);
    };
  }, []);
}
