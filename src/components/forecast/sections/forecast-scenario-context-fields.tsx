import { Plus, X } from "@/components/ui/icons";
import { useTranslation } from "react-i18next";
import { Tooltip } from "@/components/ui/tooltip";
import { ForecastScenarioMenuSelect } from "./forecast-scenario-menu-select";
import type { ForecastScenarioCovariateAdjustment } from "./forecast-scenario-types";

interface ForecastScenarioContextFieldsProps {
  covariates: string[];
  seriesIds: string[];
  targetSeriesId: string;
  adjustments: ForecastScenarioCovariateAdjustment[];
  onTargetSeriesChange: (value: string) => void;
  onAdjustmentsChange: (value: ForecastScenarioCovariateAdjustment[]) => void;
}

export function ForecastScenarioContextFields({
  covariates,
  seriesIds,
  targetSeriesId,
  adjustments,
  onTargetSeriesChange,
  onAdjustmentsChange,
}: ForecastScenarioContextFieldsProps) {
  const { t } = useTranslation();
  const canAdd = covariates.length > 0;

  return (
    <div className="fcs-context">
      {seriesIds.length > 1 && (
        <ForecastScenarioMenuSelect
          className="fcs-context-series"
          value={targetSeriesId}
          onChange={onTargetSeriesChange}
          options={[
            { value: "", label: t("forecast.scenarios.allSeries") },
            ...seriesIds.map((seriesId) => ({ value: seriesId, label: seriesId })),
          ]}
          placeholder={t("forecast.scenarios.seriesScope")}
        />
      )}

      {adjustments.map((adjustment, index) => (
        <div className="fcs-context-row" key={`${adjustment.column}-${index}`}>
          <ForecastScenarioMenuSelect
            value={adjustment.column}
            onChange={(value) => updateAdjustment(index, { column: value })}
            options={covariates.map((column) => ({ value: column, label: column }))}
          />
          <ForecastScenarioMenuSelect
            className="fcs-context-mode"
            value={adjustment.mode}
            onChange={(value) =>
              updateAdjustment(index, {
                mode: value === "absolute" ? "absolute" : "percent",
              })
            }
            options={[
              { value: "percent", label: t("forecast.scenarios.percentMode") },
              { value: "absolute", label: t("forecast.scenarios.absoluteMode") },
            ]}
          />
          <input
            className="fcs-input fcs-context-value"
            type="number"
            min="-95"
            max="500"
            step="0.1"
            value={adjustment.value}
            onChange={(event) => updateAdjustment(index, { value: Number(event.target.value) })}
            aria-label={t("forecast.scenarios.contextValue")}
          />
          <Tooltip label={t("forecast.scenarios.removeContext")}>
            <button
              className="icon-btn fcs-icon-btn"
              type="button"
              onClick={() => removeAdjustment(index)}
            >
              <X size="var(--icon-13)" />
            </button>
          </Tooltip>
        </div>
      ))}

      <button
        className="fcs-context-add"
        type="button"
        disabled={!canAdd}
        onClick={addAdjustment}
      >
        <Plus size="var(--icon-13)" />
        {t("forecast.scenarios.addContext")}
      </button>
    </div>
  );

  function addAdjustment() {
    const column = covariates[0];
    if (!column) return;
    onAdjustmentsChange([...adjustments, { column, mode: "percent", value: 10 }]);
  }

  function removeAdjustment(index: number) {
    onAdjustmentsChange(adjustments.filter((_, itemIndex) => itemIndex !== index));
  }

  function updateAdjustment(
    index: number,
    patch: Partial<ForecastScenarioCovariateAdjustment>,
  ) {
    onAdjustmentsChange(
      adjustments.map((item, itemIndex) =>
        itemIndex === index ? { ...item, ...patch } : item,
      ),
    );
  }
}
