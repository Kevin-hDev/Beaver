import { useCallback, useState } from "react";
import { useTranslation } from "react-i18next";
import "../forecast-sections.css";
import "./forecast-scenarios.css";
import "./forecast-scenario-form.css";
import "./forecast-scenario-menu.css";
import "./forecast-scenario-list.css";
import type { ForecastScenarioAnalysis } from "./forecast-scenario-types";
import { ForecastScenarioPreview } from "./forecast-scenario-preview";
import { ForecastScenarioForm } from "./forecast-scenario-form";
import { ForecastScenarioPicker } from "./forecast-scenario-picker";
import { useForecastChartResize } from "../use-forecast-chart-resize";
import { ForecastScenarioList } from "./forecast-scenario-list";
import { ForecastScenarioSeriesSelect } from "./forecast-scenario-series-select";
import { useForecastScenarioAnalysis } from "./use-forecast-scenario-analysis";
import { useForecastScenarioForm } from "./use-forecast-scenario-form";

interface ForecastScenariosProps {
  analysisId: string;
  pickerOpen: boolean;
  onFocusAnalysis: (id: string) => void;
  onAnalysisChanged: () => void;
}

export function ForecastScenarios(props: ForecastScenariosProps) {
  const { analysisId, pickerOpen, onFocusAnalysis, onAnalysisChanged } = props;
  const { t } = useTranslation();
  const [data, setData] = useState<ForecastScenarioAnalysis | null>(null);
  const [activeScenarioId, setActiveScenarioId] = useState<string | null>(null);
  const [selectedSeries, setSelectedSeries] = useState("");
  const [error, setError] = useState<string | null>(null);
  const chart = useForecastChartResize();
  const form = useForecastScenarioForm({
    analysisId,
    onAnalysisChanged,
    setData,
    setActiveScenarioId,
    setError,
    t,
  });

  const handleLoaded = useCallback((analysis: ForecastScenarioAnalysis) => {
    setData(analysis);
    setActiveScenarioId((current) =>
      current && analysis.scenarios.some((scenario) => scenario.id === current)
        ? current
        : analysis.scenarios[0]?.id ?? null,
    );
  }, []);

  const handleLoadFailed = useCallback(() => {
    setError(t("forecast.scenarios.loadFailed"));
  }, [t]);

  useForecastScenarioAnalysis({
    analysisId,
    onLoaded: handleLoaded,
    onFailed: handleLoadFailed,
  });

  const covariates = data?.covariates_used ?? data?.input_data.covariate_columns ?? [];
  const seriesIds = data?.input_data.series_ids ?? [];

  return (
    <div className="fcs-root">
      <div className={`fcs-shell ${pickerOpen ? "is-picker-open" : ""}`}>
        <div className="fcs-content fcs-main">
        {data && (
          <div className="fcs-preview-shell">
            <ForecastScenarioSeriesSelect
              seriesIds={seriesIds}
              selectedSeries={selectedSeries}
              onSelectedSeriesChange={setSelectedSeries}
            />
            <ForecastScenarioPreview
              analysis={data}
              scenarioName={form.name}
              mode={form.mode}
              adjustmentPercent={Number(form.adjustment) || 0}
              activeScenarioId={activeScenarioId}
              editingScenarioId={form.editingScenarioId}
              showDraftPreview={form.draftPreviewVisible}
              selectedSeries={selectedSeries}
              targetSeriesId={form.targetSeriesId}
              contextAdjustments={form.contextAdjustments}
              chartHeight={chart.chartHeight}
              isResizing={chart.isResizing}
            />
            <div
              className="fc-chart-resize"
              onPointerDown={chart.startResize}
              onDoubleClick={chart.resetHeight}
            />
          </div>
        )}
        <div className="fcs-scroll-body">
          <ForecastScenarioForm
            name={form.name}
            description={form.description}
            adjustment={form.adjustment}
            mode={form.mode}
            covariates={covariates}
            seriesIds={seriesIds}
            targetSeriesId={form.targetSeriesId}
            contextAdjustments={form.contextAdjustments}
            editing={Boolean(form.editingScenarioId)}
            saving={form.saving}
            error={error}
            onNameChange={form.setName}
            onDescriptionChange={form.setDescription}
            onAdjustmentChange={form.setAdjustment}
            onModeChange={form.setMode}
            onTargetSeriesChange={form.setTargetSeriesId}
            onContextAdjustmentsChange={form.setContextAdjustments}
            onSubmit={(event) => void form.submit(event)}
            onCancel={form.reset}
          />
          <ForecastScenarioList
            scenarios={data?.scenarios ?? []}
            activeScenarioId={activeScenarioId}
            onSelect={(scenario) => setActiveScenarioId(scenario.id)}
            onEdit={form.editScenario}
            onDelete={(scenarioId) => void form.removeScenario(scenarioId)}
          />
        </div>
      </div>
        <ForecastScenarioPicker
          open={pickerOpen}
          currentAnalysisId={analysisId}
          onSelectAnalysis={onFocusAnalysis}
        />
      </div>
    </div>
  );
}
