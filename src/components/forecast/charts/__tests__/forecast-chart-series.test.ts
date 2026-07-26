import { describe, expect, it } from "vitest";
import { buildSeries } from "../forecast-chart-series";
import {
  FORECAST_CHART_LINE_MONOTONE_AXIS,
  FORECAST_CHART_LINE_SMOOTHING,
} from "../forecast-chart-line-style";
import type {
  ForecastChartPalette,
  TimelineEntry,
} from "../forecast-chart-types";

const palette: ForecastChartPalette = {
  lineHistory: "history",
  linePredict: "forecast",
  pointPredict: "point",
  band90: "band",
  separator: "separator",
  forecastZone: "zone",
  areaHistoryFrom: "area-from",
  areaHistoryTo: "area-to",
  annotationUser: "user",
  annotationLlm: "llm",
  edge: "edge",
  inkMuted: "muted",
  tooltipBg: "tooltip",
  tooltipText: "text",
  scenarios: ["scenario"],
  variables: ["variable"],
};

const timeline: TimelineEntry[] = [0, 1, 2].map((index) => ({
  axisLabel: `${index}`,
  dateKey: `2026-01-0${index + 1}`,
  timestamp: index,
  fullLabel: `${index}`,
  historyValue: index,
  forecastValue: index + 1,
  scenarioValues: [],
  variableValues: [],
  annotationValues: [],
  lowerValue: index,
  upperValue: index + 2,
}));

interface DecoratedLine {
  lineStyle?: { type?: unknown };
  areaStyle?: { color?: { colorStops?: { offset: number; color: string }[] } };
  markLine?: { label?: { formatter?: string } };
  markArea?: { itemStyle?: { color?: string } };
}

describe("forecast chart line series", () => {
  it("smooths every line along time without connecting missing values", () => {
    const series = buildSeries(
      timeline,
      1,
      palette,
      [],
      [],
      {
        history: "History",
        forecast: "Forecast",
        confidence: "Confidence",
        forecastStart: "Forecast start",
        annotationUser: "User",
        annotationLlm: "LLM",
      },
      { history: true, forecast: true, confidence: true },
      null,
    );
    const lines = series.filter((item) => item.type === "line");

    expect(lines).not.toHaveLength(0);
    for (const line of lines) {
      expect(line.smooth).toBe(FORECAST_CHART_LINE_SMOOTHING);
      expect(line.smoothMonotone).toBe(FORECAST_CHART_LINE_MONOTONE_AXIS);
      expect(line.connectNulls).toBe(false);
    }
  });

  it("only renders annotations whose layer is enabled", () => {
    const annotated: TimelineEntry[] = [{
      ...timeline[0],
      annotationValues: [{
        id: "anomaly",
        date: "2026-01-01",
        text: "Anomaly",
        source: "llm",
        kind: "anomalies",
      }],
    }];
    const labels = {
      history: "History",
      forecast: "Forecast",
      confidence: "Confidence",
      forecastStart: "Forecast start",
      annotationUser: "User",
      annotationLlm: "LLM",
    };

    const hidden = buildSeries(
      annotated, null, palette, [], [], labels,
      { history: true, forecast: true, confidence: true, anomalies: false },
      null,
    );
    const visible = buildSeries(
      annotated, null, palette, [], [], labels,
      { history: true, forecast: true, confidence: true, anomalies: true },
      null,
    );

    const hiddenAnnotation = hidden.find(
      (item) => "id" in item && item.id === "annotation-llm",
    );
    const visibleAnnotation = visible.find(
      (item) => "id" in item && item.id === "annotation-llm",
    );

    expect(hiddenAnnotation?.data[0]).toBeNull();
    expect(visibleAnnotation?.data[0]).not.toBeNull();
  });

  it("renders the forecast dashed with a tinted zone and a labeled separator", () => {
    const labels = {
      history: "History",
      forecast: "Forecast",
      confidence: "Confidence",
      forecastStart: "Forecast start",
      annotationUser: "User",
      annotationLlm: "LLM",
    };
    const series = buildSeries(
      timeline,
      1,
      palette,
      [],
      [],
      labels,
      { history: true, forecast: true, confidence: true },
      null,
    );
    const forecast = series.find((item) => item.name === "Forecast") as
      | DecoratedLine
      | undefined;
    const history = series.find((item) => item.name === "History") as
      | DecoratedLine
      | undefined;

    expect(forecast?.lineStyle?.type).toEqual([5, 4]);
    expect(forecast?.markLine?.label?.formatter).toBe("Forecast start");
    expect(forecast?.markArea?.itemStyle?.color).toBe("zone");
    expect(history?.areaStyle?.color?.colorStops).toEqual([
      { offset: 0, color: "area-from" },
      { offset: 1, color: "area-to" },
    ]);
    expect(history?.lineStyle?.type).toBeUndefined();
  });

  it("omits the forecast zone and separator without a separator timestamp", () => {
    const labels = {
      history: "History",
      forecast: "Forecast",
      confidence: "Confidence",
      forecastStart: "Forecast start",
      annotationUser: "User",
      annotationLlm: "LLM",
    };
    const series = buildSeries(
      timeline,
      null,
      palette,
      [],
      [],
      labels,
      { history: true, forecast: true, confidence: true },
      null,
    );
    const forecast = series.find((item) => item.name === "Forecast") as
      | DecoratedLine
      | undefined;

    expect(forecast?.markLine).toBeUndefined();
    expect(forecast?.markArea).toBeUndefined();
  });
});
