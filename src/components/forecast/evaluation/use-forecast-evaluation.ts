import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useLatestRequest } from "@/hooks/use-latest-request";
import {
  FORECAST_ANALYSIS_UPDATED,
  listenForecastAnalysisEvents,
} from "@/lib/forecast-analysis-events";
import { preferNewestForecast } from "../forecast-revision";
import type { EvaluationAnalysis } from "./forecast-evaluation-types";

export function useForecastEvaluation(analysisId: string) {
  const [analysis, setAnalysis] = useState<EvaluationAnalysis | null>(null);
  const [loading, setLoading] = useState(true);
  const [running, setRunning] = useState(false);
  const [loadFailed, setLoadFailed] = useState(false);
  const [runFailed, setRunFailed] = useState(false);
  const [ensembleRunning, setEnsembleRunning] = useState(false);
  const [ensembleFailed, setEnsembleFailed] = useState(false);
  const runLatest = useLatestRequest();

  const refresh = useCallback(async () => {
    try {
      const next = await runLatest(
        () => invoke<EvaluationAnalysis>("get_forecast_analysis", { id: analysisId }),
      );
      if (next === undefined) return;
      setAnalysis((current) => preferNewestForecast(current, next));
      setLoadFailed(false);
    } catch {
      setLoadFailed(true);
    } finally {
      setLoading(false);
    }
  }, [analysisId, runLatest]);

  const createEnsemble = useCallback(async () => {
    setEnsembleFailed(false);
    setEnsembleRunning(true);
    try {
      const next = await invoke<EvaluationAnalysis>("create_forecast_ensemble", {
        analysisId,
        modelIds: [],
      });
      setAnalysis((current) => preferNewestForecast(current, next));
    } catch {
      setEnsembleFailed(true);
    } finally {
      setEnsembleRunning(false);
    }
  }, [analysisId]);

  const run = useCallback(async () => {
    setRunFailed(false);
    setRunning(true);
    try {
      const next = await invoke<EvaluationAnalysis>("run_forecast_backtest", {
        request: { analysis_id: analysisId, model_ids: [], max_windows: 3 },
      });
      setAnalysis((current) => preferNewestForecast(current, next));
      setRunFailed(false);
    } catch {
      setRunFailed(true);
    } finally {
      setRunning(false);
    }
  }, [analysisId]);

  useEffect(() => {
    // eslint-disable-next-line react-hooks/set-state-in-effect -- backend hydration is intentional
    void refresh();
    return listenForecastAnalysisEvents([FORECAST_ANALYSIS_UPDATED], (event) => {
      if (event.analysis_id === analysisId) void refresh();
    });
  }, [analysisId, refresh]);

  return {
    analysis,
    loading,
    running,
    loadFailed,
    runFailed,
    ensembleRunning,
    ensembleFailed,
    run,
    createEnsemble,
  };
}
