import { useCallback } from "react";
import { useTranslation } from "react-i18next";
import { CheckCircle2 } from "@/components/ui/icons";
import { ToggleSwitch } from "@/components/ui/toggle-switch";
import { MascotSprite } from "@/components/mascot/mascot-sprite";
import { useMascotPreviewActive } from "@/hooks/use-mascot-preview-active";
import { useMascotSettings } from "@/hooks/use-mascot-settings";
import { showToast } from "@/lib/toast-emitter";
import { MASCOT_SIZE_MAX, MASCOT_SIZE_MIN, type MascotSettingsPatch } from "@/services/mascot";
import { SettingsCard } from "./settings-card";
import { SettingsRow } from "./settings-row";
import "./mascot-settings.css";

export function MascotSettings() {
  const { t } = useTranslation();
  const { settings, loading, update } = useMascotSettings();
  const previewActive = useMascotPreviewActive();
  const previewWidth = Math.round(92 * settings.size_percent / 100);

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
              width={previewWidth}
            />
          </div>
          <div className="msp-preview-copy">
            <strong>{t("settings.mascot.beaverName")}</strong>
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
        <SettingsCard>
          <div className="msp-choice" aria-current="true">
            <div className="msp-choice-portrait">
              <MascotSprite animation="idle" active={false} width={52} />
            </div>
            <div className="msp-choice-copy">
              <strong>{t("settings.mascot.beaverName")}</strong>
              <span>{t("settings.mascot.beaverDesc")}</span>
            </div>
            <CheckCircle2 className="msp-choice-check" size="var(--icon-lg)" weight="fill" />
          </div>
        </SettingsCard>
      </div>
    </div>
  );
}
