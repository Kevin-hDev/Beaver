import type { TFunction } from "i18next";
import type { AnalysisCard } from "./forecast-analysis-types";

interface ForecastAnalysisCardGridProps {
  cards: AnalysisCard[];
  t: TFunction;
}

export function ForecastAnalysisCardGrid({ cards, t }: ForecastAnalysisCardGridProps) {
  return (
    <div className="fca-card-grid">
      {cards.map((card) => (
        <div key={card.labelKey} className={`fca-card ${card.tone ? `is-${card.tone}` : ""}`}>
          <span className="fca-card-label">{t(card.labelKey)}</span>
          <span className="fca-card-value">{card.value}</span>
        </div>
      ))}
    </div>
  );
}
