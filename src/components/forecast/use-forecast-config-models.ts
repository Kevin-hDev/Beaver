import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  isForecastModelSelectable,
  type ForecastModelEntry,
  type ForecastModelsResponse,
} from "./forecast-model-meta";

export function useForecastConfigModels(defaultModelId: string, selectFallbackModel = true) {
  const [models, setModels] = useState<ForecastModelEntry[]>([]);
  const [model, setModel] = useState(defaultModelId);

  useEffect(() => {
    invoke<ForecastModelsResponse>("list_forecast_models")
      .then((response) => {
        const visibleModels = response.models.filter(isForecastModelSelectable);
        setModels(visibleModels);
        if (defaultModelId && visibleModels.some((entry) => entry.id === defaultModelId)) {
          setModel(defaultModelId);
          return;
        }
        setModel(selectFallbackModel ? visibleModels[0]?.id ?? "" : "");
      })
      .catch(() => {});
  }, [defaultModelId, selectFallbackModel]);

  return { models, model, setModel };
}
