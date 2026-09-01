import { useCallback, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { showToast } from "@/lib/toast-emitter";
import { useLatestRequest } from "@/hooks/use-latest-request";
import {
  FORECAST_ANALYSIS_UPDATED,
  listenForecastAnalysisEvents,
} from "@/lib/forecast-analysis-events";
import { preferNewestForecast } from "../forecast-revision";
import type { EvaluationAnalysis } from "./forecast-evaluation-types";
import { useForecastSessionId } from "../forecast-workspace-context";

export function useForecastEvaluation(analysisId: string) {
  const sessionId = useForecastSessionId();
  const [analysis, setAnalysis] = useState<EvaluationAnalysis | null>(null);
  const [loading, setLoading] = useState(true);
  const [running, setRunning] = useState(false);
  const [loadFailed, setLoadFailed] = useState(false);
  const [ensembleRunning, setEnsembleRunning] = useState(false);
  const runLatest = useLatestRequest();
  const { t } = useTranslation();

  const refresh = useCallback(async () => {
    try {
      if (!sessionId) throw new Error("missing_forecast_session");
      const next = await runLatest(
        () => invoke<EvaluationAnalysis>("get_forecast_analysis", { sessionId, id: analysisId }),
      );
      if (next === undefined) return;
      setAnalysis((current) => preferNewestForecast(current, next));
      setLoadFailed(false);
    } catch {
      setLoadFailed(true);
    } finally {
      setLoading(false);
    }
  }, [analysisId, runLatest, sessionId]);

  const createEnsemble = useCallback(async () => {
    setEnsembleRunning(true);
    try {
      if (!sessionId) throw new Error("missing_forecast_session");
      const next = await invoke<EvaluationAnalysis>("create_forecast_ensemble", {
        sessionId,
        analysisId,
        modelIds: [],
      });
      setAnalysis((current) => preferNewestForecast(current, next));
    } catch {
      showToast(t("forecast.workbench.evaluation.ensembleFailed"), "error");
    } finally {
      setEnsembleRunning(false);
    }
  }, [analysisId, sessionId, t]);

  const run = useCallback(async () => {
    setRunning(true);
    try {
      if (!sessionId) throw new Error("missing_forecast_session");
      const next = await invoke<EvaluationAnalysis>("run_forecast_backtest", {
        sessionId,
        request: { analysis_id: analysisId, model_ids: [], max_windows: 3 },
      });
      setAnalysis((current) => preferNewestForecast(current, next));
    } catch {
      showToast(t("forecast.workbench.evaluation.runFailed"), "error");
    } finally {
      setRunning(false);
    }
  }, [analysisId, sessionId, t]);

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
    ensembleRunning,
    run,
    createEnsemble,
  };
}
