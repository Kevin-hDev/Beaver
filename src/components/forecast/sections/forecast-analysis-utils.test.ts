import { describe, it, expect } from "vitest";
import {
  buildTrendCards,
  buildUncertaintyCards,
  buildHighlightEvents,
  filterAnalysisPoints,
} from "./forecast-analysis-utils";
import { inferMetricMeta } from "../forecast-view-format";
import type { ForecastAnalysisPoint } from "./forecast-analysis-types";

function pt(date: string, value: number, series_id?: string): ForecastAnalysisPoint {
  return { date, value, ...(series_id !== undefined ? { series_id } : {}) };
}

const t = (key: string) => key;
const metric = inferMetricMeta("fr-FR", "revenue");

// --- buildTrendCards --------------------------------------------------------

describe("buildTrendCards", () => {
  it("calcule la direction rising quand la tendance monte de +>2%", () => {
    const cards = buildTrendCards([pt("d1", 100), pt("d2", 110)], "fr-FR", metric, t);
    const directionCard = cards.find((c) => c.labelKey === "forecast.analysis.direction");
    expect(directionCard?.tone).toBe("good");
    expect(directionCard?.value).toBe("forecast.analysis.rising");
  });

  it("calcule la direction falling quand la tendance baisse de >2%", () => {
    const cards = buildTrendCards([pt("d1", 100), pt("d2", 90)], "fr-FR", metric, t);
    const directionCard = cards.find((c) => c.labelKey === "forecast.analysis.direction");
    expect(directionCard?.tone).toBe("warn");
    expect(directionCard?.value).toBe("forecast.analysis.falling");
  });

  it("calcule la direction stable quand variation < 2%", () => {
    const cards = buildTrendCards([pt("d1", 100), pt("d2", 101)], "fr-FR", metric, t);
    const directionCard = cards.find((c) => c.labelKey === "forecast.analysis.direction");
    expect(directionCard?.tone).toBe("neutral");
    expect(directionCard?.value).toBe("forecast.analysis.stable");
  });

  it("gère une prédiction unique (first == last)", () => {
    const cards = buildTrendCards([pt("d1", 50)], "fr-FR", metric, t);
    expect(cards).toHaveLength(4);
    const totalCard = cards.find((c) => c.labelKey === "forecast.analysis.totalChange");
    expect(totalCard?.value).toContain("0.0%");
  });

  it("retourne exactement 4 cartes", () => {
    const cards = buildTrendCards([pt("d1", 100), pt("d2", 200)], "fr-FR", metric, t);
    expect(cards).toHaveLength(4);
  });
});

// --- buildUncertaintyCards --------------------------------------------------

describe("buildUncertaintyCards", () => {
  it("calcule la largeur moyenne et max de l'intervalle de confiance", () => {
    const quantiles = { q10: [80, 90], q90: [120, 110] };
    const predictions = [pt("d1", 100), pt("d2", 100)];
    const cards = buildUncertaintyCards(quantiles, predictions, "fr-FR", metric);

    expect(cards).toHaveLength(3);
    // Largeurs : |120-80|=40, |110-90|=20 → moyenne 30, max 40.
    const maxRangeCard = cards.find((c) => c.labelKey === "forecast.analysis.maxRange");
    expect(maxRangeCard?.value).toContain("40");
  });

  it("gère un intervalle nul (q10 == q90)", () => {
    const quantiles = { q10: [100, 100], q90: [100, 100] };
    const predictions = [pt("d1", 100), pt("d2", 100)];
    const cards = buildUncertaintyCards(quantiles, predictions, "fr-FR", metric);
    const avgCard = cards.find((c) => c.labelKey === "forecast.analysis.averageRange");
    expect(avgCard?.value).toContain("0");
  });

  it("met un tone warn si le max dépasse 1.5x la moyenne", () => {
    // Largeurs : 10 et 100 → moyenne 55, max 100 → 100 > 55*1.5=82.5 → warn.
    const quantiles = { q10: [0, 0], q90: [10, 100] };
    const predictions = [pt("d1", 5), pt("d2", 50)];
    const cards = buildUncertaintyCards(quantiles, predictions, "fr-FR", metric);
    const maxRangeCard = cards.find((c) => c.labelKey === "forecast.analysis.maxRange");
    expect(maxRangeCard?.tone).toBe("warn");
  });
});

// --- buildHighlightEvents --------------------------------------------------

describe("buildHighlightEvents", () => {
  it("retourne 4 événements (high, low, up, down)", () => {
    const preds = [pt("d1", 100), pt("d2", 150), pt("d3", 80)];
    const events = buildHighlightEvents(preds, "d1", "D", "fr-FR", metric, t);

    expect(events).toHaveLength(4);
    const ids = events.map((e) => e.id);
    expect(ids).toContain("high");
    expect(ids).toContain("low");
    expect(ids).toContain("up");
    expect(ids).toContain("down");
  });

  it("retourne un tableau vide pour des prédictions vides", () => {
    expect(buildHighlightEvents([], "d1", "D", "fr-FR", metric, t)).toEqual([]);
  });

  it("identifie le point le plus haut", () => {
    const preds = [pt("d1", 100), pt("d2", 200), pt("d3", 50)];
    const events = buildHighlightEvents(preds, "d1", "D", "fr-FR", metric, t);
    const highEvent = events.find((e) => e.id === "high");

    expect(highEvent?.value).toContain("200");
  });
});

// --- filterAnalysisPoints --------------------------------------------------

describe("filterAnalysisPoints", () => {
  it("retourne tous les points si seriesId est vide", () => {
    const points = [pt("d1", 1, "A"), pt("d2", 2, "B")];
    expect(filterAnalysisPoints(points, "")).toHaveLength(2);
  });

  it("filtre par seriesId", () => {
    const points = [pt("d1", 1, "A"), pt("d2", 2, "B"), pt("d3", 3, "A")];
    const filtered = filterAnalysisPoints(points, "A");
    expect(filtered).toHaveLength(2);
    expect(filtered.every((p) => p.series_id === "A")).toBe(true);
  });

  it("inclut les points sans series_id", () => {
    const points = [pt("d1", 1), pt("d2", 2, "A")];
    const filtered = filterAnalysisPoints(points, "A");
    expect(filtered).toHaveLength(2); // le point sans series_id est inclus
  });
});
