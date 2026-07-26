export interface ForecastAnalysisMeta {
  id: string;
  name: string;
  created_at: string;
  model: string;
  horizon: number;
  mape: number | null;
}

const MAX_RECENT_ANALYSES = 5;

export function newestForecastAnalyses(
  analyses: ForecastAnalysisMeta[],
): ForecastAnalysisMeta[] {
  return [...analyses]
    .sort((left, right) => right.created_at.localeCompare(left.created_at))
    .slice(0, MAX_RECENT_ANALYSES);
}
