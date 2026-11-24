import { useEffect, useRef } from "react";
import { helperProfiles, helperShow, helperClose } from "../../entities/profile/model/api";
import { useProfile } from "../../entities/profile";

/**
 * Opens and closes the Shard Helper panel as pages come and go. Renders nothing;
 * lives in the main window because the panel cannot watch for its own reason to
 * exist. Polls only while at least one browser profile is running so idle CPU
 * and IPC traffic stay at zero.
 */
export function HelperWatcher() {
  const shownFor = useRef<string | null>(null);
  const hasRunning = useProfile((s) => Object.keys(s.running).length > 0);

  useEffect(() => {
    if (!hasRunning) {
      if (shownFor.current !== null) {
        shownFor.current = null;
        void helperClose().catch(() => {});
      }
      return;
    }
    let alive = true;
    const tick = async () => {
      try {
        const profiles = await helperProfiles();
        if (!alive) return;
        // One panel, so one profile: two would fight for the same corner.
        const next = profiles[0] ?? null;
        if (next === shownFor.current) return;
        shownFor.current = next;
        if (next) await helperShow(next);
        else await helperClose();
      } catch { /* the bus may not be up yet */ }
    };
    void tick();
    const id = setInterval(tick, 1500);
    return () => { alive = false; clearInterval(id); };
  }, [hasRunning]);

  return null;
}
