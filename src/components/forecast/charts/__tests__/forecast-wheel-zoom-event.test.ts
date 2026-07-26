// Locks the wheel ownership contract: the inside dataZoom must keep
// zoomOnMouseWheel disabled (our own pipeline owns the wheel), so a wheel
// event over the chart produces no ECharts roam change at all.
import { describe, expect, it } from "vitest";
import * as echarts from "echarts";
import { buildForecastChartOption } from "../forecast-chart-option";
import type {
  ForecastChartOptionArgs,
  ForecastChartPalette,
} from "../forecast-chart-types";

const wait = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

const palette: ForecastChartPalette = {
  lineHistory: "#111111",
  linePredict: "#222222",
  pointPredict: "#333333",
  band90: "#444444",
  separator: "#555555",
  forecastZone: "#666666",
  areaHistoryFrom: "#777777",
  areaHistoryTo: "#888888",
  annotationUser: "#999999",
  annotationLlm: "#aaaaaa",
  edge: "#bbbbbb",
  inkMuted: "#cccccc",
  tooltipBg: "#dddddd",
  tooltipText: "#eeeeee",
  scenarios: [],
  variables: [],
};

function buildArgs(): ForecastChartOptionArgs {
  const base = Date.UTC(2020, 0, 1);
  const day = 86400000;
  const history = Array.from({ length: 60 }, (_, i) => ({
    date: new Date(base + i * day).toISOString(),
    value: i,
  }));
  const predictions = Array.from({ length: 30 }, (_, i) => ({
    date: new Date(base + (60 + i) * day).toISOString(),
    value: 60 + i,
  }));
  return {
    history,
    predictions,
    scenarios: [],
    variables: [],
    annotations: [],
    zoomWindow: { start: 0, end: 100 },
    chartWidth: 800,
    compact: false,
    quantiles: { q10: [], q90: [] },
    frequency: "D",
    endDate: predictions[predictions.length - 1].date,
    locale: "en",
    layers: { history: true, forecast: true, confidence: true },
    palette,
    labels: {
      history: "History",
      forecast: "Forecast",
      confidence: "Confidence",
      forecastStart: "Start",
      annotationUser: "User",
      annotationLlm: "LLM",
    },
  };
}

describe("wheel ownership", () => {
  it("desactive le zoom molette du roam echarts dans l'option", () => {
    const option = buildForecastChartOption(buildArgs());
    const dataZoom = (option.dataZoom as {
      zoomOnMouseWheel?: boolean;
      moveOnMouseWheel?: boolean;
      moveOnMouseMove?: boolean;
    }[])[0];
    expect(dataZoom.zoomOnMouseWheel).toBe(false);
    expect(dataZoom.moveOnMouseWheel).toBe(false);
    expect(dataZoom.moveOnMouseMove).toBe(false);
  });

  it("ne reagit plus a la molette cote roam echarts", async () => {
    const dom = document.createElement("div");
    document.body.appendChild(dom);
    const chart = echarts.init(dom, undefined, {
      renderer: "svg",
      width: 800,
      height: 400,
    });
    chart.setOption(buildForecastChartOption(buildArgs()));
    const target = dom.firstElementChild ?? dom;
    let events = 0;
    chart.on("datazoom", () => {
      events += 1;
    });
    for (const deltaY of [-120, 120, -120, 120]) {
      target.dispatchEvent(
        new WheelEvent("wheel", {
          deltaY,
          clientX: 400,
          clientY: 150,
          bubbles: true,
          cancelable: true,
        }),
      );
      await wait(50);
    }
    const zoom = (chart.getOption().dataZoom as { start: number; end: number }[])[0];
    chart.dispose();
    dom.remove();
    expect(events).toBe(0);
    expect(zoom.start).toBe(0);
    expect(zoom.end).toBe(100);
  });
});
