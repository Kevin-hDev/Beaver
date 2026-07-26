import { describe, expect, it } from "vitest";
import { inferMetricMeta } from "../forecast-view-format";
import {
  buildAdvancedVariableInsights,
  buildDecompositionCards,
  buildDriftCards,
  buildResidualAnomalyEvents,
} from "./forecast-advanced-analysis-utils";
import type { ForecastAdvancedAnalytics } from "./forecast-analysis-types";

const t = (key: string, values?: Record<string, unknown>) =>
  values ? `${key}:${JSON.stringify(values)}` : key;

function analytics(): ForecastAdvancedAnalytics {
  return {
    generated_at: "2026-07-23T00:00:00Z",
    decomposition: [{
      series_id: "A",
      status: "ready",
      method: "classical_additive",
      period: 7,
      seasonal_strength: 0.82,
    }],
    anomalies: [{
      id: "A:4",
      series_id: "A",
      date: "2026-07-05",
      observed: 180,
      expected: 100,
      residual: 80,
      score: 5.2,
      severity: "medium",
    }],
    variable_importance: {
      status: "ready",
      method: "chronological_permutation_on_naive_residual",
      reliability: "moderate",
      validation_points: 18,
      items: [{
        name: "price",
        score: 4,
        normalized_score: 0.72,
        direction: "negative",
      }],
    },
    drift: [{
      series_id: "A",
      status: "ready",
      score: 1.4,
      mean_shift: 1.2,
      variance_ratio: 1.1,
      distribution_shift: 0.45,
      detected: true,
      severity: "medium",
    }],
  };
}

describe("advanced forecast analysis mapping", () => {
  it("uses saved residual anomalies and respects the selected series", () => {
    const events = buildResidualAnomalyEvents(
      analytics(),
      "A",
      "fr-FR",
      inferMetricMeta("fr-FR", "revenue"),
      t,
    );
    expect(events).toHaveLength(1);
    expect(events[0].severity).toBe("medium");
    expect(buildResidualAnomalyEvents(analytics(), "B", "fr-FR", inferMetricMeta("fr-FR", "revenue"), t)).toEqual([]);
  });

  it("shows normalized permutation importance instead of amplitude", () => {
    const variables = buildAdvancedVariableInsights(analytics(), t);
    expect(variables[0].score).toBe(0.72);
    expect(variables[0].detail).toContain("72");
  });

  it("builds decomposition and drift cards from backend reports", () => {
    const decomposition = buildDecompositionCards(analytics(), "A", t);
    const drift = buildDriftCards(analytics(), "A", t);
    expect(decomposition).toHaveLength(3);
    expect(decomposition[2].value).toBe("82%");
    expect(drift[0].tone).toBe("warn");
  });
});
