import { useMemo } from "react";
import type { ForecastChartProps } from "./forecast-chart-types";

/**
 * Memoizes the props that actually feed buildForecastChartOption so the
 * setOption effect skips re-renders where only zoom state or callbacks
 * changed. Without this, every wheel tick (zoom state -> re-render ->
 * new props identity) rebuilt the entire chart option.
 */
export function useForecastChartOptionInput(props: ForecastChartProps) {
  const {
    history,
    predictions,
    scenarios,
    variables,
    annotations,
    activeAnnotationId,
    mode,
    compact,
    quantiles,
    frequency,
    endDate,
    locale,
    targetColumn,
    fallbackName,
    labels,
    layers,
  } = props;
  return useMemo(
    () => ({
      history,
      predictions,
      scenarios,
      variables,
      annotations,
      activeAnnotationId,
      mode,
      compact,
      quantiles,
      frequency,
      endDate,
      locale,
      targetColumn,
      fallbackName,
      labels,
      layers,
    }),
    [
      history,
      predictions,
      scenarios,
      variables,
      annotations,
      activeAnnotationId,
      mode,
      compact,
      quantiles,
      frequency,
      endDate,
      locale,
      targetColumn,
      fallbackName,
      labels,
      layers,
    ],
  );
}
