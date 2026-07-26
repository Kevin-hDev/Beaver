import { useCallback, useEffect, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { useLatestRequest } from "@/hooks/use-latest-request";
import {
  FORECAST_ANALYSIS_CREATED,
  FORECAST_ANALYSIS_DELETED,
  FORECAST_ANALYSIS_UPDATED,
  listenForecastAnalysisEvents,
} from "@/lib/forecast-analysis-events";
import { ForecastChart } from "../charts/forecast-chart";
import { formatForecastValue, inferMetricMeta } from "../forecast-view-format";
import { useForecastChartResize } from "../use-forecast-chart-resize";
import { ForecastComparisonPicker } from "./forecast-comparison-picker";
import { ForecastComparisonSummary } from "./forecast-comparison-summary";
import { ForecastComparisonTable } from "./forecast-comparison-table";
import {
  buildComparisonOptions,
  buildComparisonRows,
  buildComparisonStats,
  filterComparisonPoints,
  filterComparisonQuantiles,
} from "./forecast-comparison-utils";
import type {
  ForecastComparisonAnalysis,
  ForecastComparisonMeta,
} from "./forecast-comparison-types";
import "../forecast-view.css";
import "./forecast-comparisons.css";
import { MAX_FORECAST_COMPARISON_ANALYSES } from "../forecast-limits";


interface ForecastComparisonsProps {
  analysisId: string;
}

export function ForecastComparisons({ analysisId }: ForecastComparisonsProps) {
  const { t, i18n } = useTranslation();
  const [current, setCurrent] = useState<ForecastComparisonAnalysis | null>(null);
  const [analyses, setAnalyses] = useState<ForecastComparisonAnalysis[]>([]);
  const [selectedId, setSelectedId] = useState("");
  const [selectedSeries, setSelectedSeries] = useState("");
  const [error, setError] = useState<string | null>(null);
  const runLatest = useLatestRequest();
  const chart = useForecastChartResize();

  const load = useCallback(async () => {
    try {
      const data = await runLatest(() => loadComparisonData(analysisId));
      if (data === undefined) return;
      setCurrent(data.current);
      setAnalyses(data.analyses);
      setSelectedSeries(data.current.input_data.series_ids?.[0] ?? "");
      setError(null);
    } catch {
      setError(t("forecast.comparisons.loadFailed"));
    }
  }, [analysisId, runLatest, t]);

  useEffect(() => {
    const timer = window.setTimeout(() => void load(), 0);
    const cleanup = listenForecastAnalysisEvents(
      [
        FORECAST_ANALYSIS_CREATED,
        FORECAST_ANALYSIS_UPDATED,
        FORECAST_ANALYSIS_DELETED,
      ],
      () => void load(),
    );
    return () => {
      window.clearTimeout(timer);
      cleanup();
    };
  }, [load]);

  const seriesIds = current?.input_data.series_ids ?? [];
  const options = useMemo(
    () => current ? buildComparisonOptions(current, analyses, selectedSeries, t) : [],
    [analyses, current, selectedSeries, t],
  );
  const selectedOption = options.find((option) => option.id === selectedId) ?? options[0] ?? null;
  const activeSelectedId = selectedOption?.id ?? "";

  if (error) return <div className="fc-error">{error}</div>;
  if (!current) return <div className="fc-loading"><div className="fc-skeleton" /></div>;

  const basePredictions = filterComparisonPoints(current.predictions, selectedSeries);
  const quantiles = filterComparisonQuantiles(current.predictions, selectedSeries, current.quantiles);
  const rows = selectedOption ? buildComparisonRows(basePredictions, selectedOption.predictions) : [];
  const stats = buildComparisonStats(rows);
  const metric = inferMetricMeta(i18n.language, current.target_column, current.name);
  const formatValue = (value: number) => formatForecastValue(value, i18n.language, metric);

  return (
    <div className="fccmp-root">
      <div
        className={`fc-chart-area ${chart.isResizing ? "is-resizing" : ""}`}
        style={{ height: chart.chartHeight, minHeight: chart.chartHeight, maxHeight: chart.chartHeight }}
      >
        <div className="fc-chart-placeholder">
          <ForecastChart
            history={filterComparisonPoints(current.input_data.history, selectedSeries)}
            predictions={basePredictions}
            scenarios={selectedOption ? [{
              id: "compare",
              name: selectedOption.label,
              predictions: selectedOption.predictions,
            }] : []}
            variables={[]}
            quantiles={quantiles}
            frequency={current.frequency}
            endDate={current.input_summary.end}
            locale={i18n.language}
            targetColumn={current.target_column}
            fallbackName={current.name}
            labels={{
              history: t("forecast.view.historySeries"),
              forecast: current.name,
              confidence: t("forecast.view.confidenceRange"),
              forecastStart: t("forecast.chart.forecastStart"),
              annotationUser: t("forecast.notes.userSource"),
              annotationLlm: t("forecast.notes.llmSource"),
            }}
            layers={{ history: true, forecast: true, confidence: true, "scenario-compare": true }}
            mode="comparison"
          />
        </div>
      </div>
      <div className="fc-chart-resize" onPointerDown={chart.startResize} onDoubleClick={chart.resetHeight} />
      <div className="fccmp-scroll">
        <ForecastComparisonPicker
          options={options}
          selectedId={activeSelectedId}
          seriesIds={seriesIds}
          selectedSeries={selectedSeries}
          t={t}
          onSelect={setSelectedId}
          onSeriesChange={setSelectedSeries}
        />
        {selectedOption ? (
          <>
            <ForecastComparisonSummary stats={stats} t={t} formatValue={formatValue} />
            <ForecastComparisonTable
              rows={rows}
              endDate={current.input_summary.end}
              frequency={current.frequency}
              locale={i18n.language}
              t={t}
              formatValue={formatValue}
            />
          </>
        ) : (
          <div className="fcs-empty">
            <p className="fcs-empty-text">{t("forecast.comparisons.empty")}</p>
            <p className="fcs-empty-sub">{t("forecast.comparisons.emptySub")}</p>
          </div>
        )}
      </div>
    </div>
  );
}

async function loadComparisonData(analysisId: string) {
  const current = await invoke<ForecastComparisonAnalysis>("get_forecast_analysis", { id: analysisId });
  const metas = await invoke<ForecastComparisonMeta[]>("list_forecast_analyses");
  const safeMetas = metas.slice(-MAX_FORECAST_COMPARISON_ANALYSES);
  const loaded = await Promise.allSettled(
    safeMetas.map((meta) => invoke<ForecastComparisonAnalysis>("get_forecast_analysis", { id: meta.id })),
  );
  return {
    current,
    analyses: loaded
      .filter((result): result is PromiseFulfilledResult<ForecastComparisonAnalysis> => result.status === "fulfilled")
      .map((result) => result.value),
  };
}
