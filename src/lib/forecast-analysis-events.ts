import { listen } from "@tauri-apps/api/event";
import { cleanupTauriListener } from "@/lib/tauri-listen";

export interface ForecastAnalysisEvent {
  analysis_id: string;
  session_id?: string | null;
  revision?: number | null;
}

export const FORECAST_ANALYSIS_UPDATED = "forecast-analysis-updated";
export const FORECAST_ANALYSIS_CREATED = "forecast-analysis-created";
export const FORECAST_ANALYSIS_DELETED = "forecast-analysis-deleted";

type ForecastAnalysisEventName =
  | typeof FORECAST_ANALYSIS_UPDATED
  | typeof FORECAST_ANALYSIS_CREATED
  | typeof FORECAST_ANALYSIS_DELETED;

export function listenForecastAnalysisEvents(
  names: ForecastAnalysisEventName[],
  handler: (event: ForecastAnalysisEvent) => void,
) {
  const listeners = names.map((name) => (
    listen<ForecastAnalysisEvent>(name, (event) => handler(event.payload))
  ));
  return () => {
    for (const listener of listeners) cleanupTauriListener(listener);
  };
}
