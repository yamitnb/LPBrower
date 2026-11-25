import { PropertyRow } from "../../../../shared/ui/peek";
import { TaskbarTitleBadge } from "../../../../shared/ui/TaskbarTitleBadge";
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
          {/* eslint-disable-next-line no-restricted-syntax -- crisp inline input with live high-contrast badge preview */}
          <input
            type="text"
            aria-labelledby={labelId}
            maxLength={24}
            value={value}
            placeholder={t("peek.taskbarTitlePlaceholder") || "Nhãn hiển thị dưới Taskbar (VD: 01, FB-1, Shop...)"}
            onChange={(e) => onChange(e.target.value)}
            className="block h-[30px] min-w-0 flex-1 rounded-control border border-stroke-soft-200 bg-bg-weak-50/60 px-2 text-[13px] font-medium text-text-strong-950 transition-hover placeholder:font-normal placeholder:text-text-sub-600/70 hover:border-stroke-sub-300 focus:border-primary-base focus:bg-bg-white-0 focus:outline-hidden"
          />
          {trimmed && (
            <TaskbarTitleBadge
              value={trimmed}
              color={color}
              title="Xem trước nhãn trên icon Taskbar"
              size="md"
            />
          )}
        </div>
      )}
    </PropertyRow>
  );
}
