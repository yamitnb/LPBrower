/* Hallmark · pre-emit critique: P5 H5 E5 S5 R5 V5 */
import { useId, useState } from "react";
import { Input, cn } from "@proxyshard/shardx-ui-kit";
import { PropertySheet } from "../../../shared/ui/peek";
import { useT } from "../../../shared/i18n";
import {
  addTagName, patchDraft, patchForm, setFormOs, setFormPassword, useProfileForm, type ProfileFormDraft,
} from "../model/useProfileForm";
import {
  AndroidOsIcon, AppleOsIcon, CheckCircleIcon, ComputerIcon, KeyIcon, LinuxOsIcon, TagIcon, WindowsOsIcon,
} from "../../../shared/icons";
import { PASSWORD_MIN, passwordProblem, useProfile } from "../../../entities/profile";
import { useProfilePasswords } from "../../../entities/settings";
import { HOST_OS } from "../../../shared/lib/utils";
import { osIdFor } from "../../../shared/constants";
import { ColorRow, CookiesRow, NotesRow } from "./fields";
import { teamRefusal, useInTeam, useRightsChange } from "../../../entities/space";
import { TagsField } from "./TagsField";
import { ProxyTab } from "./ProxyTab";

const OS_CARDS = [
  { id: "Windows", label: "Windows", desc: "Win 10/11", icon: WindowsOsIcon },
  { id: "macOS", label: "macOS", desc: "Apple Silicon", icon: AppleOsIcon },
  { id: "Linux", label: "Linux", desc: "Ubuntu / x64", icon: LinuxOsIcon },
  { id: "Android", label: "Android", desc: "Mobile", icon: AndroidOsIcon },
];

/**
 * Redesigned Basics Workspace (left column of the Studio Dialog):
 * 1. OS Selector (4-up horizontal segmented cards, keeps profile name intact)
 * 2. Proxy & Network configuration (clean protocol bar + smart paste + live test)
 * 3. Organization & Metadata (Tags, Color, Notes, Cookies, optional Password)
 */
export function BasicsTab({ draft }: { draft: ProfileFormDraft }) {
  const t = useT();
  const f = draft.form;

  const fingerprints = useProfile((s) => s.fingerprints);
  const currentFp = fingerprints.find((x) => x.id === f.gpu_preset_id);
  const currentOs = currentFp ? osIdFor(currentFp.platform as string) : HOST_OS;

  const team = useInTeam();
  useRightsChange();
  const locked = useProfileForm((s) => s.unlocked !== null);
  const creating = useProfileForm((s) => s.mode === "create");
  const passwords = useProfilePasswords((s) => s.on);

  return (
    <div className="flex flex-col gap-3.5">
      {/* Section 1: Operating System Selector */}
      <section className="flex flex-col gap-2.5 rounded-panel border border-stroke-soft-200 bg-bg-white-0 p-3.5">
        <div className="flex items-center justify-between gap-2">
          <div className="flex items-center gap-2">
            <ComputerIcon className="size-3.5 text-primary-base" />
            <h3 className="m-0 text-xs font-semibold text-text-strong-950">
              {t("inlineEditor.osLabel") || "Hệ điều hành giả lập"}
            </h3>
          </div>
          <span className="text-[11px] text-text-soft-400">
            Ngẫu nhiên thông số luôn giữ nguyên OS đang chọn
          </span>
        </div>

        <div className="grid grid-cols-2 sm:grid-cols-4 gap-2">
          {OS_CARDS.map(({ id, label, desc, icon: Icon }) => {
            const active = currentOs === id;
            return (
              <button
                key={id}
                type="button"
                aria-pressed={active}
                onClick={() => void setFormOs(id)}
                className={cn(
                  "relative flex items-center gap-2.5 rounded-control border p-2.5 text-left transition-all cursor-pointer outline-hidden",
                  "focus-visible:ring-2 focus-visible:ring-primary-base/25",
                  active
                    ? "border-primary-base bg-primary-base/8 ring-1 ring-primary-base/30"
                    : "border-stroke-soft-200 bg-bg-weak-50/40 hover:bg-bg-weak-50 hover:border-stroke-sub-300",
                )}
              >
                <div
                  className={cn(
                    "flex size-7 shrink-0 items-center justify-center rounded-md transition-colors",
                    active ? "bg-primary-base text-white" : "bg-bg-white-0 text-icon-sub-600 ring-1 ring-inset ring-stroke-soft-200",
                  )}
                >
                  <Icon className="size-3.5" />
                </div>
                <div className="flex flex-col min-w-0 flex-1">
                  <span className={cn("text-xs font-semibold truncate", active ? "text-primary-text" : "text-text-strong-950")}>
                    {label}
                  </span>
                  <span className="text-[10.5px] text-text-soft-400 truncate">
                    {desc}
                  </span>
                </div>
                {active && <CheckCircleIcon className="size-3.5 shrink-0 text-primary-base" />}
              </button>
            );
          })}
        </div>
      </section>

      {/* Section 2: Proxy & Network */}
      <ProxyTab draft={draft} />

      {/* Section 3: Metadata (Tags, Color, Notes, Cookies) */}
      <section className="flex flex-col gap-2 rounded-panel border border-stroke-soft-200 bg-bg-white-0 p-3.5">
        <div className="flex items-center gap-2 pb-1.5 border-b border-stroke-soft-200/70">
          <TagIcon className="size-3.5 text-primary-base" />
          <h3 className="m-0 text-xs font-semibold text-text-strong-950">
            Phân loại, Ghi chú &amp; Cookies
          </h3>
        </div>
        <PropertySheet>
          <div className="flex flex-col gap-[2px]">
            <TagsField
              value={draft.tagIds}
              newNames={draft.newTags}
              onChange={(tagIds) => patchDraft({ tagIds })}
              onNewNamesChange={(newTags) => patchDraft({ newTags })}
              onCreate={teamRefusal("create") ? undefined : addTagName}
            />
            <ColorRow value={f.color} onChange={(color) => patchForm({ color })} />
            <NotesRow value={f.notes} onChange={(notes) => patchForm({ notes })} />
            {!team && !locked && <CookiesRow value={draft.cookies} onChange={(cookies) => patchDraft({ cookies })} />}
          </div>
        </PropertySheet>

        {creating && passwords && <PasswordFields />}
      </section>
    </div>
  );
}

