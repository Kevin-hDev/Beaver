import type { ForecastChartAnnotation } from "../charts/forecast-chart-types";

interface ForecastViewPoint {
  date: string;
  value: number;
  series_id?: string | null;
}

interface ForecastViewScenario {
  id: string;
  name: string;
  predictions: ForecastViewPoint[];
}

export interface ForecastViewResult {
  id: string;
  name: string;
  target_column?: string;
  series_column?: string | null;
  model: string;
  horizon: number;
  frequency: string;
  input_summary: { end: string };
  input_data: {
    rows?: Record<string, unknown>[];
    series_ids?: string[];
    history: ForecastViewPoint[];
  };
  covariates_used?: string[];
  predictions: ForecastViewPoint[];
  quantiles: { q10: number[]; q50: number[]; q90: number[] };
  scenarios: ForecastViewScenario[];
  ensemble?: { predictions: ForecastViewPoint[] } | null;
  metrics: {
    mape: number | null;
    mae: number | null;
    crps: number | null;
    bias: number | null;
  } | null;
  annotations?: Array<ForecastChartAnnotation>;
  advanced_analytics?: {
    anomalies?: Array<{
      id: string;
      date: string;
      score: number;
      series_id?: string | null;
    }>;
  } | null;
  data_profile?: {
    issues?: Array<{ code: string; count: number }>;
  } | null;
}

export function filterForecastSeriesData(
  data: ForecastViewResult,
  selectedSeries: string,
  scenarios: ForecastViewScenario[],
) {
  if (!data.input_data.series_ids || data.input_data.series_ids.length <= 1) {
    return {
      history: data.input_data.history,
      predictions: data.predictions,
      scenarios,
      q10: data.quantiles.q10,
      q90: data.quantiles.q90,
    };
  }

  const seriesId = selectedSeries || data.input_data.series_ids[0];
  const indices: number[] = [];
  const predictions = data.predictions.filter((point, index) => {
    const match = point.series_id === seriesId;
    if (match) indices.push(index);
    return match;
  });

  return {
    history: data.input_data.history.filter((point) => point.series_id === seriesId),
    predictions,
    scenarios: scenarios.map((scenario) => ({
      ...scenario,
      predictions: scenario.predictions.filter((point) => point.series_id === seriesId),
    })),
    q10: indices.map((index) => data.quantiles.q10[index]).filter(isNumber),
    q90: indices.map((index) => data.quantiles.q90[index]).filter(isNumber),
  };
}

export function buildForecastLayerAnnotations(
  data: ForecastViewResult,
  selectedSeries: string,
  labels: { anomaly: string; quality: string },
): ForecastChartAnnotation[] {
  const annotations = (data.annotations ?? []).map((item) => ({
    ...item,
    kind: "annotations" as const,
  }));
  const anomalies = (data.advanced_analytics?.anomalies ?? [])
    .filter((item) => !selectedSeries || item.series_id === selectedSeries)
    .map((item) => ({
      id: item.id,
      date: item.date,
      text: `${labels.anomaly} · ${formatScore(item.score)}`,
      source: "llm" as const,
      kind: "anomalies" as const,
    }));
  const quality = (data.data_profile?.issues ?? [])
    .filter((issue) => issue.count > 0)
    .map((issue, index) => ({
      id: `quality-${index}-${issue.code}`,
      date: data.input_summary.end,
      text: `${labels.quality} · ${issue.count}`,
      source: "user" as const,
      kind: "quality" as const,
    }));
  return [...annotations, ...anomalies, ...quality];
}

function isNumber(value: number | undefined): value is number {
  return value !== undefined;
}

function formatScore(value: number) {
  return Number.isFinite(value) ? value.toFixed(2) : "—";
}
