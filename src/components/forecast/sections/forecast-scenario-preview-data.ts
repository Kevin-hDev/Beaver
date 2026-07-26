import type { ScenarioLine } from "../charts/forecast-chart-types";
import type { ForecastLayerState } from "../forecast-layer-matrix";
import { buildScenarioVariableLines } from "./forecast-scenario-variable-lines";
import type { ForecastScenarioAnalysis } from "./forecast-scenario-types";

const PREVIEW_LAYERS: ForecastLayerState = {
  history: true,
  forecast: true,
  confidence: true,
  "scenario-preview": true,
};

export function buildPreviewLayers(
  scenarios: ScenarioLine[],
  variables: ReturnType<typeof buildScenarioVariableLines>,
): ForecastLayerState {
  const layers: ForecastLayerState = { ...PREVIEW_LAYERS };
  for (const scenario of scenarios) {
    layers[`scenario-${scenario.id}`] = true;
  }
  for (const variable of variables) {
    layers[variable.id] = true;
  }
  return layers;
}

export function buildSavedScenarios(
  analysis: ForecastScenarioAnalysis,
  activeScenarioId: string | null,
  editingScenarioId: string | null,
  selectedSeries: string,
): ScenarioLine[] {
  return analysis.scenarios
    .filter((scenario) => scenario.id === (editingScenarioId ?? activeScenarioId))
    .map((scenario) => ({
      id: scenario.id,
      name: scenario.name,
      predictions: selectedSeries
        ? scenario.predictions.filter((point) => point.series_id === selectedSeries)
        : scenario.predictions,
    }));
}

export function buildDraftScenario(
  analysis: ForecastScenarioAnalysis,
  predictions: ForecastScenarioAnalysis["predictions"],
  previewName: string,
  factor: number,
  mode: "percent_adjustment" | "context_adjustment",
  editingScenarioId: string | null,
  selectedSeries: string,
): ScenarioLine | null {
  if (mode === "context_adjustment") {
    const saved = analysis.scenarios.find((scenario) => scenario.id === editingScenarioId);
    if (!saved) return null;
    return {
      id: saved.id,
      name: saved.name,
      predictions: selectedSeries
        ? saved.predictions.filter((point) => point.series_id === selectedSeries)
        : saved.predictions,
    };
  }
  return {
    id: "preview",
    name: previewName,
    predictions: predictions.map((point) => ({
      ...point,
      value: point.value * factor,
    })),
  };
}

export function filterScenarioSeriesData(analysis: ForecastScenarioAnalysis, selectedSeries: string) {
  if (!analysis.input_data.series_ids || analysis.input_data.series_ids.length <= 1) {
    return {
      history: analysis.input_data.history,
      predictions: analysis.predictions,
      quantiles: analysis.quantiles,
    };
  }

  const indices: number[] = [];
  const predictions = analysis.predictions.filter((point, index) => {
    const match = point.series_id === selectedSeries;
    if (match) indices.push(index);
    return match;
  });

  return {
    history: analysis.input_data.history.filter((point) => point.series_id === selectedSeries),
    predictions,
    quantiles: {
      q10: indices.map((index) => analysis.quantiles.q10[index]).filter((value) => value !== undefined),
      q90: indices.map((index) => analysis.quantiles.q90[index]).filter((value) => value !== undefined),
    },
  };
}
