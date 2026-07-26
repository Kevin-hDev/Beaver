/* @vitest-environment jsdom */
// Findings 4+5: the wheel handler must preventDefault ONLY when the zoom
// window will actually change, and trackpad micro-deltas must accumulate
// into discrete rAF-throttled ticks instead of one update per event.
import { act, cleanup, render } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { ForecastChart } from "../forecast-chart";
import {
  FORECAST_WHEEL_GESTURE_IDLE_MS,
  FORECAST_WHEEL_TICK_THRESHOLD,
} from "../forecast-chart-zoom-utils";

const wait = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));
const flush = (ms: number) => act(() => wait(ms));

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

function renderChart() {
  const base = Date.UTC(2020, 0, 1);
  const day = 86400000;
  const point = (i: number) => ({
    date: new Date(base + i * day).toISOString(),
    value: i,
  });
  render(
    <ForecastChart
      history={Array.from({ length: 60 }, (_, i) => point(i))}
      predictions={Array.from({ length: 30 }, (_, i) => point(60 + i))}
      scenarios={[]}
      variables={[]}
      annotations={[]}
      quantiles={{ q10: [], q90: [] }}
      frequency="D"
      endDate={point(89).date}
      locale="en"
      labels={{
        history: "History",
        forecast: "Forecast",
        confidence: "Confidence",
        forecastStart: "Start",
        annotationUser: "User",
        annotationLlm: "LLM",
      }}
      layers={{ history: true, forecast: true, confidence: true }}
      mode="main"
    />,
  );
  const shell = document.querySelector<HTMLDivElement>(".fcc-chart-shell");
  if (!shell) throw new Error("chart shell missing");
  return shell;
}

function wheel(shell: HTMLDivElement, deltaY: number, deltaMode = 0) {
  const event = new WheelEvent("wheel", {
    deltaY,
    deltaMode,
    clientX: 400,
    clientY: 150,
    bubbles: true,
    cancelable: true,
  });
  shell.dispatchEvent(event);
  return event;
}

const sliderValue = () =>
  Number(document.querySelector<HTMLInputElement>(".fcc-chart-zoom")?.value);

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

describe("wheel preventDefault contract", () => {
  it("ne bloque pas un cran sans effet a la pleine etendue", async () => {
    const shell = renderChart();
    await flush(20);
    expect(sliderValue()).toBe(0);
    // Zoom OUT while already at full extent: no-op, page stays scrollable.
    expect(wheel(shell, 120).defaultPrevented).toBe(false);
    await flush(30);
    expect(sliderValue()).toBe(0);
  });

  it("bloque un cran qui zoome reellement", async () => {
    const shell = renderChart();
    await flush(20);
    expect(wheel(shell, -120).defaultPrevented).toBe(true);
    await flush(30);
    expect(sliderValue()).toBeGreaterThan(0);
  });

  it("accumule les micro-deltas trackpad en ticks discrets", async () => {
    const shell = renderChart();
    await flush(20);
    // Below-threshold micro-deltas of a real gesture are still ours.
    expect(wheel(shell, -10).defaultPrevented).toBe(true);
    await flush(30);
    expect(sliderValue()).toBe(0); // 10px < threshold: no tick yet
    wheel(shell, -10);
    wheel(shell, -10);
    wheel(shell, -10); // 40px total: one tick queued for the next frame
    await flush(30);
    expect(sliderValue()).toBeGreaterThan(0);
    expect(FORECAST_WHEEL_TICK_THRESHOLD).toBe(40);
  });

  it("normalise le mode ligne en pixels", async () => {
    const shell = renderChart();
    await flush(20);
    wheel(shell, -3, 1); // 3 lines * 16px = 48px > threshold
    await flush(30);
    expect(sliderValue()).toBeGreaterThan(0);
  });

  it("ne cumule pas deux gestes trackpad separes", async () => {
    const shell = renderChart();
    await flush(20);
    wheel(shell, -20);
    await flush(FORECAST_WHEEL_GESTURE_IDLE_MS + 20);
    wheel(shell, -20);
    await flush(30);
    expect(sliderValue()).toBe(0);
    wheel(shell, -20);
    await flush(30);
    expect(sliderValue()).toBeGreaterThan(0);
  });
});
