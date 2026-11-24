import { create } from "zustand";

const KEY = "lpbrower.confirmDirectStart";

function readStored(): boolean {
  try { return localStorage.getItem(KEY) === "on"; } catch { return false; }
}

/// Starting a profile without a proxy skips any prompt by default across all profiles.
export const useDirectStart = create<{ ask: boolean; setAsk: (ask: boolean) => void }>((set) => ({
  ask: readStored(),
  setAsk: (ask) => {
    try {
      if (ask) localStorage.setItem(KEY, "on");
      else localStorage.removeItem(KEY);
    } catch { /* private mode: until reload */ }
    set({ ask });
  },
}));
