/* Hallmark · pre-emit critique: P5 H5 E5 S5 R5 V5 */
import { useEffect, useRef, useState } from "react";
import { Button, DialogModal, cn } from "@proxyshard/shardx-ui-kit";
import {
  ChevronDownIcon, DisplayIcon, FolderIcon, NavExtensionsIcon, NavFingerprintsIcon,
  RefreshIcon, RegenerateIcon,
} from "../../../shared/icons";
import { isMod } from "../../../shared/lib/keyboard";
import { commitFocusedField } from "../../../shared/lib/commitField";
import { useUnsaved } from "../../../shared/model/unsaved";
import { LoadError, LoadingRows } from "../../../shared/ui/ListStates";
import { useT } from "../../../shared/i18n";
import { settingsRefusal, useFolderMeta, useFolders, useProfile } from "../../../entities/profile";
import { adminRefusal, inFolder, teamRefusal, useRightsChange } from "../../../entities/space";
import { FolderChip } from "../../../shared/ui/FolderChip";
import { hasOwn } from "../../../shared/lib/utils";
import { ComboPopover } from "./fields";
import {
  PROFILE_FORM_TABS, canSave, changedTabs, closeProfileForm, isDirty, nextName, openProfileForm,
  passwordProblemOf, patchDraft, patchForm, problemsOf, randomizeFingerprint, readOnlyWhy, reloadProfileForm,
  rightsProblem, saveProfileForm, setProfileFormTab, useProfileForm, type ProfileFormTab,
} from "../model/useProfileForm";
import { BasicsTab } from "./BasicsTab";
import { FingerprintTab } from "./FingerprintTab";
import { ExtensionsTab } from "./ExtensionsTab";
import { ProfileBlueprintCard } from "./ProfileBlueprintCard";

/** Compact inline folder selector pill for the modal command bar. */
function InlineFolderPicker({
  value,
  onChange,
  keep,
  disabled,
}: {
  value: string;
  onChange: (folder: string) => void;
  keep?: string;
  disabled?: boolean;
}) {
  const t = useT();
  useRightsChange();
  const refusedIn = (name: string) => (name === keep ? null : teamRefusal("create", inFolder(name)));
  const folders = useFolders();
  const metaByName = useFolderMeta((s) => s.byName);
  const [open, setOpen] = useState(false);
  const anchor = useRef<HTMLButtonElement>(null);
  const colorOf = (name: string) => (hasOwn(metaByName, name) ? metaByName[name].color : null);

  const options = [
    { value: "", label: t("peek.noFolder"), disabled: !!refusedIn(""), hint: refusedIn("") ? t("rights.noCreateHere") : undefined },
    ...[...folders, ...(value && !folders.includes(value) ? [value] : [])].map((name) => ({
      value: name,
      label: name,
      content: <FolderChip name={name} color={colorOf(name)} />,
      disabled: !!refusedIn(name),
      hint: refusedIn(name) ? t("rights.noCreateHere") : undefined,
    })),
  ];

  return (
    <>
      <button
        ref={anchor}
        type="button"
        disabled={disabled}
        aria-expanded={open}
        aria-label={t("peek.folder")}
        onClick={() => setOpen((v) => !v)}
        className={cn(
          "inline-flex h-[34px] items-center justify-between gap-2 rounded-control border border-stroke-soft-200 bg-bg-white-0 px-2.5 text-xs transition-colors outline-hidden cursor-pointer",
          "hover:border-stroke-sub-300 focus-visible:border-primary-base focus-visible:ring-2 focus-visible:ring-primary-base/15",
          "disabled:opacity-50 disabled:cursor-not-allowed min-w-[160px] max-w-[210px]",
        )}
      >
        <span className="flex items-center gap-1.5 min-w-0">
          {value ? (
            <FolderChip name={value} color={colorOf(value)} />
          ) : (
            <>
              <FolderIcon className="size-3.5 shrink-0 text-icon-soft-400" />
              <span className="truncate text-text-soft-400">{t("peek.noFolder") || "Chọn thư mục"}</span>
            </>
          )}
        </span>
        <ChevronDownIcon className="size-3.5 shrink-0 text-icon-soft-400" />
      </button>
      <ComboPopover
        open={open}
        onOpenChange={setOpen}
        anchorRef={anchor}
        label={t("peek.folder")}
        value={value}
        options={options}
        placeholder={t("folderPicker.find")}
        onChange={(v) => { if (v !== value) onChange(v); }}
        onCreate={adminRefusal() ? undefined : (typed) => { const n = typed.trim(); if (n) onChange(n); }}
        createLabel={(n) => t("folderPicker.create", { name: n })}
      />
    </>
  );
}

