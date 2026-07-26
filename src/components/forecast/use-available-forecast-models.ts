import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { cleanupTauriListener } from "@/lib/tauri-listen";
import {
  isForecastModelVisibleInSelector,
  type ForecastModelEntry,
  type ForecastModelsResponse,
} from "./forecast-model-meta";

let cachedForecastModels: ForecastModelEntry[] = [];

async function fetchForecastModels(): Promise<ForecastModelEntry[]> {
  const response = await invoke<ForecastModelsResponse>("list_forecast_models");
  cachedForecastModels = response.models.filter(isForecastModelVisibleInSelector);
  return cachedForecastModels;
}

export function useAvailableForecastModels() {
  const [models, setModels] = useState<ForecastModelEntry[]>(cachedForecastModels);
  const [loading, setLoading] = useState(cachedForecastModels.length === 0);

  const refresh = useCallback(async () => {
    try {
      setModels(await fetchForecastModels());
    } catch {
      // Keep the last known list to avoid UI flickering during transient refresh failures.
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    // eslint-disable-next-line react-hooks/set-state-in-effect -- fetch→setState is intentional
    void refresh();
    const unlistenForecast = listen("forecast-models-changed", () => void refresh());
    const unlistenProviders = listen("providers-changed", () => void refresh());
    const unlistenFsProviders = listen("fs:providers-changed", () => void refresh());
    const unlistenFsConfig = listen("fs:config-changed", () => void refresh());
    return () => {
      cleanupTauriListener(unlistenForecast);
      cleanupTauriListener(unlistenProviders);
      cleanupTauriListener(unlistenFsProviders);
      cleanupTauriListener(unlistenFsConfig);
    };
  }, [refresh]);

  return { models, loading, refresh };
}
