import { useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useLatestRequest } from "@/hooks/use-latest-request";
import {
  FORECAST_ANALYSIS_UPDATED,
  listenForecastAnalysisEvents,
} from "@/lib/forecast-analysis-events";
import type { ForecastScenarioAnalysis } from "./forecast-scenario-types";

interface UseForecastScenarioAnalysisArgs {
  analysisId: string;
  onLoaded: (analysis: ForecastScenarioAnalysis) => void;
  onFailed: () => void;
}

export function useForecastScenarioAnalysis({
  analysisId,
  onLoaded,
  onFailed,
}: UseForecastScenarioAnalysisArgs) {
  const runLatest = useLatestRequest();

  useEffect(() => {
    let active = true;
    const refresh = () => {
      void runLatest(
        () => invoke<ForecastScenarioAnalysis>("get_forecast_analysis", { id: analysisId }),
      )
        .then((analysis) => {
          if (active && analysis !== undefined) onLoaded(analysis);
        })
        .catch(() => {
          if (active) onFailed();
        });
    };
    refresh();
    const cleanup = listenForecastAnalysisEvents([FORECAST_ANALYSIS_UPDATED], (event) => {
      if (event.analysis_id === analysisId) refresh();
    });
    return () => {
      active = false;
      cleanup();
    };
  }, [analysisId, onFailed, onLoaded, runLatest]);
}
