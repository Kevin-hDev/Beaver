import { useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { ChevronDown } from "@/components/ui/icons";
import type { ForecastLayerState } from "../forecast-layer-matrix";
import { formatForecastValue, inferMetricMeta } from "../forecast-view-format";
import { ForecastChart } from "../charts/forecast-chart";
import { useForecastChartLabels } from "../charts/use-forecast-chart-labels";
import { useForecastChartResize } from "../use-forecast-chart-resize";
import { KpiRow, PeriodCell, ValueCell } from "../forecast-view-widgets";
import { buildForecastVariableLines } from "../forecast-variable-lines";
import { useForecastResult } from "../use-forecast-result";
import { ForecastScenarioMenuSelect } from "./forecast-scenario-menu-select";
import {
  buildForecastLayerAnnotations,
  filterForecastSeriesData,
  type ForecastViewResult,
} from "./forecast-view-data";
import "../forecast-view.css";
import "../forecast-view-table.css";

interface ForecastViewProps {
  analysisId: string;
  layers: ForecastLayerState;
  /** Controlled series selection (workbench); falls back to internal state. */
  selectedSeries?: string;
  onSelectedSeriesChange?: (seriesId: string) => void;
  onZoomWindowChange?: (window: { start: number; end: number }) => void;
  zoomJump?: { start: number; seq: number } | null;
}

export function ForecastView({
  analysisId,
  layers,
  selectedSeries: controlledSeries,
  onSelectedSeriesChange,
  onZoomWindowChange,
  zoomJump,
}: ForecastViewProps) {
  const { t, i18n } = useTranslation();
  const { data, error } = useForecastResult<ForecastViewResult>(analysisId, t("forecast.noAnalysis"));
  const [internalSeries, setInternalSeries] = useState("");
  const selectedSeries = controlledSeries ?? internalSeries;
  const setSelectedSeries = onSelectedSeriesChange ?? setInternalSeries;
  const [tableOpen, setTableOpen] = useState(false);
  const chart = useForecastChartResize();

  // Memoized chart inputs: zoom-state re-renders must not rebuild the
  // whole ECharts option (wheel gestures otherwise storm setOption).
  const activeSeries =
    data && selectedSeries && data.input_data.series_ids?.includes(selectedSeries)
      ? selectedSeries
      : data?.input_data.series_ids?.[0] ?? "";
  const scenarioLines = useMemo(() => {
    if (!data) return [];
    return [
      ...(data.scenarios ?? []),
      ...(data.ensemble ? [{
        id: "ensemble",
        name: t("forecast.view.ensembleSeries"),
        predictions: data.ensemble.predictions,
      }] : []),
    ];
  }, [data, t]);
  const filtered = useMemo(
    () => (data ? filterForecastSeriesData(data, activeSeries, scenarioLines) : null),
    [data, activeSeries, scenarioLines],
  );
  const annotations = useMemo(
    () =>
      data
        ? buildForecastLayerAnnotations(data, activeSeries, {
            anomaly: t("forecast.view.filters.residualAnomalies"),
            quality: t("forecast.view.filters.dataQualityIssues"),
          })
        : [],
    [data, activeSeries, t],
  );
  const variables = useMemo(
    () =>
      data && filtered
        ? buildForecastVariableLines({
            rows: data.input_data.rows ?? [],
            covariates: data.covariates_used ?? [],
            targetColumn: data.target_column ?? "",
            seriesColumn: data.series_column,
            historyValues: filtered.history.map((point) => point.value),
            forecastValues: filtered.predictions.map((point) => point.value),
            selectedSeries: activeSeries,
          })
        : [],
    [data, filtered, activeSeries],
  );
  const labels = useForecastChartLabels();
  const quantiles = useMemo(
    () => ({ q10: filtered?.q10 ?? [], q90: filtered?.q90 ?? [] }),
    [filtered],
  );

  if (error) return <div className="fc-error">{error}</div>;
  if (!data || !filtered) return <div className="fc-loading"><div className="fc-skeleton" /></div>;

  const metric = inferMetricMeta(i18n.language, data.target_column, data.name);

  return (
    <div className="fc-view">
      {data.metrics && <KpiRow metrics={data.metrics} />}
      {data.input_data.series_ids && data.input_data.series_ids.length > 1 && (
        <div className="fc-view-toolbar">
          <span className="fc-view-toolbar-label">
            {t("forecast.view.series")}
          </span>
          <ForecastScenarioMenuSelect
            className="fc-view-toolbar-menu"
            value={activeSeries}
            options={data.input_data.series_ids.map((seriesId) => ({
              value: seriesId,
              label: seriesId,
            }))}
            onChange={setSelectedSeries}
          />
        </div>
      )}
      <div
        className={`fc-chart-area ${chart.isResizing ? "is-resizing" : ""}`}
        style={{ height: chart.chartHeight, minHeight: chart.chartHeight, maxHeight: chart.chartHeight }}
      >
        <div className="fc-chart-placeholder">
          <ForecastChart
            history={filtered.history}
            predictions={filtered.predictions}
            scenarios={filtered.scenarios}
            variables={variables}
            annotations={annotations}
            quantiles={quantiles}
            frequency={data.frequency}
            endDate={data.input_summary.end}
            locale={i18n.language}
            targetColumn={data.target_column}
            fallbackName={data.name}
            labels={labels}
            layers={layers}
            mode="main"
            onZoomWindowChange={onZoomWindowChange}
            zoomJump={zoomJump}
          />
        </div>
      </div>
      <div
        className="fc-chart-resize"
        onPointerDown={chart.startResize}
        onDoubleClick={chart.resetHeight}
      />
      <button
        type="button"
        className="fc-table-toggle"
        aria-expanded={tableOpen}
        onClick={() => setTableOpen((current) => !current)}
      >
        <span className="fc-table-toggle-title">
          {t("forecast.view.detailedPredictions")}
        </span>
        <span className="fc-table-toggle-count">{filtered.predictions.length}</span>
        <ChevronDown
          size="var(--icon-sm)"
          className={`fc-table-toggle-chevron ${tableOpen ? "is-open" : ""}`}
        />
      </button>
      <div className={`fc-table-collapse ${tableOpen ? "is-open" : ""}`}>
        <div className="fc-table-collapse-inner">
          <div className="fc-predictions-table">
            <div className="fc-table-head">
              <span>{t("forecast.view.period")}</span>
              <span>{metric.columnTitle}</span>
            </div>
            <div className="fc-table-body">
              {/* Rows mount only when the section opens: long horizons
                  would otherwise render hundreds of hidden cells. */}
              {tableOpen && filtered.predictions.map((p, i) => (
                <div key={i} className="fc-table-row">
                  <PeriodCell
                    index={i}
                    rawDate={p.date}
                    endDate={data.input_summary.end}
                    frequency={data.frequency}
                    locale={i18n.language}
                  />
                  <ValueCell
                    unitLabel={metric.unitLabel}
                    formattedValue={formatForecastValue(p.value, i18n.language, metric)}
                  />
                </div>
              ))}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
