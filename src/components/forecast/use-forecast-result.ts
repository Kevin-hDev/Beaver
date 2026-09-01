import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useLatestRequest } from "@/hooks/use-latest-request";
import {
  FORECAST_ANALYSIS_UPDATED,
  listenForecastAnalysisEvents,
} from "@/lib/forecast-analysis-events";
import { useForecastSessionId } from "./forecast-workspace-context";

export function useForecastResult<T>(analysisId: string, errorMessage: string) {
  const sessionId = useForecastSessionId();
  const [result, setResult] = useState<{ analysisId: string; data: T } | null>(null);
  const [failure, setFailure] = useState<{ analysisId: string; message: string } | null>(null);
  const runLatest = useLatestRequest();
  const refresh = useCallback(async () => {
    try {
      if (!sessionId) throw new Error("missing_forecast_session");
      const next = await runLatest(
        () => invoke<T>("get_forecast_analysis", { sessionId, id: analysisId }),
      );
      if (next === undefined) return;
      setResult({ analysisId, data: next });
      setFailure(null);
    } catch {
      setFailure({ analysisId, message: errorMessage });
    }
  }, [analysisId, errorMessage, runLatest, sessionId]);

  useEffect(() => {
    // eslint-disable-next-line react-hooks/set-state-in-effect -- backend hydration is intentional
    void refresh();
    return listenForecastAnalysisEvents([FORECAST_ANALYSIS_UPDATED], (event) => {
      if (event.analysis_id === analysisId) void refresh();
    });
  }, [analysisId, refresh]);

  return {
    data: result?.analysisId === analysisId ? result.data : null,
    error: failure?.analysisId === analysisId ? failure.message : null,
  };
}
