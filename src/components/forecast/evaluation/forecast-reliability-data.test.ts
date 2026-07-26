import { describe, expect, it } from "vitest";
import type { ModelBacktestResult } from "./forecast-evaluation-types";
import {
  buildReliabilityBars,
  reliabilityMean,
} from "./forecast-reliability-data";

function result(
  modelId: string,
  smape: number | null,
  overrides: Partial<ModelBacktestResult> = {},
): ModelBacktestResult {
  return {
    model_id: modelId,
    kind: "model",
    metrics: smape == null
      ? null
      : { mase: 1, smape, mae: 1, rmse: 1, bias: 0, stability: 1 },
    calibration: null,
    duration_ms: 10,
    rank: null,
    beats_best_baseline: null,
    warning: null,
    ...overrides,
  };
}

describe("buildReliabilityBars", () => {
  it("keeps only results with a finite sMAPE", () => {
    const bars = buildReliabilityBars(
      [result("a", 4.2), result("b", null), result("c", Number.NaN)],
      "a",
    );
    expect(bars.map((bar) => bar.modelId)).toEqual(["a"]);
  });

  it("orders by rank and flags the current model", () => {
    const bars = buildReliabilityBars(
      [
        result("chronos", 3.1, { rank: 1 }),
        result("naive", 5.4, { rank: 2, kind: "baseline" }),
      ],
      "chronos",
    );

    expect(bars[0]).toMatchObject({ modelId: "chronos", current: true, value: 3.1 });
    expect(bars[1]).toMatchObject({ modelId: "naive", kind: "baseline", current: false });
  });
});

describe("reliabilityMean", () => {
  it("averages the bar values", () => {
    const bars = buildReliabilityBars(
      [result("a", 2), result("b", 4)],
      "a",
    );
    expect(reliabilityMean(bars)).toBe(3);
  });

  it("returns null without bars", () => {
    expect(reliabilityMean([])).toBeNull();
  });
});
