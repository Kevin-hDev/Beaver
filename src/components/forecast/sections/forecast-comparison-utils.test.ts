import { describe, it, expect } from "vitest";
import { buildComparisonRows, buildComparisonStats } from "./forecast-comparison-utils";
import type { ForecastPoint } from "./forecast-comparison-types";

function point(date: string, value: number): ForecastPoint {
  return { date, value };
}

describe("buildComparisonRows", () => {
  it("calcule delta et deltaPercent entre deux séries", () => {
    const base = [point("2026-01-01", 100), point("2026-01-02", 200)];
    const compared = [point("2026-01-01", 110), point("2026-01-02", 250)];

    const rows = buildComparisonRows(base, compared);

    expect(rows).toHaveLength(2);
    expect(rows[0].delta).toBe(10);
    expect(rows[0].deltaPercent).toBe(10); // +10%
    expect(rows[1].delta).toBe(50);
    expect(rows[1].deltaPercent).toBe(25); // +25%
  });

  it("gère les deltas négatifs", () => {
    const base = [point("d1", 200)];
    const compared = [point("d1", 150)];

    const rows = buildComparisonRows(base, compared);

    expect(rows[0].delta).toBe(-50);
    expect(rows[0].deltaPercent).toBe(-25);
  });

  it("retourne deltaPercent 0 quand baseValue est 0 (pas de division par zéro)", () => {
    const base = [point("d1", 0)];
    const compared = [point("d1", 50)];

    const rows = buildComparisonRows(base, compared);

    expect(rows[0].delta).toBe(50);
    expect(rows[0].deltaPercent).toBe(0); // évite NaN/Infinity
  });

  it("tronque à la série la plus courte", () => {
    const base = [point("d1", 1), point("d2", 2), point("d3", 3)];
    const compared = [point("d1", 10)];

    const rows = buildComparisonRows(base, compared);

    expect(rows).toHaveLength(1);
  });

  it("retourne un tableau vide si les deux séries sont vides", () => {
    expect(buildComparisonRows([], [])).toEqual([]);
  });

  it("préserve la date de la série de base", () => {
    const base = [point("2026-01-01", 100)];
    const compared = [point("2026-02-01", 110)]; // date différente

    const rows = buildComparisonRows(base, compared);

    expect(rows[0].date).toBe("2026-01-01");
  });
});

describe("buildComparisonStats", () => {
  it("retourne null pour un tableau vide", () => {
    expect(buildComparisonStats([])).toBeNull();
  });

  it("calcule la moyenne des deltas", () => {
    const rows = [
      { index: 0, date: "d1", baseValue: 100, compareValue: 110, delta: 10, deltaPercent: 10 },
      { index: 1, date: "d2", baseValue: 100, compareValue: 130, delta: 30, deltaPercent: 30 },
    ];
    const stats = buildComparisonStats(rows);

    expect(stats).not.toBeNull();
    expect(stats!.averageDelta).toBe(20);
    expect(stats!.averageDeltaPercent).toBe(20);
  });

  it("détecte la direction 'higher' (tous deltas positifs)", () => {
    const rows = [
      { index: 0, date: "d1", baseValue: 100, compareValue: 110, delta: 10, deltaPercent: 10 },
    ];
    const stats = buildComparisonStats(rows);

    expect(stats!.direction).toBe("higher");
  });

  it("détecte la direction 'lower' (tous deltas négatifs)", () => {
    const rows = [
      { index: 0, date: "d1", baseValue: 100, compareValue: 90, delta: -10, deltaPercent: -10 },
    ];
    const stats = buildComparisonStats(rows);

    expect(stats!.direction).toBe("lower");
  });

  it("détecte la direction 'mixed' (deltas positifs et négatifs)", () => {
    const rows = [
      { index: 0, date: "d1", baseValue: 100, compareValue: 110, delta: 10, deltaPercent: 10 },
      { index: 1, date: "d2", baseValue: 100, compareValue: 90, delta: -10, deltaPercent: -10 },
    ];
    const stats = buildComparisonStats(rows);

    expect(stats!.direction).toBe("mixed");
  });

  it("calcule le delta absolu maximum", () => {
    const rows = [
      { index: 0, date: "d1", baseValue: 100, compareValue: 105, delta: 5, deltaPercent: 5 },
      { index: 1, date: "d2", baseValue: 100, compareValue: 130, delta: 30, deltaPercent: 30 },
      { index: 2, date: "d3", baseValue: 100, compareValue: 90, delta: -10, deltaPercent: -10 },
    ];
    const stats = buildComparisonStats(rows);

    expect(stats!.maxDelta).toBe(30);
  });
});
