import { cn } from "@proxyshard/shardx-ui-kit";

/** Default high-contrast LPBrowser Royal Blue used when a profile has no custom accent color. */
export const DEFAULT_TASKBAR_BADGE_BG = "#1D4ED8";

function parseHexRgb(hex: string): [number, number, number] | null {
  const h = hex.trim().replace(/^#/, "");
  if (h.length === 3) {
    const r = parseInt(h[0] + h[0], 16);
    const g = parseInt(h[1] + h[1], 16);
    const b = parseInt(h[2] + h[2], 16);
    return Number.isNaN(r) || Number.isNaN(g) || Number.isNaN(b) ? null : [r, g, b];
  }
  if (h.length === 6) {
    const r = parseInt(h.slice(0, 2), 16);
    const g = parseInt(h.slice(2, 4), 16);
    const b = parseInt(h.slice(4, 6), 16);
    return Number.isNaN(r) || Number.isNaN(g) || Number.isNaN(b) ? null : [r, g, b];
  }
  return null;
}

function wcagLuminance(r: number, g: number, b: number): number {
  const lin = (c: number) => {
    const v = c / 255;
    return v <= 0.03928 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4;
  };
  return 0.2126 * lin(r) + 0.7152 * lin(g) + 0.0722 * lin(b);
}

function contrastRatio(l1: number, l2: number): number {
  const hi = Math.max(l1, l2);
  const lo = Math.min(l1, l2);
  return (hi + 0.05) / (lo + 0.05);
}

/**
 * Resolves a guaranteed high-contrast `(backgroundColor, color)` pair matching
 * the OS taskbar badge renderer (`src-tauri/src/profile_icon.rs`).
 */
export function resolveTaskbarBadgeColors(rawColor?: string): { backgroundColor: string; color: string } {
  const rgb = rawColor ? parseHexRgb(rawColor) : null;
  if (!rgb) {
    return { backgroundColor: DEFAULT_TASKBAR_BADGE_BG, color: "#FFFFFF" };
  }
  let [r, g, b] = rgb;
  let lum = wcagLuminance(r, g, b);
  // Truly bright yellows/limes use deep obsidian ink for > 9:1 readability.
  if (lum > 0.42) {
    return { backgroundColor: `rgb(${r}, ${g}, ${b})`, color: "#0B1220" };
  }
  // Mid-tone saturated colors (orange, green, sky, bright blue) darken gently
  // until pure white text reaches >= 4.5:1 WCAG AA contrast without turning muddy.
  for (let i = 0; i < 5 && contrastRatio(1, lum) < 4.5; i++) {
    r = Math.round(r * 0.88);
    g = Math.round(g * 0.88);
    b = Math.round(b * 0.88);
    lum = wcagLuminance(r, g, b);
  }
  return { backgroundColor: `rgb(${r}, ${g}, ${b})`, color: "#FFFFFF" };
}

/** High-contrast, clearly legible Taskbar Title badge preview matching the OS taskbar icon. */
export function TaskbarTitleBadge({
  value,
  color,
  title,
  size = "md",
  className,
}: {
  value: string;
  color?: string;
  title?: string;
  size?: "sm" | "md";
  className?: string;
}) {
  const style = resolveTaskbarBadgeColors(color);
  return (
    <span
      title={title}
      style={style}
      className={cn(
        "inline-flex shrink-0 items-center justify-center truncate rounded-md font-bold tracking-[0.02em] shadow-xs ring-1 ring-inset ring-black/15",
        size === "sm"
          ? "max-w-[132px] px-2.5 py-0.5 text-[11.5px] leading-[16px]"
          : "max-w-[160px] px-2.5 py-[3px] text-[12.5px] leading-[18px]",
        className,
      )}
    >
      {value}
    </span>
  );
}
