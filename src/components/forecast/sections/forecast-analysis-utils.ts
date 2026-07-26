import { buildPeriodMeta, formatForecastValue, type ForecastMetricMeta } from "../forecast-view-format";
import type { AnalysisCard, AnalysisEvent, ForecastAnalysisData, ForecastAnalysisPoint } from "./forecast-analysis-types";

export function filterAnalysisPoints(points: ForecastAnalysisPoint[], seriesId: string) {
  if (!seriesId) return points;
  return points.filter((point) => point.series_id == null || point.series_id === seriesId);
}

export function filterAnalysisQuantiles(
  points: ForecastAnalysisPoint[],
  seriesId: string,
  quantiles: ForecastAnalysisData["quantiles"],
) {
  if (!seriesId) return quantiles;
  const indices = points
    .map((point, index) => (point.series_id == null || point.series_id === seriesId ? index : -1))
    .filter((index) => index >= 0);
  return {
    q10: indices.map((index) => quantiles.q10[index]).filter((value) => value !== undefined),
    q90: indices.map((index) => quantiles.q90[index]).filter((value) => value !== undefined),
  };
}

export function buildTrendCards(
  predictions: ForecastAnalysisPoint[],
  locale: string,
  metric: ForecastMetricMeta,
  t: (key: string) => string,
): AnalysisCard[] {
  const first = predictions[0]?.value ?? 0;
  const last = predictions[predictions.length - 1]?.value ?? first;
  const delta = last - first;
  const percent = first === 0 ? 0 : (delta / first) * 100;
  return [
    { labelKey: "forecast.analysis.direction", value: t(directionKey(percent)), tone: Math.abs(percent) < 2 ? "neutral" : percent > 0 ? "good" : "warn" },
    { labelKey: "forecast.analysis.totalChange", value: `${formatForecastValue(delta, locale, metric)} · ${percent.toFixed(1)}%` },
    { labelKey: "forecast.analysis.startValue", value: formatForecastValue(first, locale, metric) },
    { labelKey: "forecast.analysis.endValue", value: formatForecastValue(last, locale, metric) },
  ];
}

export function buildUncertaintyCards(
  quantiles: ForecastAnalysisData["quantiles"],
  predictions: ForecastAnalysisPoint[],
  locale: string,
  metric: ForecastMetricMeta,
): AnalysisCard[] {
  const widths = quantiles.q90.map((upper, index) => Math.max(0, upper - (quantiles.q10[index] ?? upper)));
  const average = averageOf(widths);
  const max = Math.max(...widths, 0);
  const maxIndex = widths.findIndex((width) => width === max);
  const period = predictions[maxIndex]?.date ?? "";
  return [
    { labelKey: "forecast.analysis.averageRange", value: formatForecastValue(average, locale, metric) },
    { labelKey: "forecast.analysis.maxRange", value: formatForecastValue(max, locale, metric), tone: max > average * 1.5 ? "warn" : "neutral" },
    { labelKey: "forecast.analysis.maxRangePeriod", value: period || "—" },
  ];
}

export function buildHighlightEvents(
  predictions: ForecastAnalysisPoint[],
  endDate: string,
  frequency: string,
  locale: string,
  metric: ForecastMetricMeta,
  t: (key: string) => string,
): AnalysisEvent[] {
  if (!predictions.length) return [];
  const sorted = [...predictions].map((point, index) => ({ point, index }));
  const high = sorted.reduce((best, item) => item.point.value > best.point.value ? item : best, sorted[0]);
  const low = sorted.reduce((best, item) => item.point.value < best.point.value ? item : best, sorted[0]);
  const moves = predictions.slice(1).map((point, index) => ({ index: index + 1, delta: point.value - predictions[index].value }));
  const up = moves.reduce((best, item) => item.delta > best.delta ? item : best, { index: 0, delta: 0 });
  const down = moves.reduce((best, item) => item.delta < best.delta ? item : best, { index: 0, delta: 0 });
  return [
    event("high", t("forecast.analysis.highestPoint"), high.index, high.point.value, predictions, endDate, frequency, locale, metric),
    event("low", t("forecast.analysis.lowestPoint"), low.index, low.point.value, predictions, endDate, frequency, locale, metric),
    event("up", t("forecast.analysis.strongestRise"), up.index, up.delta, predictions, endDate, frequency, locale, metric),
    event("down", t("forecast.analysis.strongestDrop"), down.index, down.delta, predictions, endDate, frequency, locale, metric),
  ];
}

function event(id: string, label: string, index: number, value: number, predictions: ForecastAnalysisPoint[], endDate: string, frequency: string, locale: string, metric: ForecastMetricMeta): AnalysisEvent {
  const point = predictions[index] ?? predictions[0];
  const period = buildPeriodMeta(index, point.date, endDate, frequency, locale);
  return { id, label, value: formatForecastValue(value, locale, metric), meta: period.secondaryLabel };
}

function directionKey(percent: number) {
  if (percent > 2) return "forecast.analysis.rising";
  if (percent < -2) return "forecast.analysis.falling";
  return "forecast.analysis.stable";
}

function averageOf(values: number[]) {
  return values.length ? values.reduce((sum, value) => sum + value, 0) / values.length : 0;
}
