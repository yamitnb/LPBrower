import { useMemo, useState } from "react";
import { Button, DialogModal, IconButton, Input, Switch } from "@proxyshard/shardx-ui-kit";
import {
  allQuery, BUILTIN_VIEWS, emptyFilters, leaveList, useFolders, useProfileUi, useViewCounts, useViews,
  VIEW_NAME_MAX, viewName, type ProfileFilters, type ProfileSort, type ProfileView,
} from "../../../entities/profile";
import { useTags, type Tag } from "../../../entities/tag";
import { deleteView } from "../../../features/manage-views";
import { AddIcon, DeleteIcon, FilterIcon } from "../../../shared/icons";
import { SelectField } from "../../../shared/ui/SelectField";
import { TagChip } from "../../../shared/ui/TagChip";
import { ViewIcon } from "../../../shared/ui/ViewIcon";
import { focusAfterRowDelete } from "../rowDelete";
import { loadAllLangs, useT } from "../../../shared/i18n";
import { useNav } from "../../../shared/model/navigation";
import { toast } from "../../../shared/model/toast";
import { SettingsGroup } from "../SettingRow";
import { queueViewOp, usePendingView } from "../useSettingsDraft";

function summarizeView(view: ProfileView, tags: readonly Tag[], t: ReturnType<typeof useT>): string {
  const parts: string[] = [];
  const { folder, filters, sort } = view.query;
  if (folder !== "all") parts.push(`${t("peek.folder")}: ${folder}`);
  if (filters.status === "running") parts.push(`${t("profileFilterBar.statusTitle")}: ${t("profileFilterBar.statusRunning")}`);
  else if (filters.status === "idle") parts.push(`${t("profileFilterBar.statusTitle")}: ${t("profileFilterBar.statusIdle")}`);
  if (filters.proxy === "bound") parts.push(`${t("profileFilterBar.connectionTitle")}: ${t("profileFilterBar.connectionBound")}`);
  else if (filters.proxy === "direct") parts.push(`${t("profileFilterBar.connectionTitle")}: ${t("profileFilterBar.connectionDirect")}`);
  else if (filters.proxy === "shared") parts.push(`${t("profileFilterBar.connectionTitle")}: ${t("profileFilterBar.connectionShared")}`);
  else if (filters.proxy !== "all") parts.push(`${t("profileFilterBar.connectionTitle")}: Proxy`);
  if (filters.tag) {
    const tag = tags.find((x) => x.id === filters.tag);
    if (tag) parts.push(`${t("profileFilterBar.tagTitle")}: ${tag.name}`);
  }
  const lastRunLabel: Partial<Record<ProfileFilters["lastRun"], string>> = {
    day: t("profileFilterBar.lastRunDay"),
    week: t("profileFilterBar.lastRunWeek"),
    month: t("profileFilterBar.lastRunMonth"),
    stale: t("profileFilterBar.lastRunStale"),
    unused: t("profileFilterBar.lastRunUnused"),
    never: t("profileFilterBar.lastRunNever"),
  };
  if (filters.lastRun !== "any" && lastRunLabel[filters.lastRun]) {
    parts.push(`${t("profileFilterBar.lastRunTitle")}: ${lastRunLabel[filters.lastRun]}`);
  }
  if (filters.country) parts.push(`${t("profileFilterBar.countryTitle")}: ${filters.country}`);
  const sortLabel: Partial<Record<ProfileSort, string>> = {
    "name-asc": t("profileFilterBar.orderNameAsc"),
    "name-desc": t("profileFilterBar.orderNameDesc"),
    "created-desc": t("profileFilterBar.orderNewest"),
    "created-asc": t("profileFilterBar.orderOldest"),
    "launched-desc": t("profileFilterBar.orderLaunched"),
    "runtime-desc": t("profileFilterBar.orderRuntime"),
    status: t("profileFilterBar.orderStatus"),
  };
  if (sort !== "added" && sortLabel[sort]) {
    parts.push(`${t("profileFilterBar.orderTitle")}: ${sortLabel[sort]}`);
  }
  return parts.length > 0 ? parts.join(", ") : t("settings.viewSummaryAll");
}

