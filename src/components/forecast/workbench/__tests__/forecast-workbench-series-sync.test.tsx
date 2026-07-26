/* @vitest-environment jsdom */
// Integration test for finding 1: switching the active series in the main
// chart toolbar must propagate to the fan and seasonality companion cards.
import { cleanup, fireEvent, render, screen, within } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { ForecastViewResult } from "../../sections/forecast-view-data";
import { ForecastWorkbenchForecast } from "../forecast-workbench-forecast";

const EMPTY_SOURCES = {
  scenarioLayers: [],
  comparisonLayers: [],
  covariateNames: [],
  eventLayers: [],
  anomalyLayers: [],
  qualityLayers: [],
};

vi.mock("../../use-forecast-result", () => ({
  useForecastResult: () => ({ data: buildFixture(), error: null }),
}));
vi.mock("../../use-forecast-layer-sources", () => ({
  useForecastLayerSources: () => ({ sources: EMPTY_SOURCES }),
}));
vi.mock("react-i18next", async (importOriginal) => {
  const actual = await importOriginal<typeof import("react-i18next")>();
  return {
    ...actual,
    useTranslation: () => ({
      t: (key: string, opts?: { title?: string; series?: string }) =>
        opts ? `${opts.title} — ${opts.series}` : key,
      i18n: { language: "en" },
    }),
  };
});

function buildFixture(): ForecastViewResult {
  const month = (index: number) => {
    const year = 2023 + Math.floor(index / 12);
    return `${year}-${String((index % 12) + 1).padStart(2, "0")}-01`;
  };
  const history = ["A", "B"].flatMap((seriesId, s) =>
    Array.from({ length: 30 }, (_, i) => ({
      date: month(i),
      value: s * 1000 + i + 10,
      series_id: seriesId,
    })),
  );
  const predictions = ["A", "B"].flatMap((seriesId, s) =>
    Array.from({ length: 6 }, (_, i) => ({
      date: month(30 + i),
      value: s * 1000 + i + 40,
      series_id: seriesId,
    })),
  );
  return {
    id: "a1",
    name: "fixture",
    target_column: "value",
    series_column: "series",
    model: "chronos",
    horizon: 6,
    frequency: "M",
    input_summary: { end: "2025-06-01" },
    input_data: { rows: [], series_ids: ["A", "B"], history },
    covariates_used: [],
    predictions,
    quantiles: {
      q10: predictions.map((_, i) => i),
      q50: predictions.map((_, i) => i + 1),
      q90: predictions.map((_, i) => i + 2),
    },
    scenarios: [],
    ensemble: null,
    metrics: null,
    annotations: [],
    advanced_analytics: null,
    data_profile: null,
  };
}

function stubCanvas() {
  const gradient = { addColorStop: () => undefined };
  const base = {
    canvas: {},
    measureText: () => ({ width: 0 }),
    createLinearGradient: () => gradient,
    createRadialGradient: () => gradient,
    createPattern: () => null,
  };
  const ctx = new Proxy(base, {
    get: (target, prop) =>
      prop in target ? target[prop as keyof typeof target] : () => undefined,
    set: () => true,
  });
  vi.spyOn(HTMLCanvasElement.prototype, "getContext").mockImplementation(
    (() => ctx) as unknown as typeof HTMLCanvasElement.prototype.getContext,
  );
}

beforeEach(() => {
  vi.stubGlobal("ResizeObserver", class {
    observe() {}
    unobserve() {}
    disconnect() {}
  });
  if (typeof globalThis.requestAnimationFrame !== "function") {
    vi.stubGlobal("requestAnimationFrame", (cb: FrameRequestCallback) =>
      setTimeout(() => cb(performance.now()), 0));
    vi.stubGlobal("cancelAnimationFrame", (id: number) => clearTimeout(id));
  }
  vi.spyOn(HTMLElement.prototype, "clientWidth", "get").mockReturnValue(800);
  vi.spyOn(HTMLElement.prototype, "clientHeight", "get").mockReturnValue(400);
  stubCanvas();
});

afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
  vi.restoreAllMocks();
});

const cardTitles = () =>
  [...document.querySelectorAll(".fcrd-title")].map((node) => node.textContent);

describe("ForecastWorkbenchForecast series sync", () => {
  it("propage la serie active du graphique principal aux deux compagnons", async () => {
    render(<ForecastWorkbenchForecast analysisId="a1" />);

    expect(cardTitles()).toEqual([
      "forecast.chartCard.main",
      "forecast.chartCard.fan — A",
      "forecast.chartCard.seasonality — A",
    ]);
    // Seasonality legend proves the companion received series A history.
    expect(document.querySelectorAll(".fcse-chip").length).toBe(3);

    const trigger = document.querySelector<HTMLButtonElement>(".fcs-menu-trigger");
    expect(trigger).toBeTruthy();
    fireEvent.click(trigger!);
    const menu = await screen.findByRole("menu");
    fireEvent.click(within(menu).getByText("B"));

    expect(cardTitles()).toEqual([
      "forecast.chartCard.main",
      "forecast.chartCard.fan — B",
      "forecast.chartCard.seasonality — B",
    ]);
    expect(
      document.querySelector(".fcs-menu-label")?.textContent,
    ).toBe("B");
    // The seasonality model rebuilt for series B (same year coverage).
    expect(document.querySelectorAll(".fcse-chip").length).toBe(3);
  });
});
