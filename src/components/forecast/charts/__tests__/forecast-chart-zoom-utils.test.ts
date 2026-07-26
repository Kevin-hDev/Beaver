import { describe, expect, it } from "vitest";
import {
  clampForecastZoomWindow,
  computeWheelZoomWindow,
  forecastZoomSliderValue,
  normalizeWheelDelta,
  zoomAnchorRatio,
  FORECAST_CHART_MIN_ZOOM_SPAN,
  FORECAST_WHEEL_TICK_THRESHOLD,
  type ForecastZoomWindow,
} from "../forecast-chart-zoom-utils";

describe("forecast chart zoom utils", () => {
  it("borne le zoom au minimum visible par la barre", () => {
    const zoom = clampForecastZoomWindow(45, 46);
    expect(zoom.end - zoom.start).toBe(FORECAST_CHART_MIN_ZOOM_SPAN);
  });

  it("garde une fenetre trop zoomee dans les bornes du graphe", () => {
    expect(clampForecastZoomWindow(98, 99)).toEqual({ start: 90, end: 100 });
  });

  it("conserve le dezoom complet", () => {
    expect(clampForecastZoomWindow(-10, 130)).toEqual({ start: 0, end: 100 });
  });

  it("borne la valeur visuelle du slider", () => {
    expect(forecastZoomSliderValue(1)).toBe(100 - FORECAST_CHART_MIN_ZOOM_SPAN);
  });
});

describe("computeWheelZoomWindow zoom out", () => {
  it("agrandit symetriquement autour du centre courant", () => {
    const next = computeWheelZoomWindow({ start: 25, end: 75 }, 1, 0.9);
    expect(next.end - next.start).toBeCloseTo(56, 5);
    expect((next.start + next.end) / 2).toBeCloseTo(50, 5);
  });

  it("decale dans les bornes quand le centre est pres d'un bord", () => {
    const next = computeWheelZoomWindow({ start: 75, end: 100 }, 1, 0.5);
    expect(next.end).toBe(100);
    expect(next.end - next.start).toBeCloseTo(28, 4);
  });

  it("atteint exactement la pleine etendue sans la depasser", () => {
    expect(computeWheelZoomWindow({ start: 4, end: 94 }, 1, 0.5))
      .toEqual({ start: 0, end: 100 });
    expect(computeWheelZoomWindow({ start: 0, end: 100 }, 1, 0.5))
      .toEqual({ start: 0, end: 100 });
  });
});

describe("computeWheelZoomWindow zoom in", () => {
  it("retrecit vers le curseur au milieu", () => {
    const next = computeWheelZoomWindow({ start: 0, end: 100 }, -1, 0.5);
    expect(next.end - next.start).toBeCloseTo(100 / 1.12, 5);
    expect((next.start + next.end) / 2).toBeCloseTo(50, 5);
  });

  it("retrecit vers le curseur a gauche", () => {
    const next = computeWheelZoomWindow({ start: 0, end: 100 }, -1, 0);
    expect(next.start).toBeCloseTo(0, 5);
    expect(next.end - next.start).toBeCloseTo(100 / 1.12, 5);
  });

  it("retrecit vers le curseur a droite sans depasser le bord", () => {
    const next = computeWheelZoomWindow({ start: 0, end: 100 }, -1, 1);
    expect(next.end).toBeCloseTo(100, 5);
    expect(next.end - next.start).toBeCloseTo(100 / 1.12, 5);
  });

  it("respecte le plancher minSpan sans deplacer la fenetre", () => {
    const current = { start: 40, end: 40 + FORECAST_CHART_MIN_ZOOM_SPAN };
    expect(computeWheelZoomWindow(current, -1, 0.3)).toEqual(current);
  });

  it("ignore une direction nulle ou invalide", () => {
    expect(computeWheelZoomWindow({ start: 10, end: 90 }, 0, 0.5))
      .toEqual({ start: 10, end: 90 });
    expect(computeWheelZoomWindow({ start: 10, end: 90 }, Number.NaN, 0.5))
      .toEqual({ start: 10, end: 90 });
  });
});

describe("computeWheelZoomWindow ticks repetes", () => {
  it("reste dans les bornes sur 20 zooms avant puis 20 zooms arriere", () => {
    let window: ForecastZoomWindow = { start: 0, end: 100 };
    const anchors = [0, 0.25, 0.5, 0.75, 1];
    for (let i = 0; i < 20; i++) {
      window = computeWheelZoomWindow(window, -1, anchors[i % anchors.length]);
      expect(window.start).toBeGreaterThanOrEqual(0);
      expect(window.end).toBeLessThanOrEqual(100);
      expect(window.end - window.start).toBeGreaterThanOrEqual(
        FORECAST_CHART_MIN_ZOOM_SPAN - 1e-9,
      );
    }
    for (let i = 0; i < 20; i++) {
      window = computeWheelZoomWindow(window, 1, anchors[i % anchors.length]);
      expect(window.start).toBeGreaterThanOrEqual(0);
      expect(window.end).toBeLessThanOrEqual(100);
    }
    expect(window).toEqual({ start: 0, end: 100 });
  });
});

describe("normalizeWheelDelta", () => {
  it("convertit les modes ligne et page en pixels", () => {
    expect(normalizeWheelDelta(3, 0)).toBe(3);
    expect(normalizeWheelDelta(3, 1)).toBe(48);
    expect(normalizeWheelDelta(-1, 2)).toBe(-400);
  });

  it("ignore les deltas invalides", () => {
    expect(normalizeWheelDelta(Number.NaN, 0)).toBe(0);
    expect(normalizeWheelDelta(0, 1)).toBe(0);
  });

  it("garde un seuil de tick en pixels coherent", () => {
    // A classic 120px wheel notch must cross the threshold exactly once.
    expect(120 / FORECAST_WHEEL_TICK_THRESHOLD).toBeGreaterThanOrEqual(1);
    expect(FORECAST_WHEEL_TICK_THRESHOLD).toBeLessThanOrEqual(120);
  });
});

describe("zoomAnchorRatio", () => {
  it("mappe la position du curseur sur le trace", () => {
    expect(zoomAnchorRatio(50, 50, 700)).toBeCloseTo(0, 5);
    expect(zoomAnchorRatio(400, 50, 700)).toBeCloseTo(0.5, 5);
    expect(zoomAnchorRatio(750, 50, 700)).toBeCloseTo(1, 5);
  });

  it("borne hors trace et degrade en centre", () => {
    expect(zoomAnchorRatio(10, 50, 700)).toBe(0);
    expect(zoomAnchorRatio(900, 50, 700)).toBe(1);
    expect(zoomAnchorRatio(400, 0, 0)).toBe(0.5);
    expect(zoomAnchorRatio(Number.NaN, 0, 700)).toBe(0.5);
  });
});
