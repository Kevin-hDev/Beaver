import { useMemo } from "react";
import { useTranslation } from "react-i18next";
import type { ForecastChartOptionArgs } from "./forecast-chart-types";

/** Memoized chart series labels shared by the main and fan charts. */
export function useForecastChartLabels(): ForecastChartOptionArgs["labels"] {
  const { t } = useTranslation();
  return useMemo(
    () => ({
      history: t("forecast.view.historySeries"),
      forecast: t("forecast.view.forecastSeries"),
      confidence: t("forecast.view.confidenceRange"),
      forecastStart: t("forecast.chart.forecastStart"),
      annotationUser: t("forecast.notes.userSource"),
      annotationLlm: t("forecast.notes.llmSource"),
    }),
    [t],
  );
}