/// The Views saved from the Profiles list or created here, to open, rename,
/// show in the sidebar or delete, plus a read-only overview of built-in Views.
export function ViewsSection() {
  const t = useT();
  const views = useViews((s) => s.views);
  const tags = useTags((s) => s.tags);
  const allViews = useMemo(() => [...BUILTIN_VIEWS, ...views], [views]);
  const counts = useViewCounts(allViews);
  const setSection = useNav((s) => s.setSection);
  const applyView = useProfileUi((s) => s.applyView);
  const [creating, setCreating] = useState(false);

  const openView = (id: string) => {
    void leaveList(() => {
      applyView(id);
      void setSection("profiles");
    });
  };

  return (
    <div className="flex flex-col gap-5">
      <div className="flex items-center justify-between gap-3">
        <p className="m-0 max-w-[72ch] text-balance! text-help text-text-sub-600">{t("settings.viewsHelp")}</p>
        <Button
          variant="primary"
          mode="filled"
          size="xsmall"
          className="shrink-0"
          leftIcon={<AddIcon className="size-3.5" />}
          onClick={() => setCreating(true)}
        >
          {t("settings.viewCreateButton")}
        </Button>
      </div>

      <SettingsGroup list>
        {views.length === 0 ? (
          <p className="m-0 py-6 text-center text-body text-balance text-text-sub-600">{t("settings.viewsEmpty")}</p>
        ) : (
          <ul className="m-0 flex list-none flex-col p-0">
            {views.map((v) => (
              <ViewRow
                key={v.id}
                view={v}
                count={counts[v.id] ?? 0}
                summary={summarizeView(v, tags, t)}
                onOpen={() => openView(v.id)}
              />
            ))}
          </ul>
        )}
      </SettingsGroup>

      <SettingsGroup title={t("settings.viewsBuiltinTitle")} help={t("settings.viewsBuiltinHelp")} list>
        <ul className="m-0 flex list-none flex-col p-0">
          {BUILTIN_VIEWS.map((v) => {
            const count = counts[v.id] ?? 0;
            return (
              <li key={v.id} data-builtin-view={v.id} className="row-line relative flex items-center gap-3 px-cell py-3">
                <span className="flex size-6 shrink-0 items-center justify-center rounded-chip-sm bg-bg-weak-50 text-icon-sub-600">
                  <ViewIcon kind={v.icon} className="size-3.5" />
                </span>
                <div className="min-w-0 flex-1">
                  <div className="truncate text-ui text-text-strong-950">{viewName(v, t)}</div>
                  <div className="truncate text-secondary text-text-soft-400">{summarizeView(v, tags, t)}</div>
                </div>
                <span className="whitespace-nowrap text-secondary tabular-nums text-text-soft-400">
                  {count === 0 ? t("settings.tagsCountNone") : count === 1 ? t("settings.tagsCountOne") : t("settings.tagsCountMany", { n: count })}
                </span>
                <Button variant="neutral" mode="stroke" size="3xsmall" onClick={() => openView(v.id)}>
                  {t("settings.viewOpen")}
                </Button>
              </li>
            );
          })}
        </ul>
      </SettingsGroup>

      {creating && <CreateViewModal open={creating} onClose={() => setCreating(false)} />}
    </div>
  );
}

function ViewRow({ view, count, summary, onOpen }: { view: ProfileView; count: number; summary: string; onOpen: () => void }) {
  const t = useT();
  // What the page shows: the waiting rename and pin over the saved ones.
  const { name, pinned, error } = usePendingView(view);
  const rename = (to: string) => queueViewOp({ kind: "rename", id: view.id, name: to });

  return (
    <li data-view-row={view.id} className="row-line relative flex flex-wrap items-center gap-x-4 gap-y-2 px-cell py-3">
      <div className="flex min-w-[220px] flex-1 flex-col gap-1">
        <div className="w-[240px] max-w-full">
          <Input
            aria-label={t("settings.viewNameFor", { name: view.name })}
            inputSize="xsmall"
            value={name}
            maxLength={VIEW_NAME_MAX}
            error={error}
            onChange={(e) => rename(e.target.value)}
            // An empty name is no name: leaving the field keeps the one it had.
            onBlur={() => { if (!name.trim()) rename(view.name); }}
            onKeyDown={(e) => {
              // An Esc that cancels an input method's text is its own.
              if (e.nativeEvent.isComposing || e.keyCode === 229) return;
              if (e.key === "Escape") rename(view.name);
            }}
          />
        </div>
        <span className="truncate text-secondary text-text-soft-400">{summary}</span>
      </div>
      <Switch
        role="switch"
        aria-label={t("settings.viewPinned", { name: view.name })}
        label={t("settings.viewPinnedShort")}
        checked={pinned}
        onChange={(on) => queueViewOp({ kind: "pin", id: view.id, pinned: on })}
      />
      <span className="whitespace-nowrap text-secondary tabular-nums text-text-soft-400">
        {count === 0 ? t("settings.tagsCountNone") : count === 1 ? t("settings.tagsCountOne") : t("settings.tagsCountMany", { n: count })}
      </span>
      <Button variant="neutral" mode="stroke" size="3xsmall" onClick={onOpen}>
        {t("settings.viewOpen")}
      </Button>
      <IconButton
        data-row-delete
        size="2xsmall"
        icon={<DeleteIcon className="size-3.5" />}
        label={t("settings.viewDelete", { name: view.name })}
        onClick={(e) => {
          const next = focusAfterRowDelete(e.currentTarget);
          deleteView(view.id);
          next();
        }}
      />
    </li>
  );
}

