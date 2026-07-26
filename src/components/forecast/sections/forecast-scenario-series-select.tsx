import { useTranslation } from "react-i18next";
import { ForecastScenarioMenuSelect } from "./forecast-scenario-menu-select";

interface ForecastScenarioSeriesSelectProps {
  seriesIds: string[];
  selectedSeries: string;
  onSelectedSeriesChange: (value: string) => void;
}

export function ForecastScenarioSeriesSelect({
  seriesIds,
  selectedSeries,
  onSelectedSeriesChange,
}: ForecastScenarioSeriesSelectProps) {
  const { t } = useTranslation();
  if (seriesIds.length <= 1) return null;

  return (
    <div className="fcs-series-bar">
      <span className="fcs-series-label">
        {t("forecast.view.series")}
      </span>
      <ForecastScenarioMenuSelect
        className="fcs-series-menu"
        value={selectedSeries || seriesIds[0]}
        options={seriesIds.map((seriesId) => ({
          value: seriesId,
          label: seriesId,
        }))}
        onChange={onSelectedSeriesChange}
      />
    </div>
  );
}
