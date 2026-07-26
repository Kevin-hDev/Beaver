import type { ForecastLayerState } from "../forecast-layer-matrix";
import type {
  ForecastChartOptionArgs,
  ForecastChartPalette,
  ScenarioLine,
  TimelineEntry,
  VariableLine,
} from "./forecast-chart-types";
import {
  FORECAST_CHART_LINE_MONOTONE_AXIS,
  FORECAST_CHART_LINE_SMOOTHING,
} from "./forecast-chart-line-style";
import { buildAnnotationSeries } from "./forecast-chart-annotations";

export function buildSeries(
  timeline: TimelineEntry[],
  separatorTimestamp: number | null,
  palette: ForecastChartPalette,
  scenarios: ScenarioLine[],
  variables: VariableLine[],
  labels: ForecastChartOptionArgs["labels"],
  layers: ForecastLayerState,
  activeAnnotationId: string | null,
) {
  const lower = pointData(timeline, (entry) =>
    layers.confidence ? entry.lowerValue : null,
  );
  const band = pointData(timeline, (entry) =>
    layers.confidence && entry.lowerValue != null && entry.upperValue != null
      ? entry.upperValue - entry.lowerValue
      : null,
  );
  const forecastZoneEnd = forecastZoneEndTimestamp(timeline);
  const forecastSeparator = layers.forecast ? separatorTimestamp : null;

  return [
    seriesLine(labels.confidence, lower, { opacity: 0 }, undefined, "confidence-90", false),
    { ...seriesLine(labels.confidence, band, { opacity: 0 }, palette.band90, "confidence-90", false), areaStyle: { color: palette.band90 } },
    {
      ...seriesLine(
        labels.history,
        pointData(timeline, (entry) => (layers.history ? entry.historyValue : null)),
        { color: palette.lineHistory, width: 2 },
        undefined,
        undefined,
        false
      ),
      areaStyle: { color: historyAreaGradient(palette) },
    },
    {
      ...seriesLine(
        labels.forecast,
        pointData(timeline, (entry) => (layers.forecast ? entry.forecastValue : null)),
        { color: palette.linePredict, width: 2, type: [5, 4] },
        undefined,
        undefined,
        true
      ),
      itemStyle: { color: palette.pointPredict, borderColor: palette.linePredict, borderWidth: 2 },
      markLine: forecastSeparator != null ? {
        symbol: ["none", "none"],
        silent: true,
        label: {
          show: true,
          formatter: labels.forecastStart,
          position: "insideEndTop" as const,
          rotate: 0,
          distance: 6,
          color: palette.inkMuted,
          fontSize: 11,
        },
        lineStyle: { color: palette.separator, type: "dashed" as const, width: 1 },
        data: [{ xAxis: forecastSeparator }],
      } : undefined,
      markArea: forecastSeparator != null && forecastZoneEnd != null ? {
        silent: true,
        label: { show: false },
        itemStyle: { color: palette.forecastZone },
        data: [
          [{ xAxis: forecastSeparator }, { xAxis: forecastZoneEnd }] as [
            { xAxis: number },
            { xAxis: number },
          ],
        ],
      } : undefined,
    },
    ...scenarios.map((scenario, index) =>
      seriesLine(
        scenario.name,
        pointData(timeline, (entry) => (
          layers[`scenario-${scenario.id}`]
            ? entry.scenarioValues.find((value) => value.id === scenario.id)?.value ?? null
            : null
        )),
        { color: scenarioColor(palette, index), width: 1.5, type: "dashed" },
        undefined,
        undefined,
        false
      )
    ),
    ...variables.map((variable, index) =>
      seriesLine(
        variable.name,
        pointData(timeline, (entry) =>
          layers[variable.id]
            ? entry.variableValues.find((value) => value.id === variable.id)?.value ?? null
            : null,
        ),
        { color: variableColor(palette, index), width: 1.2, type: "dotted" },
        undefined,
        undefined,
        false
      )
    ),
    ...buildAnnotationSeries(
      timeline,
      palette,
      labels,
      layers,
      activeAnnotationId,
    ),
  ];
}

function seriesLine(
  name: string,
  data: ChartPoint[],
  lineStyle: Record<string, unknown>,
  areaColor?: string,
  stack?: string,
  showSymbol?: boolean
) {
  return {
    name,
    type: "line" as const,
    data,
    stack,
    symbol: "circle" as const,
    showSymbol: Boolean(showSymbol),
    symbolSize: 5,
    smooth: FORECAST_CHART_LINE_SMOOTHING,
    smoothMonotone: FORECAST_CHART_LINE_MONOTONE_AXIS,
    connectNulls: false,
    lineStyle,
    areaStyle: areaColor ? { color: areaColor } : undefined,
    emphasis: { disabled: true },
  };
}

type ChartPoint = [number, number] | null;

function pointData(
  timeline: TimelineEntry[],
  valueForEntry: (entry: TimelineEntry) => number | null,
): ChartPoint[] {
  return timeline.map((entry) => {
    const value = valueForEntry(entry);
    return value == null ? null : [entry.timestamp, value];
  });
}

function scenarioColor(palette: ForecastChartPalette, index: number): string {
  return palette.scenarios[index % Math.max(palette.scenarios.length, 1)] || palette.lineHistory;
}

function variableColor(palette: ForecastChartPalette, index: number): string {
  return palette.variables[index % Math.max(palette.variables.length, 1)] || palette.inkMuted;
}

/** Matches the 2% right boundaryGap so the zone tint covers the padded edge. */
function forecastZoneEndTimestamp(timeline: TimelineEntry[]): number | null {
  const first = timeline[0]?.timestamp;
  const last = timeline[timeline.length - 1]?.timestamp;
  if (first == null || last == null || last <= first) return last ?? null;
  return last + (last - first) * 0.02;
}

function historyAreaGradient(palette: ForecastChartPalette) {
  return {
    type: "linear" as const,
    x: 0,
    y: 0,
    x2: 0,
    y2: 1,
    colorStops: [
      { offset: 0, color: palette.areaHistoryFrom },
      { offset: 1, color: palette.areaHistoryTo },
    ],
  };
}