function CreateViewModal({ open, onClose }: { open: boolean; onClose: () => void }) {
  const t = useT();
  const folders = useFolders();
  const tags = useTags((s) => s.tags);
  const [name, setName] = useState("");
  const [folder, setFolder] = useState("all");
  const [status, setStatus] = useState<ProfileFilters["status"]>("all");
  const [proxy, setProxy] = useState<"all" | "bound" | "direct" | "shared">("all");
  const [tag, setTag] = useState("");
  const [lastRun, setLastRun] = useState<ProfileFilters["lastRun"]>("any");
  const [sort, setSort] = useState<ProfileSort>("added");
  const [pinned, setPinned] = useState(true);
  const [taken, setTaken] = useState(false);

  const trimmed = name.trim();
  const dirty = trimmed.length > 0 || folder !== "all" || status !== "all" || proxy !== "all" || tag !== "" || lastRun !== "any" || sort !== "added" || !pinned;

  const submit = async () => {
    if (!trimmed) return;
    await loadAllLangs();
    const base = allQuery();
    const q = {
      ...base,
      folder,
      sort,
      filters: { ...emptyFilters(), status, proxy, tag, lastRun },
    };
    const saved = useViews.getState().save(trimmed, q, pinned);
    if (!saved) {
      setTaken(true);
      return;
    }
    toast.saved(t("viewDialog.saved", { name: saved.name }));
    onClose();
  };

  return (
    <DialogModal
      open={open}
      onClose={onClose}
      dirty={dirty}
      icon={<FilterIcon className="size-3.5" />}
      title={t("settings.viewCreateTitle")}
      confirmLabel={t("viewDialog.save")}
      onConfirm={() => void submit()}
      isDisabled={!trimmed || taken}
      cancelLabel={t("viewDialog.cancel")}
      onCancel={onClose}
    >
      <div className="flex flex-col gap-3.5 py-4">
        <Input
          aria-label={t("viewDialog.name")}
          label={t("viewDialog.name")}
          inputSize="xsmall"
          autoFocus
          value={name}
          maxLength={VIEW_NAME_MAX}
          error={taken ? t("views.nameTaken", { name: trimmed }) : undefined}
          onChange={(e) => { setName(e.target.value); setTaken(false); }}
        />
        <div className="grid grid-cols-2 gap-3">
          <SelectField
            label={t("peek.folder")}
            value={folder}
            onChange={setFolder}
            options={[
              { value: "all", label: t("settings.viewFolderAll") },
              ...folders.map((f) => ({ value: f, label: f })),
            ]}
          />
          <SelectField
            label={t("profileFilterBar.statusTitle")}
            value={status}
            onChange={(v) => setStatus(v as ProfileFilters["status"])}
            options={[
              { value: "all", label: t("settings.viewAny") },
              { value: "running", label: t("profileFilterBar.statusRunning") },
              { value: "idle", label: t("profileFilterBar.statusIdle") },
            ]}
          />
          <SelectField
            label={t("profileFilterBar.connectionTitle")}
            value={proxy}
            onChange={(v) => setProxy(v as "all" | "bound" | "direct" | "shared")}
            options={[
              { value: "all", label: t("settings.viewAny") },
              { value: "bound", label: t("profileFilterBar.connectionBound") },
              { value: "direct", label: t("profileFilterBar.connectionDirect") },
              { value: "shared", label: t("profileFilterBar.connectionShared") },
            ]}
          />
          <SelectField
            label={t("profileFilterBar.tagTitle")}
            value={tag}
            onChange={setTag}
            options={[
              { value: "", label: t("settings.viewAny") },
              ...tags.map((x) => ({ value: x.id, label: x.name, content: <TagChip tag={x} /> })),
            ]}
          />
          <SelectField
            label={t("profileFilterBar.lastRunTitle")}
            value={lastRun}
            onChange={(v) => setLastRun(v as ProfileFilters["lastRun"])}
            options={[
              { value: "any", label: t("settings.viewAny") },
              { value: "day", label: t("profileFilterBar.lastRunDay") },
              { value: "week", label: t("profileFilterBar.lastRunWeek") },
              { value: "month", label: t("profileFilterBar.lastRunMonth") },
              { value: "stale", label: t("profileFilterBar.lastRunStale") },
              { value: "never", label: t("profileFilterBar.lastRunNever") },
            ]}
          />
          <SelectField
            label={t("profileFilterBar.orderTitle")}
            value={sort}
            onChange={(v) => setSort(v as ProfileSort)}
            options={[
              { value: "added", label: t("profileFilterBar.orderAdded") },
              { value: "name-asc", label: t("profileFilterBar.orderNameAsc") },
              { value: "name-desc", label: t("profileFilterBar.orderNameDesc") },
              { value: "created-desc", label: t("profileFilterBar.orderNewest") },
              { value: "created-asc", label: t("profileFilterBar.orderOldest") },
              { value: "launched-desc", label: t("profileFilterBar.orderLaunched") },
              { value: "runtime-desc", label: t("profileFilterBar.orderRuntime") },
              { value: "status", label: t("profileFilterBar.orderStatus") },
            ]}
          />
        </div>
        <Switch role="switch" label={t("viewDialog.pinned")} checked={pinned} onChange={setPinned} />
      </div>
    </DialogModal>
  );
}
