import type { TFunction } from "i18next";
import type { ForecastComparisonStats } from "./forecast-comparison-types";

interface ForecastComparisonSummaryProps {
  stats: ForecastComparisonStats | null;
  t: TFunction;
  formatValue: (value: number) => string;
}

export function ForecastComparisonSummary({
  stats,
  t,
  formatValue,
}: ForecastComparisonSummaryProps) {
  const cards = [
    {
      label: t("forecast.comparisons.averageDelta"),
      value: stats ? formatValue(stats.averageDelta) : "—",
    },
    {
      label: t("forecast.comparisons.maxDelta"),
      value: stats ? formatValue(stats.maxDelta) : "—",
    },
    {
      label: t("forecast.comparisons.averageDeltaPct"),
      value: stats ? `${stats.averageDeltaPercent.toFixed(1)}%` : "—",
    },
    {
      label: t("forecast.comparisons.direction"),
      value: stats ? t(`forecast.comparisons.${stats.direction}`) : "—",
    },
  ];

  return (
    <div className="fccmp-summary">
      {cards.map((card) => (
        <div key={card.label} className="fccmp-card">
          <span className="fccmp-card-label">{card.label}</span>
          <span className="fccmp-card-value">{card.value}</span>
        </div>
      ))}
    </div>
  );
}
