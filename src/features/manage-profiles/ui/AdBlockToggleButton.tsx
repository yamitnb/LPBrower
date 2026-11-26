import { Button, Tooltip, cn } from "@proxyshard/shardx-ui-kit";
import { AdBlockOffIcon, AdBlockOnIcon } from "../../../shared/icons";
import { toggleAdblock, useAdblock } from "../../../entities/settings";
import { useT } from "../../../shared/i18n";

/** Quick one-click Ad Blocker toggle in the Profiles toolbar. */
export function AdBlockToggleButton() {
  const t = useT();
  const on = useAdblock((s) => s.on);
  const busy = useAdblock((s) => s.busy);

  return (
    <Tooltip content={on ? t("adblock.toolbarTipOn") : t("adblock.toolbarTipOff")}>
      <Button
        variant={on ? "primary" : "neutral"}
        mode={on ? "lighter" : "stroke"}
        size="xsmall"
        disabled={busy}
        aria-pressed={on}
        data-adblock-toggle={on ? "on" : "off"}
        className={cn(
          on && "border border-primary-base/30",
        )}
        leftIcon={
          on ? (
            <AdBlockOnIcon className="size-3.5 text-primary-base" />
          ) : (
            <AdBlockOffIcon className="size-3.5 text-icon-sub-600" />
          )
        }
        onClick={() => void toggleAdblock()}
      >
        <span className="inline-flex items-center gap-1.5">
          <span>{t("adblock.toolbarLabel")}</span>
          <span
            aria-hidden="true"
            className={cn(
              "size-1.5 rounded-full transition-colors",
              on ? "bg-success-base" : "bg-text-soft-400/60",
            )}
          />
        </span>
      </Button>
    </Tooltip>
  );
}