/** Optional password lock for a newly created profile (#127). */
function PasswordFields() {
  const t = useT();
  const password = useProfileForm((s) => s.password);
  const repeat = useProfileForm((s) => s.passwordRepeat);
  const tried = useProfileForm((s) => s.tried);
  const id = useId();
  const [left, setLeft] = useState({ password: false, repeat: false });
  const problem = passwordProblem(password, repeat);
  const short = problem === "short" && (tried || left.password) ? t("errors.passwordTooShort") : undefined;
  const mismatch = problem === "mismatch" && (tried || left.repeat) ? t("profileLock.mismatch") : undefined;
  const field = { inputSize: "xsmall", wrapperClassName: "gap-[6px]", secret: true, type: "password", leftIcon: <KeyIcon className="size-3.5" /> } as const;
  return (
    <section data-form-password aria-labelledby={id} aria-describedby={`${id}-help`} className="flex flex-col gap-3 pt-3 mt-1 border-t border-stroke-soft-200">
      <div className="flex flex-col gap-0.5">
        <h3 id={id} className="m-0 text-xs font-semibold text-text-strong-950">{t("profileForm.passwordTitle")}</h3>
        <p id={`${id}-help`} className="m-0 text-help text-text-sub-600">{t("profileForm.passwordHelp", { n: PASSWORD_MIN })}</p>
      </div>
      <div className="grid grid-cols-2 gap-3">
        <Input
          {...field}
          label={t("profileLock.passwordLabel")}
          value={password}
          error={short}
          aria-invalid={!!short || undefined}
          onChange={(e) => setFormPassword({ password: e.target.value })}
          onBlur={() => setLeft((l) => ({ ...l, password: true }))}
        />
        <Input
          {...field}
          label={t("profileLock.repeatLabel")}
          value={repeat}
          error={mismatch}
          aria-invalid={!!mismatch || undefined}
          onChange={(e) => setFormPassword({ passwordRepeat: e.target.value })}
          onBlur={() => setLeft((l) => ({ ...l, repeat: true }))}
        />
      </div>
    </section>
  );
}
