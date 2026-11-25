/* Hallmark · pre-emit critique: P5 H5 E5 S5 R5 V5 */
import { useT } from "../../../shared/i18n";
import { useProfile, type ProfileForm } from "../../../entities/profile";
import {
  useFingerprintPick, OsField, TemplateField,
  HardwareFields, ScreenField, WindowFields, LocaleFields, NoiseFields, PrivacyFields, MediaFields,
} from "../../manage-profiles/ui/fields";
import { patchForm, setFieldInvalid, useProfileForm, type ProfileFormDraft } from "../model/useProfileForm";
import { ComputerIcon, GhostModeIcon, NavFingerprintsIcon } from "../../../shared/icons";

/**
 * Redesigned Fingerprint Tab: clean 2-section structured hardware & anti-detect controls.
 * Pairs with the sticky right-hand ProfileBlueprintCard so every adjustment updates live.
 */
export function FingerprintTab({ draft }: { draft: ProfileFormDraft }) {
  const t = useT();
  const fingerprints = useProfile((s) => s.fingerprints);
  const f = draft.form;
  const u = <K extends keyof ProfileForm>(k: K, v: ProfileForm[K]) => patchForm({ [k]: v } as Partial<ProfileForm>);
  const patchFor = (id: string, changes: Partial<ProfileForm>) => {
    if (useProfileForm.getState().draft?.form.id === id) patchForm(changes);
  };
  const pick = useFingerprintPick(f, patchFor, fingerprints);

  return (
    <div className="flex flex-col gap-3.5">
      {/* Section 1: Hardware & Display */}
      <section className="flex flex-col gap-3 rounded-panel border border-stroke-soft-200 bg-bg-white-0 p-3.5">
        <div className="flex items-center justify-between pb-2 border-b border-stroke-soft-200/70">
          <div className="flex items-center gap-2">
            <ComputerIcon className="size-[19px] text-primary-base" />
            <h3 className="m-0 text-xs font-semibold text-text-strong-950">
              {t("profileForm.groupHardware") || "Phần cứng & Màn hình"}
            </h3>
          </div>
          <span className="rounded-full bg-bg-weak-50 px-2 py-0.5 text-[11px] font-medium text-text-sub-600 ring-1 ring-inset ring-stroke-soft-200">
            {pick.osFilter}
          </span>
        </div>

        {pick.missing && (
          <p data-template-missing className="m-0 text-secondary text-text-sub-600">
            {t("peek.templateMissing")}
          </p>
        )}

        <div data-tour="form-fingerprint" className="flex flex-col gap-3">
          <OsField pick={pick} />
          <TemplateField f={f} u={u} pick={pick} label={t("peek.template")} help={t("profileForm.gpuHelp")} />
        </div>
        <HardwareFields f={f} u={u} os={pick.osFilter} />
        <ScreenField f={f} u={u} os={pick.osFilter} />
        <WindowFields f={f} u={u} />
      </section>

      {/* Section 2: Locale, Noise & Privacy */}
      <section className="flex flex-col gap-3 rounded-panel border border-stroke-soft-200 bg-bg-white-0 p-3.5">
        <div className="flex items-center justify-between pb-2 border-b border-stroke-soft-200/70">
          <div className="flex items-center gap-2">
            <NavFingerprintsIcon className="size-[19px] text-primary-base" />
            <h3 className="m-0 text-xs font-semibold text-text-strong-950">
              {t("profileForm.groupLocale") || "Định vị, Nhiễu (Noise) & WebRTC"}
            </h3>
          </div>
          <span className="inline-flex items-center gap-1.5 text-[11px] text-text-soft-400">
            <GhostModeIcon className="size-[17px]" />
            Fingerprint Protection
          </span>
        </div>

        <div data-tour="form-locale">
          <LocaleFields f={f} u={u} />
        </div>
        <NoiseFields
          f={f}
          u={u}
          help={t("profileForm.noiseHelp") || "Chèn nhiễu số siêu nhỏ vào Canvas, WebGL, Audio và Fonts để chống website nhận diện."}
        />
        <PrivacyFields f={f} u={u} webrtcHelp={t("profileForm.webrtcHelp")} onInvalid={setFieldInvalid} />
        <MediaFields f={f} u={u} />
      </section>

      {pick.warning}
    </div>
  );
}
