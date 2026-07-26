import { describe, expect, it } from "vitest";
import { formatCoverage, formatDuration, rankedResults } from "./forecast-evaluation-utils";
import type { ModelBacktestResult } from "./forecast-evaluation-types";

function result(model_id: string, rank: number | null): ModelBacktestResult {
  return {
    model_id,
    rank,
    kind: "baseline",
    metrics: null,
    calibration: null,
    duration_ms: 0,
    beats_best_baseline: null,
    warning: null,
  };
}

describe("forecast evaluation utils", () => {
  it("keeps ranked results before unavailable models", () => {
    expect(rankedResults([result("b", null), result("a", 1)]).map((item) => item.model_id))
      .toEqual(["a", "b"]);
  });

  it("formats measured coverage as a percentage", () => {
    expect(formatCoverage(0.875)).toBe("87.5%");
    expect(formatDuration(1250)).toBe("1.3 s");
  });
});
