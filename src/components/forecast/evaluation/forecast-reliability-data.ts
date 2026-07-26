import type { ModelBacktestResult } from "./forecast-evaluation-types";
import { rankedResults } from "./forecast-evaluation-utils";

export interface ReliabilityBar {
  modelId: string;
  value: number;
  kind: "baseline" | "model";
  current: boolean;
}

/**
 * The rolling backtest only stores aggregate metrics per model (no
 * per-window breakdown), so the reliability chart plots one bar per
 * evaluated model using its aggregate sMAPE.
 */
export function buildReliabilityBars(
  results: ModelBacktestResult[],
  currentModel: string,
): ReliabilityBar[] {
  return rankedResults(results)
    .filter(
      (result) =>
        result.metrics !== null && Number.isFinite(result.metrics.smape),
    )
    .map((result) => ({
      modelId: result.model_id,
      value: result.metrics?.smape ?? 0,
      kind: result.kind,
      current: result.model_id === currentModel,
    }));
}

export function reliabilityMean(bars: ReliabilityBar[]): number | null {
  if (!bars.length) return null;
  return bars.reduce((total, bar) => total + bar.value, 0) / bars.length;
}
