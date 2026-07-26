import { useCallback, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import {
  buildForecastLayerGroups,
  createInitialLayerState,
  type ForecastLayerState,
} from "../forecast-layer-matrix";
import { ForecastViewFilters } from "../forecast-view-filters";
import { ForecastView } from "../sections/forecast-view";
import {
  filterForecastSeriesData,
  type ForecastViewResult,
} from "../sections/forecast-view-data";
import { useForecastLayerSources } from "../use-forecast-layer-sources";
import { useForecastResult } from "../use-forecast-result";
import { ForecastChartCard } from "../charts/forecast-chart-card";
import { ForecastFanChart } from "../charts/forecast-fan-chart";
import { ForecastZoomJumpBars } from "../charts/forecast-zoom-jump-bars";
import { sameForecastZoomWindow } from "../charts/forecast-chart-zoom-utils";
import { ForecastSeasonalityChart } from "../charts/forecast-seasonality-chart";
import {
  SEASONALITY_MIN_HISTORY,
  supportsSeasonalityFrequency,
} from "../charts/forecast-seasonality-data";
import "./forecast-workbench-forecast.css";

export function ForecastWorkbenchForecast({ analysisId }: { analysisId: string }) {
  const { t } = useTranslation();
  const [layers, setLayers] = useState<ForecastLayerState>(createInitialLayerState);
  const [fanResize, setFanResize] = useState(0);
  const [seasonalityResize, setSeasonalityResize] = useState(0);
  const [mainZoomWindow, setMainZoomWindow] = useState({ start: 0, end: 100 });
  const [fanZoomWindow, setFanZoomWindow] = useState({ start: 0, end: 100 });
  const [mainJump, setMainJump] = useState<{ start: number; seq: number } | null>(null);
  const [fanJump, setFanJump] = useState<{ start: number; seq: number } | null>(null);
  // Bail on no-op updates so setOption -> datazoom events cannot loop.
  // useCallback keeps the chart props stable across zoom-state re-renders.
  const handleMainZoomWindow = useCallback((window: { start: number; end: number }) =>
    setMainZoomWindow((current) =>
      sameForecastZoomWindow(current, window) ? current : window,
    ), []);
  const handleFanZoomWindow = useCallback((window: { start: number; end: number }) =>
    setFanZoomWindow((current) =>
      sameForecastZoomWindow(current, window) ? current : window,
    ), []);
  const handleMainJump = useCallback((start: number) =>
    setMainJump((jump) => ({ start, seq: (jump?.seq ?? 0) + 1 })), []);
  const handleFanJump = useCallback((start: number) =>
    setFanJump((jump) => ({ start, seq: (jump?.seq ?? 0) + 1 })), []);
  // The fan chart remounts on expand (key bump), so its zoom resets to full.
  const handleFanExpanded = useCallback(() => {
    setFanResize((value) => value + 1);
    setFanZoomWindow({ start: 0, end: 100 });
  }, []);
  const { sources } = useForecastLayerSources(analysisId, setLayers);
  const groups = buildForecastLayerGroups(sources, t);
  const { data } = useForecastResult<ForecastViewResult>(
    analysisId,
    t("forecast.noAnalysis"),
  );
  // Active series shared by the main chart and both companion charts, so
  // every card always shows the SAME series.
  const [selectedSeries, setSelectedSeries] = useState("");
  const seriesIds = data?.input_data.series_ids ?? [];
  const activeSeries =
    selectedSeries && seriesIds.includes(selectedSeries)
      ? selectedSeries
      : seriesIds[0] ?? "";
  const cardTitle = (key: string) =>
    seriesIds.length > 1
      ? t("forecast.chartCard.withSeries", { title: t(key), series: activeSeries })
      : t(key);
  const filtered = useMemo(
    () => (data ? filterForecastSeriesData(data, activeSeries, []) : null),
    [data, activeSeries],
  );

  return (
    <div className="fcwf-root">
      <div className="fcwf-toolbar">
        <ForecastViewFilters groups={groups} layers={layers} onChange={setLayers} />
      </div>
      <div className="fcwf-stack">
        <ForecastChartCard
          title={t("forecast.chartCard.main")}
          headerCenter={
            <ForecastZoomJumpBars window={mainZoomWindow} onJump={handleMainJump} />
          }
        >
          <ForecastView
            analysisId={analysisId}
            layers={layers}
            selectedSeries={selectedSeries}
            onSelectedSeriesChange={setSelectedSeries}
            onZoomWindowChange={handleMainZoomWindow}
            zoomJump={mainJump}
          />
        </ForecastChartCard>
        {filtered && filtered.predictions.length > 0 ? (
          <ForecastChartCard
            title={cardTitle("forecast.chartCard.fan")}
            onExpanded={handleFanExpanded}
            headerCenter={
              <ForecastZoomJumpBars window={fanZoomWindow} onJump={handleFanJump} />
            }
          >
            <ForecastFanChart
              analysisId={analysisId}
              seriesId={activeSeries}
              resizeSignal={fanResize}
              onZoomWindowChange={handleFanZoomWindow}
              zoomJump={fanJump}
            />
          </ForecastChartCard>
        ) : null}
        {filtered &&
        filtered.history.length > SEASONALITY_MIN_HISTORY &&
        supportsSeasonalityFrequency(data?.frequency) ? (
          <ForecastChartCard
            title={cardTitle("forecast.chartCard.seasonality")}
            onExpanded={() => setSeasonalityResize((value) => value + 1)}
          >
            <ForecastSeasonalityChart
              analysisId={analysisId}
              seriesId={activeSeries}
              resizeSignal={seasonalityResize}
            />
          </ForecastChartCard>
        ) : null}
      </div>
    </div>
  );
}
