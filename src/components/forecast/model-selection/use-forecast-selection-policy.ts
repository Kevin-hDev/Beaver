import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import i18n from "@/i18n";
import { useLatestRequest } from "@/hooks/use-latest-request";
import { showToast } from "@/lib/toast-emitter";
import { cleanupTauriListener } from "@/lib/tauri-listen";
import {
  DEFAULT_FORECAST_SELECTION_POLICY,
  isForecastSelectionPolicy,
  type ForecastSelectionMode,
  type ForecastSelectionPolicy,
} from "./forecast-selection-types";

export function useForecastSelectionPolicy() {
  const [policy, setPolicy] = useState<ForecastSelectionPolicy>(
    DEFAULT_FORECAST_SELECTION_POLICY,
  );
  const [ready, setReady] = useState(false);
  const runLatest = useLatestRequest();

  const persist = useCallback((
    request: () => Promise<ForecastSelectionPolicy>,
  ) => {
    void runLatest(request)
      .then((next) => {
        if (next === undefined) return;
        if (!isForecastSelectionPolicy(next)) throw new Error("invalid-forecast-policy");
        setPolicy(next);
      })
      .catch(() => showToast(i18n.t("forecast.selection.saveFailed")));
  }, [runLatest]);

  const selectModel = useCallback((modelId: string) => {
    persist(() => (
      invoke<ForecastSelectionPolicy>("set_selected_forecast_model", { name: modelId })
    ));
  }, [persist]);

  const setMode = useCallback((mode: ForecastSelectionMode) => {
    persist(() => (
      invoke<ForecastSelectionPolicy>("set_forecast_selection_mode", { mode })
    ));
  }, [persist]);

  const setCloudAllowed = useCallback((allowed: boolean) => {
    persist(() => (
      invoke<ForecastSelectionPolicy>("set_forecast_auto_cloud_allowed", { allowed })
    ));
  }, [persist]);

  useEffect(() => {
    let mounted = true;
    void runLatest(
      () => invoke<ForecastSelectionPolicy>("get_forecast_selection_policy"),
    )
      .then((next) => {
        if (next === undefined) return;
        if (!isForecastSelectionPolicy(next)) throw new Error("invalid-forecast-policy");
        if (mounted) setPolicy(next);
      })
      .catch(() => showToast(i18n.t("forecast.selection.loadFailed")))
      .finally(() => {
        if (mounted) setReady(true);
      });
    const unlisten = listen<ForecastSelectionPolicy>(
      "forecast-selection-policy-changed",
      (event) => {
        if (!isForecastSelectionPolicy(event.payload)) return;
        void runLatest(() => Promise.resolve(event.payload))
          .then((next) => {
            if (mounted && next !== undefined) setPolicy(next);
          });
      },
    );
    return () => {
      mounted = false;
      cleanupTauriListener(unlisten);
    };
  }, [runLatest]);

  return {
    policy,
    selectedModelId: policy.manual_model_id ?? "",
    selectModel,
    setMode,
    setCloudAllowed,
    ready,
  };
}
