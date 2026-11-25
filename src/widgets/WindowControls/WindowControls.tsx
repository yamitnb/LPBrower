import { getCurrentWindow } from "@tauri-apps/api/window";
import { IconButton, Portal, cn } from "@proxyshard/shardx-ui-kit";
import { HOST_OS } from "../../shared/lib/utils";
import { useT } from "../../shared/i18n";

/** Width the three caption buttons take at the right end of the header. */
export const CAPTION_ROOM = 3 * 34;
/** Windows and Linux draw these; macOS has its traffic lights. */
export const HAS_CAPTIONS = HOST_OS !== "macOS";

const Glyph = ({ d }: { d: "min" | "max" | "close" }) => (
  <svg data-caption-glyph="" width="9" height="9" viewBox="0 0 10 10" aria-hidden="true">
    {d === "min" && <line x1="1" y1="5" x2="9" y2="5" stroke="currentColor" strokeWidth="1.1" />}
    {d === "max" && <rect x="1.5" y="1.5" width="7" height="7" fill="none" stroke="currentColor" strokeWidth="1.1" />}
    {d === "close" && (
      <>
        <line x1="1.2" y1="1.2" x2="8.8" y2="8.8" stroke="currentColor" strokeWidth="1.1" />
        <line x1="8.8" y1="1.2" x2="1.2" y2="8.8" stroke="currentColor" strokeWidth="1.1" />
      </>
    )}
  </svg>
);

/// Compact minimize, maximize and close in the window's top-right corner:
/// 34 px wide x 32 px tall, flush with the edge so Close stays an instant corner target.
export function ShellWindowControls({ className }: { className?: string }) {
  const t = useT();
  if (!HAS_CAPTIONS) return null;
  const button = "h-full! w-[34px]! rounded-none! focus-ring-inset";
  return (
    <Portal>
    <div data-captions className={cn("fixed right-0 top-0 z-(--layer-chrome) flex h-[32px]", className)}>
      <IconButton label={t("windowControls.minimize")} tooltipSide="bottom" size="xsmall" static className={button} icon={<Glyph d="min" />} onClick={() => void getCurrentWindow().minimize()} />
      <IconButton label={t("windowControls.maximize")} tooltipSide="bottom" size="xsmall" static className={button} icon={<Glyph d="max" />} onClick={() => void getCurrentWindow().toggleMaximize()} />
      <IconButton label={t("windowControls.close")} tooltipSide="bottom" size="xsmall" static className={cn(button, "hover:bg-error-fill! hover:text-static-white")} icon={<Glyph d="close" />} onClick={() => void getCurrentWindow().close()} />
    </div>
    </Portal>
  );
}