/**
 * Redesigned Create / Edit Profile Studio Dialog:
 * - Top Command Bar: Live `LPB|<Name>` input + auto-name button + inline Folder pill + Randomize CTA
 * - Segmented Pill Navigation: Cơ bản | Fingerprint | Tiện ích mở rộng
 * - Split Studio Body: Scrollable workspace (left) + Pinned Live Blueprint Inspector (right)
 */
export function ProfileFormDialog() {
  const t = useT();
  const s = useProfileForm();
  const { open, mode, tab, draft, status, error, session } = s;
  const nameRef = useRef<HTMLInputElement>(null);
  const dirty = isDirty(s);
  const changed = changedTabs(s);
  const problems = problemsOf(draft);
  const passwordProblem = passwordProblemOf(s);
  const saving = status === "saving";
  const listed = useProfile((st) => st.profiles.find((p) => p.id === s.id));
  const name = s.saved?.form.name ?? listed?.name ?? "";

  useRightsChange();
  const readOnly = listed ? readOnlyWhy(s) : null;
  const refused = readOnly ? null : rightsProblem(s);
  const hidden = mode === "edit" && listed ? settingsRefusal(listed) : null;

  useUnsaved(open && dirty ? {
    id: "profile-form",
    label: mode === "create" ? (t("profileForm.newTitle") || "Tạo hồ sơ mới") : name,
    save: saveProfileForm,
    discard: closeProfileForm,
  } : null);

  useEffect(() => {
    if (!open) return;
    const onKey = (e: KeyboardEvent) => {
      if (!isMod(e) || e.altKey || e.shiftKey || e.key.toLowerCase() !== "s") return;
      e.preventDefault();
      commitFocusedField();
      if (!e.repeat && canSave()) void saveProfileForm();
    };
    window.addEventListener("keydown", onKey, true);
    return () => window.removeEventListener("keydown", onKey, true);
  }, [open]);

  const ready = !!draft;
  useEffect(() => {
    if (!open || !ready || mode !== "create") return;
    const id = requestAnimationFrame(() => { nameRef.current?.focus(); nameRef.current?.select(); });
    return () => cancelAnimationFrame(id);
  }, [open, ready, mode, session]);

  const navTabs: { value: ProfileFormTab; label: string; icon: typeof DisplayIcon }[] = [
    { value: "basics", label: t("profileForm.tabBasics") || "Cơ bản", icon: DisplayIcon },
    { value: "fingerprint", label: "Fingerprint", icon: NavFingerprintsIcon },
    { value: "extensions", label: t("profileForm.tabExtensions") || "Tiện ích", icon: NavExtensionsIcon },
  ];

  const activeTab: ProfileFormTab = PROFILE_FORM_TABS.includes(tab) ? tab : "basics";

  const note = status === "conflict" ? (
    <span role="alert">
      {t("profileForm.conflict")}{" "}
      {s.theirs && <Button mode="link" size="3xsmall" onClick={reloadProfileForm}>{t("profileForm.reload")}</Button>}
    </span>
  ) : status === "error" && draft ? (
    <span role="alert" className="flex flex-col">
      <span className="text-error-text">{error}</span>
      {s.errorDetail && <span data-error-detail>{s.errorDetail}</span>}
    </span>
  ) : readOnly && draft ? <span data-read-only>{t("profileForm.readOnly", { why: readOnly })}</span>
    : saving || !draft ? null
    : problems.name ? t("profileForm.nameRequired")
    : passwordProblem ? <span data-password-problem>{passwordProblem}</span>
    : refused ? <span data-rights-problem>{refused}</span>
    : s.badFields.length > 0 ? t("profileForm.numberInvalid")
    : mode === "edit" && !dirty ? t("profileForm.noChanges")
    : null;

  return (
    <DialogModal
      open={open}
      onOpenChange={(o) => { if (!o) closeProfileForm(); }}
      title={mode === "create" ? (t("profileForm.createProfileTitle") || "Tạo hồ sơ mới") : readOnly ? t("profileForm.viewTitle", { name }) : (t("profileForm.editTitle", { name }) || `Chỉnh sửa ${name}`)}
      size="xl"
      maxWidthClassName="max-w-[1060px] w-full"
      className="max-h-[calc(var(--viewport-h)*0.92)]! h-[88vh] min-h-[640px] flex flex-col"
      dirty={dirty}
      initialFocusRef={nameRef}
      {...(readOnly ? { cancelLabel: t("profileForm.close") } : {
        onConfirm: () => { commitFocusedField(); void saveProfileForm(); },
        confirmLabel: saving ? t("profileForm.saving") : mode === "create" ? (t("profileForm.create") || "Tạo hồ sơ") : (t("profileForm.save") || "Lưu thay đổi"),
      })}
      isLoading={saving}
      isDisabled={!canSave(s)}
      footerNote={note}
    >
      {draft ? (
        <div data-profile-form className="flex flex-col gap-3.5">
          {/* Top Command Strip: LPB|Name + Folder + Randomize */}
          <div className="flex flex-wrap items-center gap-2.5 rounded-panel border border-stroke-soft-200 bg-bg-weak-50/70 p-2.5">
            {/* Clean Profile Name Input Group */}
            <div className="flex flex-1 min-w-[240px] items-center rounded-control border border-stroke-soft-200 bg-bg-white-0 focus-within:border-primary-base focus-within:ring-2 focus-within:ring-primary-base/15 transition-all h-[34px] overflow-hidden">
              <input
                ref={nameRef}
                type="text"
                required
                disabled={saving || !!readOnly}
                aria-label={t("profileForm.name") || "Tên hồ sơ"}
                aria-invalid={!!problems.name || undefined}
                value={draft.form.name}
                placeholder="Nhập tên hồ sơ (VD: Ads-01, Shopee...)"
                onChange={(e) => patchForm({ name: e.target.value })}
                className="h-full min-w-0 flex-1 bg-transparent px-3 text-[13px] font-medium text-text-strong-950 placeholder:text-text-soft-400 placeholder:font-normal outline-hidden"
              />
              <button
                type="button"
                disabled={saving || !!readOnly}
                onClick={() => patchForm({ name: nextName() })}
                title="Gợi ý tên tự động tiếp theo"
                aria-label="Gợi ý tên tự động tiếp theo"
                className="flex h-full items-center px-2.5 text-icon-soft-400 hover:text-text-strong-950 hover:bg-bg-weak-50 transition-colors cursor-pointer disabled:opacity-40 border-l border-stroke-soft-200/70"
              >
                <RefreshIcon className="size-3.5" />
              </button>
            </div>

            {/* Inline Folder Picker */}
            <InlineFolderPicker
              value={draft.folder}
              keep={mode === "edit" ? s.saved?.folder : undefined}
              onChange={(folder) => patchDraft({ folder })}
              disabled={saving || !!readOnly}
            />

            {/* Primary Quick Action: Randomize Specs (locks OS) */}
            <Button
              type="button"
              variant="primary"
              mode="lighter"
              size="xsmall"
              className="shrink-0 h-[34px] px-3 font-medium"
              disabled={saving || !!readOnly}
              leftIcon={<RegenerateIcon className="size-3.5" />}
              onClick={() => void randomizeFingerprint()}
              title="Tạo ngẫu nhiên lại thông số cấu hình và giữ nguyên hệ điều hành đang chọn"
            >
              {t("profileForm.randomizeSpecs") || "Ngẫu nhiên thông số"}
            </Button>
          </div>

          {problems.name && s.tried && (
            <p role="alert" className="m-0 -mt-2 px-1 text-xs text-error-text">
              {problems.name}
            </p>
          )}

          {/* Segmented Tab Bar */}
          <div
            role="tablist"
            aria-label={t("profileForm.tabsLabel")}
            className="flex items-center gap-1 border-b border-stroke-soft-200 pb-2"
          >
            {navTabs.map(({ value, label, icon: Icon }) => {
              const selected = activeTab === value;
              const hasError = value === "basics" && (!!problems.name || (!!passwordProblem && s.tried));
              const isChanged = changed.has(value) || (value === "basics" && changed.has("proxy"));
              return (
                <button
                  key={value}
                  type="button"
                  role="tab"
                  aria-selected={selected}
                  onClick={() => setProfileFormTab(value)}
                  className={cn(
                    "inline-flex items-center gap-1.5 rounded-control px-3 py-1.5 text-xs font-medium transition-all cursor-pointer outline-hidden",
                    "focus-visible:ring-2 focus-visible:ring-primary-base/30",
                    selected
                      ? "bg-primary-base text-white shadow-2xs"
                      : "text-text-sub-600 hover:bg-bg-weak-50 hover:text-text-strong-950",
                  )}
                >
                  <Icon className={cn("size-[18px]", selected ? "text-white" : "text-icon-sub-600")} />
                  <span>{label}</span>
                  {hasError ? (
                    <span className="size-1.5 rounded-full bg-error-base" aria-label={t("profileForm.tabError")} />
                  ) : isChanged ? (
                    <span className={cn("size-1.5 rounded-full", selected ? "bg-white/80" : "bg-primary-base")} aria-label={t("profileForm.tabChanged")} />
                  ) : null}
                </button>
              );
            })}
          </div>

          {/* Two-Zone Studio Grid: Left Active Workspace + Right Sticky Blueprint Inspector */}
          <div className="grid grid-cols-1 lg:grid-cols-[minmax(0,1fr)_310px] gap-4 items-start">
            <fieldset
              disabled={saving || !!readOnly}
              data-read-only={readOnly ? "" : undefined}
              className="m-0 min-w-0 border-0 p-0"
            >
              {activeTab !== "basics" && hidden ? (
                <p data-settings-hidden className="m-0 rounded-panel border border-stroke-soft-200 bg-bg-weak-50 p-4 text-secondary text-text-sub-600">
                  {hidden}
                </p>
              ) : activeTab === "fingerprint" ? (
                <FingerprintTab key={session} draft={draft} />
              ) : activeTab === "extensions" ? (
                <ExtensionsTab draft={draft} />
              ) : (
                <BasicsTab draft={draft} />
              )}
            </fieldset>

            {/* Right Sticky Live Blueprint Card */}
            <div className="lg:sticky lg:top-0">
              <ProfileBlueprintCard draft={draft} readOnly={!!readOnly} saving={saving} />
            </div>
          </div>
        </div>
      ) : status === "error" ? (
        <LoadError
          headingLevel={3}
          className="py-4"
          what={t("otherStates.profile")}
          error={error}
          onRetry={() => openProfileForm(s.mode === "edit" && s.id ? { mode: "edit", id: s.id, tab } : { mode: "create", tab })}
        />
      ) : (
        <div className="py-4"><LoadingRows rows={5} rowHeight={44} /></div>
      )}
    </DialogModal>
  );
}
