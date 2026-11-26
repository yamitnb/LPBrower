import { cn } from "@proxyshard/shardx-ui-kit";
import type { AdBlockMode } from "../../../../entities/profile";
import { useAdblock } from "../../../../entities/settings";
import { AdBlockOffIcon, AdBlockOnIcon } from "../../../../shared/icons";
import { useT } from "../../../../shared/i18n";
import { PropertyRow } from "../../../../shared/ui/peek";

/** Per-profile Ad Blocker override: inherit global setting, force On, or force Off. */
export function AdBlockRow({
  value,
  onChange,
}: {
  value: AdBlockMode;
  onChange: (v: AdBlockMode) => void;
}) {
  const t = useT();
  const globalOn = useAdblock((s) => s.on);
  const effectiveOn = value === "on" || (value === "default" && globalOn);
  const options: { id: AdBlockMode; label: string }[] = [
    {
      id: "default",
      label: `${t("adblock.modeDefault")} (${globalOn ? t("adblock.modeOn") : t("adblock.modeOff")})`,
    },
    { id: "on", label: t("adblock.modeOn") },
    { id: "off", label: t("adblock.modeOff") },
  ];

  return (
    <PropertyRow id="adblock" label={t("adblock.label")}>
      {(labelId) => (
        <div
          role="radiogroup"
          aria-labelledby={labelId}
          className="my-0.5 inline-flex items-center gap-1 rounded-control border border-stroke-soft-200 bg-bg-weak-50/70 p-0.5"
        >
          {options.map((opt) => {
            const active = value === opt.id;
            return (
              <button
                key={opt.id}
                type="button"
                role="radio"
                aria-checked={active}
                data-adblock-option={opt.id}
                onClick={() => onChange(opt.id)}
                className={cn(
                  "inline-flex h-[24px] cursor-pointer items-center gap-1.5 rounded-[5px] px-2.5 text-[12px] font-medium transition-all outline-hidden",
                  "focus-visible:ring-2 focus-visible:ring-primary-base/25",
                  active
                    ? "bg-bg-white-0 text-text-strong-950 shadow-xs ring-1 ring-inset ring-stroke-soft-200"
                    : "text-text-sub-600 hover:text-text-strong-950",
                )}
              >
                {active && (
                  effectiveOn ? (
                    <AdBlockOnIcon className="size-3.5 text-primary-base" />
                  ) : (
                    <AdBlockOffIcon className="size-3.5 text-icon-sub-600" />
                  )
                )}
                <span>{opt.label}</span>
              </button>
            );
          })}
        </div>
      )}
    </PropertyRow>
  );
}
