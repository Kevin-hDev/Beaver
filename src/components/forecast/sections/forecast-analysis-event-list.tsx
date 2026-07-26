import type { TFunction } from "i18next";
import type { AnalysisEvent } from "./forecast-analysis-types";

interface ForecastAnalysisEventListProps {
  events: AnalysisEvent[];
  emptyKey: string;
  t: TFunction;
}

export function ForecastAnalysisEventList({ events, emptyKey, t }: ForecastAnalysisEventListProps) {
  if (!events.length) {
    return <p className="fca-empty-line">{t(emptyKey)}</p>;
  }
  return (
    <div className="fca-event-list">
      {events.map((event) => (
        <div key={event.id} className="fca-event">
          <span className={`fca-severity ${event.severity ? `is-${event.severity}` : ""}`} />
          <div className="fca-event-body">
            <span className="fca-event-label">{event.label}</span>
            <span className="fca-event-meta">{event.meta}</span>
          </div>
          <span className="fca-event-value">{event.value}</span>
        </div>
      ))}
    </div>
  );
}
