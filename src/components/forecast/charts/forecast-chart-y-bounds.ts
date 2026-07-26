import type { ForecastChartOptionArgs, TimelineEntry } from "./forecast-chart-types";

const TARGET_TICKS = 5;

export function buildYAxisBounds(
  timeline: TimelineEntry[],
  layers: ForecastChartOptionArgs["layers"],
): { min: number; max: number } | null {
  const values: number[] = [];
  for (const entry of timeline) {
    if (entry.historyValue != null && layers.history) values.push(entry.historyValue);
    if (entry.forecastValue != null && layers.forecast) values.push(entry.forecastValue);
    if (layers.confidence) {
      if (entry.lowerValue != null) values.push(entry.lowerValue);
      if (entry.upperValue != null) values.push(entry.upperValue);
    }
    for (const scenario of entry.scenarioValues) values.push(scenario.value);
    for (const variable of entry.variableValues) values.push(variable.value);
  }
  if (!values.length) return null;
  let min = values[0];
  let max = values[0];
  for (const value of values) {
    if (value < min) min = value;
    if (value > max) max = value;
  }
  const span = max - min;
  const padding = span <= 0 ? Math.max(Math.abs(max) * 0.08, 1) : span * 0.12;
  return roundBoundsOutward(min - padding, max + padding);
}

/** Round bounds outward to clean steps so axis ticks land on round values. */
export function roundBoundsOutward(
  min: number,
  max: number,
): { min: number; max: number } {
  const span = max - min;
  const fallback = Math.max(Math.abs(max) * 0.04, 0.5);
  const step = niceStep(span > 0 ? span / TARGET_TICKS : fallback);
  return { min: snapDown(min, step), max: snapUp(max, step) };
}

/** Nice-number step: 1 / 2 / 2.5 / 5 × 10^n at or above the raw step. */
export function niceStep(rawStep: number): number {
  if (!Number.isFinite(rawStep) || rawStep <= 0) return 1;
  const magnitude = Math.pow(10, Math.floor(Math.log10(rawStep)));
  const normalized = rawStep / magnitude;
  const multiplier = normalized > 5
    ? 10
    : normalized > 2.5
      ? 5
      : normalized > 2
        ? 2.5
        : normalized > 1
          ? 2
          : 1;
  return multiplier * magnitude;
}

function snapDown(value: number, step: number): number {
  return snap(Math.floor(value / step) * step, step);
}

function snapUp(value: number, step: number): number {
  return snap(Math.ceil(value / step) * step, step);
}

function snap(value: number, step: number): number {
  const decimals = Math.max(0, -Math.floor(Math.log10(step)));
  return Number(value.toFixed(decimals + 2));
}
