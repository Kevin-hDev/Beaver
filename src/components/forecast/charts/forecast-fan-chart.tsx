import { useMemo } from "react";
import { useTranslation } from "react-i18next";
import {
  filterForecastSeriesData,
  type ForecastViewResult,
} from "../sections/forecast-view-data";
import { useForecastResult } from "../use-forecast-result";
import { ForecastChart } from "./forecast-chart";
import { useForecastChartLabels } from "./use-forecast-chart-labels";
import type { ForecastChartProps } from "./forecast-chart-types";

const FAN_HISTORY_WINDOW = 18;
const FAN_LAYERS = { history: true, forecast: true, confidence: true };
// Stable empty lists: fresh literals would defeat the chart option memo.
const NO_SCENARIOS: ForecastChartProps["scenarios"] = [];
const NO_VARIABLES: ForecastChartProps["variables"] = [];
const NO_ANNOTATIONS: NonNullable<ForecastChartProps["annotations"]> = [];

interface ForecastFanChartProps {
  analysisId: string;
  /** Active series; falls back to the first series when absent/unknown. */
  seriesId?: string;
  /** Bumped by the card after an expand transition completes. */
  resizeSignal?: number;
  onZoomWindowChange?: (window: { start: number; end: number }) => void;
  zoomJump?: { start: number; seq: number } | null;
}

export function ForecastFanChart({
  analysisId,
  seriesId,
  resizeSignal = 0,
  onZoomWindowChange,
  zoomJump,
}: ForecastFanChartProps) {
  const { t, i18n } = useTranslation();
  const { data, error } = useForecastResult<ForecastViewResult>(
    analysisId,
    t("forecast.noAnalysis"),
  );
  const labels = useForecastChartLabels();

  // Memoized chart inputs: fan zoom-state re-renders (workbench) must not
  // rebuild the whole ECharts option on every wheel frame.
  const ids = data?.input_data.series_ids ?? [];
  const activeSeries = seriesId && ids.includes(seriesId) ? seriesId : ids[0] ?? "";
  const filtered = useMemo(
    () => (data ? filterForecastSeriesData(data, activeSeries, []) : null),
    [data, activeSeries],
  );
  const history = useMemo(
    () => filtered?.history.slice(-FAN_HISTORY_WINDOW) ?? [],
    [filtered],
  );
  const quantiles = useMemo(
    () => ({ q10: filtered?.q10 ?? [], q90: filtered?.q90 ?? [] }),
    [filtered],
  );

  if (error) return <div className="fcwf-companion-empty">{error}</div>;
  if (!data || !filtered) {
    return <div className="fc-loading"><div className="fc-skeleton" /></div>;
  }

  if (!filtered.predictions.length) {
    return (
      <div className="fcwf-companion-empty">
        {t("forecast.companion.insufficientData")}
      </div>
    );
  }

  return (
    <div className="fcwf-companion fcwf-companion-fan">
      {/* key remount: the canvas can lose its bitmap while the card body is
          clipped/composited during collapse, so expanding re-initializes the
          chart at full size. Full (non-compact) mode for the zoom controls. */}
      <ForecastChart
        key={resizeSignal}
        history={history}
        predictions={filtered.predictions}
        scenarios={NO_SCENARIOS}
        variables={NO_VARIABLES}
        annotations={NO_ANNOTATIONS}
        quantiles={quantiles}
        frequency={data.frequency}
        endDate={data.input_summary.end}
        locale={i18n.language}
        targetColumn={data.target_column}
        fallbackName={data.name}
        labels={labels}
        layers={FAN_LAYERS}
        mode="main"
        onZoomWindowChange={onZoomWindowChange}
        zoomJump={zoomJump}
      />
    </div>
  );
}
