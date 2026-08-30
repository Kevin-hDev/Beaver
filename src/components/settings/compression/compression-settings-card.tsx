import { useCallback, useMemo, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { SettingsCard } from "@/components/settings/settings-card";
import { SettingsRow } from "@/components/settings/settings-row";
import { ToggleSwitch } from "@/components/ui/toggle-switch";
import { useCompressionProfiles } from "@/hooks/use-compression-profiles";
import { useContextProgress } from "@/hooks/use-context-progress";
import { CompressionPanel } from "./compression-panel";
import "../compression-slider.css";
import "./compression-settings-card.css";

interface CompressionSettingsCardProps {
  defaultModel: string;
}

function modelRoute(value: string): { provider: string; model: string } {
  const separator = value.indexOf(":");
  if (separator < 0) return { provider: "ollama", model: value };
  return {
    provider: value.slice(0, separator),
    model: value.slice(separator + 1),
  };
}

export function CompressionSettingsCard({ defaultModel }: CompressionSettingsCardProps) {
  const { t } = useTranslation();
  const controller = useCompressionProfiles();
  const advancedButtonRef = useRef<HTMLButtonElement>(null);
  const [panelOpen, setPanelOpen] = useState(false);
  const route = useMemo(() => modelRoute(defaultModel), [defaultModel]);
  const { max } = useContextProgress(route.model, 0, route.provider);
  const active = controller.view?.profiles.find(
    (profile) => profile.id === controller.view?.global_profile_id,
  );
  const [thresholdDraft, setThresholdDraft] = useState<{
    profileId: string;
    value: number;
  } | null>(null);

  const unavailableUnder64 = max > 0 && max < 64_000 && !active?.allow_under_64k;
  const threshold = active && thresholdDraft?.profileId === active.id
    ? thresholdDraft.value
    : active?.threshold_percent;
  const closePanel = useCallback(() => {
    setPanelOpen(false);
    requestAnimationFrame(() => advancedButtonRef.current?.focus());
  }, []);

  return (
    <SettingsCard className="csc-card">
      <SettingsRow
        title={t("settings.advanced.compressionEnabledTitle")}
        description={t("settings.advanced.compressionEnabledDesc")}
      >
        {controller.view ? (
          <ToggleSwitch
            checked={controller.view.automatic_enabled}
            disabled={controller.busy || unavailableUnder64}
            ariaLabel={t("settings.advanced.compressionEnabledTitle")}
            onCheckedChange={(enabled) => { void controller.setAutomaticEnabled(enabled); }}
          />
        ) : <span className="csc-loading" aria-busy="true">—</span>}
      </SettingsRow>

      <SettingsRow
        title={t("settings.advanced.compressionThresholdTitle")}
        description={t("settings.advanced.compressionThresholdDesc")}
      >
        {active && threshold != null ? (
          <div className="csc-threshold">
            <input
              className="compression-slider csc-slider"
              type="range"
              min={1}
              max={90}
              value={threshold}
              disabled={!controller.view?.automatic_enabled || unavailableUnder64}
              aria-label={t("settings.advanced.compressionThresholdTitle")}
              onChange={(event) => {
                const next = Math.min(90, Math.max(1, Number(event.target.value)));
                setThresholdDraft({ profileId: active.id, value: next });
                void controller.save({ ...active, threshold_percent: next }).then(() => {
                  setThresholdDraft((current) => (
                    current?.profileId === active.id && current.value === next ? null : current
                  ));
                });
              }}
            />
            <span className="csc-value">{threshold}%</span>
          </div>
        ) : <span className="csc-loading" aria-busy="true">—</span>}
      </SettingsRow>

      {unavailableUnder64 && (
        <div className="settings-row csc-note">
          <div className="settings-row-desc">
            {t("settings.advanced.compressionDisabledUnder64")}
          </div>
        </div>
      )}

      <SettingsRow
        title={t("settings.advanced.compressionAdvancedTitle")}
        description={active
          ? t("settings.advanced.compressionAdvancedDesc", { name: active.name })
          : "—"}
      >
        <button
          ref={advancedButtonRef}
          type="button"
          className="btn btn-sm btn-secondary"
          disabled={!controller.view}
          onClick={() => setPanelOpen(true)}
        >
          {t("settings.advanced.compressionAdvanced")}
        </button>
      </SettingsRow>
      {panelOpen && (
        <CompressionPanel
          controller={controller}
          currentWindow={max}
          onClose={closePanel}
        />
      )}
    </SettingsCard>
  );
}
