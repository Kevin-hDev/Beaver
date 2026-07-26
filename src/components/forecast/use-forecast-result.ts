import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useLatestRequest } from "@/hooks/use-latest-request";
import {
  FORECAST_ANALYSIS_UPDATED,
  listenForecastAnalysisEvents,
} from "@/lib/forecast-analysis-events";

export function useForecastResult<T>(analysisId: string, errorMessage: string) {
  const [result, setResult] = useState<{ analysisId: string; data: T } | null>(null);
  const [failure, setFailure] = useState<{ analysisId: string; message: string } | null>(null);
  const runLatest = useLatestRequest();
  const refresh = useCallback(async () => {
    try {
      const next = await runLatest(
        () => invoke<T>("get_forecast_analysis", { id: analysisId }),
      );
      if (next === undefined) return;
      setResult({ analysisId, data: next });
      setFailure(null);
    } catch {
      setFailure({ analysisId, message: errorMessage });
    }
  }, [analysisId, errorMessage, runLatest]);

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
