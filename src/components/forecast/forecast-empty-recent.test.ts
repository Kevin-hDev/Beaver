import { describe, expect, it } from "vitest";
import {
  newestForecastAnalyses,
  type ForecastAnalysisMeta,
} from "./forecast-empty-recent";

function analysis(id: string, createdAt: string): ForecastAnalysisMeta {
  return {
    id,
    name: id,
    created_at: createdAt,
    model: "model",
    horizon: 12,
    mape: null,
  };
}

describe("newestForecastAnalyses", () => {
  it("retient les cinq analyses les plus récentes", () => {
    const analyses = [
      analysis("old", "2026-01-01T00:00:00Z"),
      analysis("newest", "2026-07-01T00:00:00Z"),
      analysis("two", "2026-06-01T00:00:00Z"),
      analysis("three", "2026-05-01T00:00:00Z"),
      analysis("four", "2026-04-01T00:00:00Z"),
      analysis("five", "2026-03-01T00:00:00Z"),
    ];

    expect(newestForecastAnalyses(analyses).map((item) => item.id)).toEqual([
      "newest",
      "two",
      "three",
      "four",
      "five",
    ]);
  });
});
