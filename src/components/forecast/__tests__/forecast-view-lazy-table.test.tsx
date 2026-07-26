/* @vitest-environment jsdom */
// Finding 8: prediction rows must mount only when the table section is
// expanded, so collapsed cards don't render hundreds of hidden cells.
import { cleanup, fireEvent, render } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { ForecastViewResult } from "../sections/forecast-view-data";
import { ForecastView } from "../sections/forecast-view";

const FIXTURE: ForecastViewResult = {
  id: "a1",
  name: "fixture",
  target_column: "value",
  model: "chronos",
  horizon: 3,
  frequency: "M",
  input_summary: { end: "2025-03-01" },
  input_data: {
    history: [
      { date: "2025-01-01", value: 10 },
      { date: "2025-02-01", value: 20 },
      { date: "2025-03-01", value: 30 },
    ],
  },
  predictions: [
    { date: "2025-04-01", value: 40 },
    { date: "2025-05-01", value: 50 },
    { date: "2025-06-01", value: 60 },
  ],
  quantiles: { q10: [35, 45, 55], q50: [40, 50, 60], q90: [45, 55, 65] },
  scenarios: [],
  ensemble: null,
  metrics: null,
};

vi.mock("../use-forecast-result", () => ({
  useForecastResult: () => ({ data: FIXTURE, error: null }),
}));
vi.mock("react-i18next", async (importOriginal) => {
  const actual = await importOriginal<typeof import("react-i18next")>();
  return {
    ...actual,
    useTranslation: () => ({
      t: (key: string) => key,
      i18n: { language: "en" },
    }),
  };
});

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

describe("ForecastView lazy predictions table", () => {
  it("ne monte les lignes qu'une fois la section depliee", () => {
    render(<ForecastView analysisId="a1" layers={{}} />);
    expect(document.querySelectorAll(".fc-table-row")).toHaveLength(0);

    fireEvent.click(document.querySelector(".fc-table-toggle")!);
    expect(document.querySelectorAll(".fc-table-row")).toHaveLength(3);

    fireEvent.click(document.querySelector(".fc-table-toggle")!);
    expect(document.querySelectorAll(".fc-table-row")).toHaveLength(0);
  });
});
