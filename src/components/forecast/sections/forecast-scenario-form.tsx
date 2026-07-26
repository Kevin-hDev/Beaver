import type { FormEvent } from "react";
import { useTranslation } from "react-i18next";
import { ForecastScenarioContextFields } from "./forecast-scenario-context-fields";
import type { ForecastScenarioCovariateAdjustment } from "./forecast-scenario-types";

type ScenarioMode = "percent_adjustment" | "context_adjustment";

interface ForecastScenarioFormProps {
  name: string;
  description: string;
  adjustment: string;
  mode: ScenarioMode;
  covariates: string[];
  seriesIds: string[];
  targetSeriesId: string;
  contextAdjustments: ForecastScenarioCovariateAdjustment[];
  editing: boolean;
  saving: boolean;
  error: string | null;
  onNameChange: (value: string) => void;
  onDescriptionChange: (value: string) => void;
  onAdjustmentChange: (value: string) => void;
  onModeChange: (value: ScenarioMode) => void;
  onTargetSeriesChange: (value: string) => void;
  onContextAdjustmentsChange: (value: ForecastScenarioCovariateAdjustment[]) => void;
  onSubmit: (event: FormEvent<HTMLFormElement>) => void;
  onCancel: () => void;
}

export function ForecastScenarioForm(props: ForecastScenarioFormProps) {
  const { t } = useTranslation();

  return (
    <form className="fcs-form" onSubmit={(event) => void props.onSubmit(event)}>
      <input
        className="fcs-input"
        value={props.name}
        maxLength={80}
        onChange={(event) => props.onNameChange(event.target.value)}
        placeholder={t("forecast.scenarios.name")}
      />
      <input
        className="fcs-input"
        value={props.description}
        maxLength={500}
        onChange={(event) => props.onDescriptionChange(event.target.value)}
        placeholder={t("forecast.scenarios.description")}
      />
      <div className="fcs-mode">
        <button
          className={`fcs-mode-btn ${props.mode === "percent_adjustment" ? "is-active" : ""}`}
          type="button"
          onClick={() => props.onModeChange("percent_adjustment")}
        >
          {t("forecast.scenarios.percentType")}
        </button>
        <button
          className={`fcs-mode-btn ${props.mode === "context_adjustment" ? "is-active" : ""}`}
          type="button"
          onClick={() => props.onModeChange("context_adjustment")}
          disabled={props.covariates.length === 0}
        >
          {t("forecast.scenarios.contextType")}
        </button>
      </div>
      {props.mode === "context_adjustment" && (
        <ForecastScenarioContextFields
          covariates={props.covariates}
          seriesIds={props.seriesIds}
          targetSeriesId={props.targetSeriesId}
          adjustments={props.contextAdjustments}
          onTargetSeriesChange={props.onTargetSeriesChange}
          onAdjustmentsChange={props.onContextAdjustmentsChange}
        />
      )}
      <p className="fcs-hint">
        {props.mode === "context_adjustment"
          ? t("forecast.scenarios.contextHint")
          : t("forecast.scenarios.percentHint")}
      </p>
      <div className="fcs-form-row">
        {props.mode === "percent_adjustment" && (
          <input
            className="fcs-input fcs-input-number"
            type="number"
            min="-95"
            max="500"
            step="0.1"
            value={props.adjustment}
            onChange={(event) => props.onAdjustmentChange(event.target.value)}
            aria-label={t("forecast.scenarios.adjustment")}
          />
        )}
        <button
          className={`fcs-add-btn ${props.editing ? "is-primary" : ""}`}
          type="submit"
          disabled={props.saving || !canSubmit(props)}
        >
          {props.saving
            ? t("forecast.scenarios.saving")
            : props.editing
              ? props.mode === "context_adjustment"
                ? t("forecast.scenarios.relaunch")
                : t("forecast.scenarios.update")
              : t("forecast.scenarios.launch")}
        </button>
        {props.editing && (
          <button className="fcs-add-btn" type="button" onClick={props.onCancel}>
            {t("forecast.scenarios.cancel")}
          </button>
        )}
      </div>
      {props.error && <p className="fcs-error">{props.error}</p>}
    </form>
  );
}

function canSubmit(props: ForecastScenarioFormProps): boolean {
  if (!props.name.trim()) return false;
  if (props.mode === "percent_adjustment") return true;
  return props.contextAdjustments.length > 0 && props.covariates.length > 0;
}
