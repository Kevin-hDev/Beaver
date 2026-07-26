import { describe, expect, it } from "vitest";
import {
  buildForecastLayerAnnotations,
  type ForecastViewResult,
} from "./forecast-view-data";

function analysis(): ForecastViewResult {
  return {
    id: "analysis",
    name: "Forecast",
    model: "model",
    horizon: 1,
    frequency: "D",
    input_summary: { end: "2026-07-23" },
    input_data: { history: [] },
    predictions: [],
    quantiles: { q10: [], q50: [], q90: [] },
    scenarios: [],
    metrics: null,
    annotations: [{
      id: "note",
      date: "2026-07-21",
      text: "Decision",
      source: "user",
    }],
    advanced_analytics: {
      anomalies: [{
        id: "anomaly",
        date: "2026-07-22",
        score: 2.5,
        series_id: "north",
      }],
    },
    data_profile: {
      issues: [{ code: "missing_periods", count: 2 }],
    },
  };
}

describe("buildForecastLayerAnnotations", () => {
  it("maps notes, anomalies and quality issues to separate chart layers", () => {
    const result = buildForecastLayerAnnotations(analysis(), "north", {
      anomaly: "Anomaly",
      quality: "Quality",
    });

    expect(result.map((item) => item.kind)).toEqual([
      "annotations",
      "anomalies",
      "quality",
    ]);
  });

  it("keeps anomalies scoped to the selected series", () => {
    const result = buildForecastLayerAnnotations(analysis(), "south", {
      anomaly: "Anomaly",
      quality: "Quality",
    });

    expect(result.some((item) => item.kind === "anomalies")).toBe(false);
  });
});
