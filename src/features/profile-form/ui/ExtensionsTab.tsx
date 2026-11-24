import { ExtensionPicker } from "../../manage-profiles";
import { patchForm, type ProfileFormDraft } from "../model/useProfileForm";
import { NavExtensionsIcon } from "../../../shared/icons";

/** Extensions: the library's extensions, wrapped in a crisp studio section card. */
export function ExtensionsTab({ draft }: { draft: ProfileFormDraft }) {
  const count = draft.form.extensions?.length ?? 0;
  return (
    <section
      data-tour="form-extensions"
      className="flex flex-col gap-3 rounded-panel border border-stroke-soft-200 bg-bg-white-0 p-3.5"
    >
      <div className="flex items-center justify-between pb-2 border-b border-stroke-soft-200/70">
        <div className="flex items-center gap-2">
          <NavExtensionsIcon className="size-3.5 text-primary-base" />
          <h3 className="m-0 text-xs font-semibold text-text-strong-950">
            Tiện ích mở rộng
          </h3>
        </div>
        <span className="rounded-full bg-bg-weak-50 px-2 py-0.5 text-[11px] font-medium text-text-sub-600 ring-1 ring-inset ring-stroke-soft-200">
          {count} đã chọn
        </span>
      </div>
      <ExtensionPicker listOnly value={draft.form.extensions} onChange={(extensions) => patchForm({ extensions })} />
    </section>
  );
}
