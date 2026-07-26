import { useTranslation } from "react-i18next";
import { ForecastChart } from "../charts/forecast-chart";
import { buildScenarioVariableLines } from "./forecast-scenario-variable-lines";
import {
  buildDraftScenario,
  buildPreviewLayers,
  buildSavedScenarios,
  filterScenarioSeriesData,
} from "./forecast-scenario-preview-data";
import type {
  ForecastScenarioAnalysis,
  ForecastScenarioCovariateAdjustment,
} from "./forecast-scenario-types";
import "../forecast-view.css";
import "./forecast-scenario-preview.css";

interface ForecastScenarioPreviewProps {
  analysis: ForecastScenarioAnalysis;
  scenarioName: string;
  mode: "percent_adjustment" | "context_adjustment";
  adjustmentPercent: number;
  activeScenarioId: string | null;
  editingScenarioId: string | null;
  showDraftPreview: boolean;
  selectedSeries: string;
  targetSeriesId: string;
  contextAdjustments: ForecastScenarioCovariateAdjustment[];
  chartHeight: number;
  isResizing: boolean;
}

export function ForecastScenarioPreview({
  analysis,
  scenarioName,
  mode,
  adjustmentPercent,
  activeScenarioId,
  editingScenarioId,
  showDraftPreview,
  selectedSeries,
  targetSeriesId,
  contextAdjustments,
  chartHeight,
  isResizing,
}: ForecastScenarioPreviewProps) {
  const { t, i18n } = useTranslation();
  const factor = 1 + adjustmentPercent / 100;
  const seriesId = selectedSeries || analysis.input_data.series_ids?.[0] || "";
  const filtered = filterScenarioSeriesData(analysis, seriesId);
  const previewName = scenarioName.trim() || t("forecast.scenarios.preview");
  const savedScenarios =
    editingScenarioId && showDraftPreview
      ? []
      : buildSavedScenarios(analysis, activeScenarioId, editingScenarioId, seriesId);
  const previewScenario = showDraftPreview
    ? buildDraftScenario(
        analysis,
        filtered.predictions,
        previewName,
        factor,
        mode,
        editingScenarioId,
        seriesId,
      )
    : null;
  const chartScenarios = previewScenario ? [...savedScenarios, previewScenario] : savedScenarios;
  const variableLines = buildScenarioVariableLines({
    analysis,
    historyValues: filtered.history.map((point) => point.value),
    forecastValues: filtered.predictions.map((point) => point.value),
    selectedSeries: seriesId,
    activeScenarioId,
    draftAdjustments: contextAdjustments,
    draftTargetSeriesId: targetSeriesId || null,
    showDraftPreview,
  });
  const previewLayers = buildPreviewLayers(chartScenarios, variableLines);

  return (
    <div className="fcs-preview">
      <div
        className={`fc-chart-area ${isResizing ? "is-resizing" : ""}`}
        style={{ height: chartHeight, minHeight: chartHeight, maxHeight: chartHeight }}
      >
        <div className="fc-chart-placeholder">
          <ForecastChart
            history={filtered.history}
            predictions={filtered.predictions}
            scenarios={chartScenarios}
            variables={variableLines}
            quantiles={filtered.quantiles}
            frequency={analysis.frequency}
            endDate={analysis.input_summary.end}
            locale={i18n.language}
            targetColumn={analysis.target_column}
            fallbackName={previewName}
            labels={{
              history: t("forecast.view.historySeries"),
              forecast: t("forecast.view.forecastSeries"),
              confidence: t("forecast.view.confidenceRange"),
              forecastStart: t("forecast.chart.forecastStart"),
              annotationUser: t("forecast.notes.userSource"),
              annotationLlm: t("forecast.notes.llmSource"),
            }}
            layers={previewLayers}
            mode="scenario"
          />
        </div>
      </div>
    </div>
  );
}
