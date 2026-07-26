import type { TFunction } from "i18next";
import { ForecastScenarioMenuSelect } from "./forecast-scenario-menu-select";
import type { ForecastComparisonOption } from "./forecast-comparison-types";
import "./forecast-scenario-menu.css";

interface ForecastComparisonPickerProps {
  options: ForecastComparisonOption[];
  selectedId: string;
  seriesIds: string[];
  selectedSeries: string;
  t: TFunction;
  onSelect: (id: string) => void;
  onSeriesChange: (seriesId: string) => void;
}

export function ForecastComparisonPicker({
  options,
  selectedId,
  seriesIds,
  selectedSeries,
  t,
  onSelect,
  onSeriesChange,
}: ForecastComparisonPickerProps) {
  return (
    <div className="fccmp-picker">
      {seriesIds.length > 1 && (
        <div className="fccmp-field">
          <span>{t("forecast.comparisons.series")}</span>
          <ForecastScenarioMenuSelect
            value={selectedSeries}
            options={seriesIds.map((seriesId) => ({ value: seriesId, label: seriesId }))}
            onChange={onSeriesChange}
            placeholder={t("forecast.comparisons.series")}
          />
        </div>
      )}
      <div className="fccmp-option-list">
        {options.map((option) => (
          <button
            key={option.id}
            className={`fccmp-option ${selectedId === option.id ? "is-active" : ""}`}
            onClick={() => onSelect(option.id)}
          >
            <span className="fccmp-option-title">{option.label}</span>
            <span className="fccmp-option-meta">{option.meta}</span>
          </button>
        ))}
      </div>
    </div>
  );
}
