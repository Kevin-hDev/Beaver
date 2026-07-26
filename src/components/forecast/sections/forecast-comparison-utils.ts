import type {
  ForecastComparisonAnalysis,
  ForecastComparisonOption,
  ForecastComparisonRow,
  ForecastComparisonStats,
  ForecastPoint,
} from "./forecast-comparison-types";

export function filterComparisonPoints(points: ForecastPoint[], seriesId: string): ForecastPoint[] {
  if (!seriesId) return points;
  return points.filter((point) => point.series_id == null || point.series_id === seriesId);
}

export function filterComparisonQuantiles(
  points: ForecastPoint[],
  seriesId: string,
  quantiles: { q10: number[]; q90: number[] },
) {
  if (!seriesId) return quantiles;
  const indices = points
    .map((point, index) => (point.series_id == null || point.series_id === seriesId ? index : -1))
    .filter((index) => index >= 0);
  return {
    q10: indices.map((index) => quantiles.q10[index]).filter((value) => value !== undefined),
    q90: indices.map((index) => quantiles.q90[index]).filter((value) => value !== undefined),
  };
}

export function buildComparisonOptions(
  current: ForecastComparisonAnalysis,
  analyses: ForecastComparisonAnalysis[],
  seriesId: string,
  t: (key: string, values?: Record<string, unknown>) => string,
): ForecastComparisonOption[] {
  const options: ForecastComparisonOption[] = [];
  for (const scenario of current.scenarios ?? []) {
    options.push({
      id: `scenario:${scenario.id}`,
      kind: "scenario",
      label: scenario.name,
      meta: t("forecast.comparisons.scenarioMeta"),
      predictions: filterComparisonPoints(scenario.predictions, seriesId),
    });
  }

  for (const analysis of analyses) {
    if (analysis.id === current.id || !isCompatible(current, analysis)) continue;
    options.push({
      id: `forecast:${analysis.id}`,
      kind: "forecast",
      label: analysis.name,
      meta: t("forecast.comparisons.forecastMeta", {
        model: analysis.model,
        horizon: analysis.horizon,
      }),
      predictions: filterComparisonPoints(analysis.predictions, seriesId),
    });
  }
  return options;
}

export function buildComparisonRows(
  base: ForecastPoint[],
  compared: ForecastPoint[],
): ForecastComparisonRow[] {
  const size = Math.min(base.length, compared.length);
  return Array.from({ length: size }, (_, index) => {
    const baseValue = base[index].value;
    const compareValue = compared[index].value;
    const delta = compareValue - baseValue;
    return {
      index,
      date: base[index].date,
      baseValue,
      compareValue,
      delta,
      deltaPercent: baseValue === 0 ? 0 : (delta / baseValue) * 100,
    };
  });
}

export function buildComparisonStats(rows: ForecastComparisonRow[]): ForecastComparisonStats | null {
  if (!rows.length) return null;
  const averageDelta = rows.reduce((sum, row) => sum + row.delta, 0) / rows.length;
  const averageDeltaPercent = rows.reduce((sum, row) => sum + row.deltaPercent, 0) / rows.length;
  const maxDelta = rows.reduce((max, row) => Math.max(max, Math.abs(row.delta)), 0);
  const hasHigher = rows.some((row) => row.delta > 0);
  const hasLower = rows.some((row) => row.delta < 0);
  return {
    averageDelta,
    maxDelta,
    averageDeltaPercent,
    direction: hasHigher && hasLower ? "mixed" : hasHigher ? "higher" : "lower",
  };
}

function isCompatible(current: ForecastComparisonAnalysis, candidate: ForecastComparisonAnalysis): boolean {
  return (
    current.frequency === candidate.frequency &&
    current.horizon === candidate.horizon &&
    (current.target_column ?? "") === (candidate.target_column ?? "")
  );
}
