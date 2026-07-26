import type { EChartsOption } from "echarts";
import { inferMetricMeta, type ForecastMetricMeta } from "../forecast-view-format";
import { formatAxisDate } from "./forecast-chart-axis-date";
import { formatAxisValue, formatTooltip } from "./forecast-chart-format";
import { buildSeries } from "./forecast-chart-series";
import { buildTimeline } from "./forecast-chart-timeline";
import type { ForecastChartOptionArgs } from "./forecast-chart-types";
import { buildYAxisBounds } from "./forecast-chart-y-bounds";
import { FORECAST_CHART_MIN_ZOOM_SPAN } from "./forecast-chart-zoom-utils";

export function buildForecastChartOption(args: ForecastChartOptionArgs): EChartsOption {
  const metric = inferMetricMeta(args.locale, args.targetColumn, args.fallbackName);
  const timeline = buildTimeline(args);
  const separatorTimestamp = args.history.length > 0
    ? timeline[args.history.length - 1]?.timestamp ?? null
    : null;
  const yAxisBounds = buildYAxisBounds(timeline, args.layers);

  return {
    animation: false,
    grid: {
      left: args.compact ? 8 : 18,
      right: args.compact ? 8 : 18,
      top: args.compact ? 10 : 18,
      bottom: args.compact ? 18 : 38,
      containLabel: true,
    },
    tooltip: {
      trigger: "axis",
      backgroundColor: args.palette.tooltipBg,
      borderColor: args.palette.edge,
      borderWidth: 1,
      padding: [7, 10],
      textStyle: { color: args.palette.tooltipText, fontFamily: "var(--font-sans)", fontSize: 12, lineHeight: 17 },
      formatter: (raw) => formatTooltip(raw, timeline, metric, args.labels, args.locale),
      confine: true,
      position: tooltipPosition,
    },
    xAxis: buildXAxis(args.locale, args.palette.edge, args.palette.inkMuted, args.chartWidth),
    yAxis: buildYAxis(
      args.locale,
      metric,
      args.palette.edge,
      args.palette.inkMuted,
      yAxisBounds,
    ),
    dataZoom: [
      {
        type: "inside",
        xAxisIndex: 0,
        start: args.zoomWindow.start,
        end: args.zoomWindow.end,
        minSpan: FORECAST_CHART_MIN_ZOOM_SPAN,
        realtime: true,
        throttle: 16,
        filterMode: "none",
        // Wheel zoom is owned by use-forecast-wheel-zoom (ECharts roam wheel
        // anchors zoom at the cursor and drifts; we compute the window).
        zoomOnMouseWheel: false,
        moveOnMouseWheel: false,
        moveOnMouseMove: false,
      },
    ],
    series: buildSeries(
      timeline,
      separatorTimestamp,
      args.palette,
      args.scenarios,
      args.variables,
      args.labels,
      args.layers,
      args.activeAnnotationId ?? null,
    ),
  };
}

function buildXAxis(locale: string, edge: string, inkMuted: string, chartWidth: number) {
  return {
    type: "time" as const,
    boundaryGap: ["0%", "2%"] as [string, string],
    splitNumber: forecastXAxisSplitNumber(chartWidth),
    axisLine: { lineStyle: { color: edge } },
    axisTick: { show: false },
    axisLabel: {
      color: inkMuted,
      fontSize: 11,
      hideOverlap: true,
      margin: 10,
      formatter: (value: number) => formatAxisDate(value, locale),
    },
  };
}

export function forecastXAxisSplitNumber(chartWidth: number): number {
  if (!Number.isFinite(chartWidth) || chartWidth <= 0) return 4;
  return Math.max(3, Math.min(8, Math.floor(chartWidth / 135)));
}

function tooltipPosition(
  point: number[],
  _params: unknown,
  _dom: unknown,
  _rect: unknown,
  size: { contentSize: number[]; viewSize: number[] },
): [number, number] {
  const gap = 14;
  const margin = 8;
  const [pointerX, pointerY] = point;
  const [tooltipWidth, tooltipHeight] = size.contentSize;
  const [viewWidth, viewHeight] = size.viewSize;
  const preferredLeft = pointerX > viewWidth / 2
    ? pointerX - tooltipWidth - gap
    : pointerX + gap;
  const left = clamp(preferredLeft, margin, viewWidth - tooltipWidth - margin);
  const preferredTop = pointerY - tooltipHeight - gap;
  const fallbackTop = pointerY + gap;
  const top = preferredTop >= margin ? preferredTop : fallbackTop;
  return [left, clamp(top, margin, viewHeight - tooltipHeight - margin)];
}

function clamp(value: number, min: number, max: number): number {
  if (max < min) return min;
  if (value < min) return min;
  if (value > max) return max;
  return value;
}

function buildYAxis(
  locale: string,
  metric: ForecastMetricMeta,
  edge: string,
  inkMuted: string,
  bounds: { min: number; max: number } | null,
) {
  return {
    type: "value" as const,
    scale: true,
    min: bounds?.min,
    max: bounds?.max,
    splitLine: { lineStyle: { color: edge, opacity: 0.5 } },
    axisLine: { show: false },
    axisTick: { show: false },
    axisLabel: {
      color: inkMuted,
      fontSize: 11,
      formatter: (value: number) => formatAxisValue(value, locale, metric),
    },
  };
}
