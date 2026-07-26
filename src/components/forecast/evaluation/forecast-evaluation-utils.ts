import type { ModelBacktestResult } from "./forecast-evaluation-types";

export function rankedResults(results: ModelBacktestResult[]): ModelBacktestResult[] {
  return [...results].sort((left, right) => {
    const leftRank = left.rank ?? Number.MAX_SAFE_INTEGER;
    const rightRank = right.rank ?? Number.MAX_SAFE_INTEGER;
    return leftRank - rightRank || left.model_id.localeCompare(right.model_id);
  });
}

export function formatMetric(value: number | null | undefined, digits = 2): string {
  return Number.isFinite(value) ? Number(value).toFixed(digits) : "—";
}

export function formatCoverage(value: number | null | undefined): string {
  return Number.isFinite(value) ? `${(Number(value) * 100).toFixed(1)}%` : "—";
}

export function formatDuration(milliseconds: number): string {
  if (!Number.isFinite(milliseconds) || milliseconds < 0) return "—";
  return milliseconds < 1000
    ? `${Math.round(milliseconds)} ms`
    : `${(milliseconds / 1000).toFixed(1)} s`;
}

export function baselineTranslationKey(modelId: string): string {
  return ["naive", "seasonal_naive", "drift", "ets"].includes(modelId)
    ? `forecast.workbench.evaluation.baselines.${modelId}`
    : "";
}
