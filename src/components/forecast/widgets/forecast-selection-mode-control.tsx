import { useTranslation } from "react-i18next";
import { ToggleSwitch } from "@/components/ui/toggle-switch";
import type { ForecastSelectionMode } from "../model-selection/forecast-selection-types";

interface ForecastSelectionModeControlProps {
  mode: ForecastSelectionMode;
  allowCloud: boolean;
  ready: boolean;
  onModeChange: (mode: ForecastSelectionMode) => void;
  onCloudAllowedChange: (allowed: boolean) => void;
}

export function ForecastSelectionModeControl({
  mode,
  allowCloud,
  ready,
  onModeChange,
  onCloudAllowedChange,
}: ForecastSelectionModeControlProps) {
  const { t } = useTranslation();

  return (
    <div className="fmsel-mode" aria-label={t("forecast.selection.label")}>
      <div className="fmsel-mode-row">
        <span className="fmsel-mode-label">{t("forecast.selection.label")}</span>
        <div className="fmsel-mode-options">
          {(["manual", "auto"] as const).map((candidate) => (
            <button
              key={candidate}
              type="button"
              disabled={!ready}
              className={`fmsel-mode-option ${mode === candidate ? "is-active" : ""}`}
              aria-pressed={mode === candidate}
              onClick={() => onModeChange(candidate)}
            >
              {t(`forecast.selection.${candidate}`)}
            </button>
          ))}
        </div>
      </div>
      {mode === "auto" && (
        <div className="fmsel-cloud">
          <span>{t("forecast.selection.allowCloud")}</span>
          <ToggleSwitch
            checked={allowCloud}
            onCheckedChange={onCloudAllowedChange}
            ariaLabel={t("forecast.selection.allowCloud")}
            disabled={!ready}
          />
        </div>
      )}
    </div>
  );
}
