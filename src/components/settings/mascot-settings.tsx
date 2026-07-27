import { useCallback } from "react";
import { useTranslation } from "react-i18next";
import { CheckCircle2 } from "@/components/ui/icons";
import { ToggleSwitch } from "@/components/ui/toggle-switch";
import { MascotSprite } from "@/components/mascot/mascot-sprite";
import { useMascotPreviewActive } from "@/hooks/use-mascot-preview-active";
import { useMascotSettings } from "@/hooks/use-mascot-settings";
import { showToast } from "@/lib/toast-emitter";
import { cn } from "@/lib/utils";
import { MASCOT_SIZE_MAX, MASCOT_SIZE_MIN, type MascotSettingsPatch } from "@/services/mascot";
import type { MascotId } from "@/types/mascot";
import { SettingsCard } from "./settings-card";
import { SettingsRow } from "./settings-row";
import "./mascot-settings.css";
import "./mascot-picker.css";

const MASCOT_OPTIONS: Array<{
  id: MascotId;
  nameKey: string;
  descriptionKey: string;
}> = [
  {
    id: "cl-go-beaver",
    nameKey: "settings.mascot.beaverName",
    descriptionKey: "settings.mascot.beaverDesc",
  },
  {
    id: "circuit",
    nameKey: "settings.mascot.circuitName",
    descriptionKey: "settings.mascot.circuitDesc",
  },
];

export function MascotSettings() {
  const { t } = useTranslation();
  const { settings, loading, update } = useMascotSettings();
  const previewActive = useMascotPreviewActive();
  const previewWidth = Math.round(92 * settings.size_percent / 100);
  const selectedMascot = MASCOT_OPTIONS.find(({ id }) => id === settings.mascot_id)
    ?? MASCOT_OPTIONS[0];

  const save = useCallback((patch: MascotSettingsPatch) => {
    void update(patch).catch(() => showToast(t("errors.saveFailed"), "error"));
  }, [t, update]);

  return (
    <div className="msp-page">
      <div className="msp-content">
        <h2 className="msp-title">{t("settings.tabs.mascot")}</h2>

        <section className="msp-preview" aria-label={t("settings.mascot.previewTitle")}>
          <div className="msp-bubble">
            <MascotSprite
              animation="idle"
              active={previewActive}
              mascotId={settings.mascot_id}
              width={previewWidth}
            />
          </div>
          <div className="msp-preview-copy">
            <strong>{t(selectedMascot.nameKey)}</strong>
            <span>
              {previewActive
                ? t("settings.mascot.previewActive")
                : t("settings.mascot.previewPaused")}
            </span>
          </div>
        </section>

        <h3 className="msp-section-title">{t("settings.mascot.settingsTitle")}</h3>
        <SettingsCard>
          <SettingsRow
            title={t("settings.mascot.enabledTitle")}
            description={t("settings.mascot.enabledDesc")}
          >
            <ToggleSwitch
              checked={settings.enabled}
              disabled={loading}
              ariaLabel={t("settings.mascot.enabledTitle")}
              onCheckedChange={(enabled) => save({ enabled })}
            />
          </SettingsRow>
          <SettingsRow
            title={t("settings.mascot.sizeTitle")}
            description={t("settings.mascot.sizeDesc")}
          >
            <div className="msp-size-control">
              <input
                className="msp-size-slider"
                type="range"
                min={MASCOT_SIZE_MIN}
                max={MASCOT_SIZE_MAX}
                value={settings.size_percent}
                aria-label={t("settings.mascot.sizeTitle")}
                onChange={(event) => save({ size_percent: Number(event.target.value) })}
              />
              <span>{settings.size_percent}%</span>
            </div>
          </SettingsRow>
        </SettingsCard>

        <h3 className="msp-section-title">{t("settings.mascot.collectionTitle")}</h3>
        <div className="msp-choice-grid" aria-label={t("settings.mascot.collectionTitle")}>
          {MASCOT_OPTIONS.map((option) => {
            const selected = settings.mascot_id === option.id;
            return (
              <button
                key={option.id}
                type="button"
                className="msp-choice"
                aria-pressed={selected}
                data-selected={selected ? "true" : "false"}
                disabled={loading}
                onClick={() => save({ mascot_id: option.id })}
              >
                <span className="msp-choice-portrait">
                  <MascotSprite
                    animation="idle"
                    active={false}
                    mascotId={option.id}
                    width={58}
                  />
                </span>
                <span className="msp-choice-copy">
                  <strong>{t(option.nameKey)}</strong>
                  <span>{t(option.descriptionKey)}</span>
                </span>
                <span
                  className={cn("msp-choice-status", selected && "is-visible")}
                  aria-hidden={!selected}
                >
                  <CheckCircle2 size="var(--icon-md)" weight="fill" />
                  {t("settings.mascot.selected")}
                </span>
              </button>
            );
          })}
        </div>
      </div>
    </div>
  );
}
