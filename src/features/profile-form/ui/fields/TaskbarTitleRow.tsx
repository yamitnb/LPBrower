import { PropertyRow } from "../../../../shared/ui/peek";
import { useT } from "../../../../shared/i18n";

/// Taskbar Title row: custom label/badge rendered on the browser's taskbar icon
/// and passed as the native OS window title (`--window-name`).
export function TaskbarTitleRow({
  value,
  color,
  onChange,
}: {
  value: string;
  color?: string;
  onChange: (v: string) => void;
}) {
  const t = useT();
  const trimmed = value.trim();
  return (
    <PropertyRow id="taskbar-title" label={t("peek.taskbarTitle") || "Taskbar Title"}>
      {(labelId) => (
        <div className="my-0.5 flex w-full min-w-0 items-center gap-2">
          {/* eslint-disable-next-line no-restricted-syntax -- borderless inline sheet field matching NotesRow */}
          <input
            type="text"
            aria-labelledby={labelId}
            maxLength={24}
            value={value}
            placeholder={t("peek.taskbarTitlePlaceholder") || "Nhãn hiển thị dưới Taskbar (VD: 01, FB-1, Shop...)"}
            onChange={(e) => onChange(e.target.value)}
            className="block h-[28px] min-w-0 flex-1 rounded-control bg-transparent px-1.5 text-ui text-text-strong-950 transition-hover placeholder:text-text-soft-400 focus:bg-bg-hover focus:outline-hidden"
          />
          {trimmed && (
            <span
              title="Xem trước nhãn trên icon Taskbar"
              style={color ? { backgroundColor: color } : undefined}
              className="inline-flex max-w-[96px] shrink-0 items-center justify-center truncate rounded-full bg-primary-base px-2 py-0.5 text-[10.5px] font-bold tracking-tight text-white shadow-2xs ring-1 ring-white/80"
            >
              {trimmed}
            </span>
          )}
        </div>
      )}
    </PropertyRow>
  );
}
