import { formatForecastValue, type ForecastMetricMeta } from "../forecast-view-format";
import type {
  AnalysisCard,
  AnalysisEvent,
  ForecastAdvancedAnalytics,
  ForecastDecomposition,
  ForecastDriftReport,
  VariableInsight,
} from "./forecast-analysis-types";

type Translate = (key: string, values?: Record<string, unknown>) => string;

export function buildResidualAnomalyEvents(
  analytics: ForecastAdvancedAnalytics | null | undefined,
  seriesId: string,
  locale: string,
  metric: ForecastMetricMeta,
  t: Translate,
): AnalysisEvent[] {
  return (analytics?.anomalies ?? [])
    .filter((item) => matchesSeries(item.series_id, seriesId))
    .slice(0, 8)
    .map((item) => ({
      id: item.id,
      label: t("forecast.analysis.residualAnomaly"),
      value: formatForecastValue(item.observed, locale, metric),
      meta: t("forecast.analysis.residualAnomalyMeta", {
        date: item.date,
        score: item.score.toFixed(1),
      }),
      severity: item.severity,
    }));
}

export function buildAdvancedVariableInsights(
  analytics: ForecastAdvancedAnalytics | null | undefined,
  t: Translate,
): VariableInsight[] {
  const report = analytics?.variable_importance;
  if (!report || report.status !== "ready") return [];
  return report.items.slice(0, 8).map((item) => ({
    name: item.name,
    score: item.normalized_score,
    detail: t("forecast.analysis.variableImportanceValue", {
      value: Math.round(item.normalized_score * 100),
      direction: t(`forecast.analysis.variableDirection.${item.direction}`),
    }),
  }));
}

export function buildDecompositionCards(
  analytics: ForecastAdvancedAnalytics | null | undefined,
  seriesId: string,
  t: Translate,
): AnalysisCard[] {
  const report = findSeries(analytics?.decomposition, seriesId);
  if (!report || report.status !== "ready") return [];
  return [
    {
      labelKey: "forecast.analysis.decompositionMethod",
      value: t(`forecast.analysis.methods.${report.method}`),
    },
    {
      labelKey: "forecast.analysis.seasonalPeriod",
      value: report.period > 1 ? String(report.period) : t("forecast.analysis.notDetected"),
    },
    {
      labelKey: "forecast.analysis.seasonalStrength",
      value: report.seasonal_strength == null
        ? t("forecast.analysis.notAvailable")
        : `${Math.round(report.seasonal_strength * 100)}%`,
    },
  ];
}

export function buildDriftCards(
  analytics: ForecastAdvancedAnalytics | null | undefined,
  seriesId: string,
  t: Translate,
): AnalysisCard[] {
  const report = findSeries(analytics?.drift, seriesId);
  if (!report || report.status !== "ready") return [];
  return [
    {
      labelKey: "forecast.analysis.driftStatus",
      value: report.detected ? t("forecast.analysis.driftDetected") : t("forecast.analysis.driftStable"),
      tone: report.detected ? "warn" : "good",
    },
    {
      labelKey: "forecast.analysis.driftScore",
      value: report.score?.toFixed(2) ?? t("forecast.analysis.notAvailable"),
    },
    {
      labelKey: "forecast.analysis.distributionShift",
      value: report.distribution_shift == null
        ? t("forecast.analysis.notAvailable")
        : `${Math.round(report.distribution_shift * 100)}%`,
    },
  ];
}

function matchesSeries(value: string | null | undefined, selected: string): boolean {
  return !selected ? value == null : value === selected;
}

function findSeries<T extends ForecastDecomposition | ForecastDriftReport>(
  reports: T[] | undefined,
  selected: string,
): T | undefined {
  return reports?.find((report) => matchesSeries(report.series_id, selected));
}
