import { useCallback, useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useTranslation } from "react-i18next";
import { cleanupTauriListener } from "@/lib/tauri-listen";
import {
  isForecastModelConfigurable,
  listForecastFamilies,
  type ForecastModelEntry,
  type ForecastModelsResponse,
  type ForecastProviderEntry,
} from "../forecast/forecast-model-meta";
import { ForecastConfigView } from "./forecast-settings-config";
import { ForecastModelsView } from "./forecast-settings-models";
import type { DeepPartial, SettingsNavState } from "@/types/navigation";
import "./forecast-settings.css";

interface ForecastSettingsProps {
  navState: SettingsNavState;
  onNavChange: (partial: DeepPartial<SettingsNavState>) => void;
  onNavReplace: (partial: DeepPartial<SettingsNavState>) => void;
}

export function ForecastSettings({ navState, onNavChange, onNavReplace }: ForecastSettingsProps) {
  const { t } = useTranslation();
  const subTab = navState.forecastSubTab;
  const [models, setModels] = useState<ForecastModelEntry[]>([]);
  const [providers, setProviders] = useState<ForecastProviderEntry[]>([]);

  const refresh = useCallback(() => {
    void invoke<ForecastModelsResponse>("list_forecast_models")
      .then((result) => {
        setModels(result.models);
        setProviders(result.providers);
      })
      .catch(() => {});
  }, []);

  useEffect(() => {
    refresh();
    const unlistenModels = listen("forecast-models-changed", refresh);
    const unlistenProviders = listen("providers-changed", refresh);
    const unlistenFsProviders = listen("fs:providers-changed", refresh);
    const unlistenFsConfig = listen("fs:config-changed", refresh);
    return () => {
      cleanupTauriListener(unlistenModels);
      cleanupTauriListener(unlistenProviders);
      cleanupTauriListener(unlistenFsProviders);
      cleanupTauriListener(unlistenFsConfig);
    };
  }, [refresh]);

  const families = useMemo(() => listForecastFamilies(models), [models]);
  const configModels = useMemo(() => models.filter(isForecastModelConfigurable), [models]);
  const selectedConfigModel = useMemo(
    () => configModels.find((model) => model.id === navState.forecastConfigModelId)
      ?? configModels[0]
      ?? null,
    [configModels, navState.forecastConfigModelId],
  );
  const selectedFamily = useMemo(
    () => families.find((group) => group.id === navState.forecastFamilyId) ?? null,
    [families, navState.forecastFamilyId],
  );
  const selectedModel = useMemo(
    () => models.find((model) => model.id === navState.forecastModelId) ?? null,
    [models, navState.forecastModelId],
  );

  useEffect(() => {
    if (navState.forecastConfigModelId || configModels.length === 0) return;
    void invoke<string | null>("get_selected_forecast_model")
      .then((selected) => {
        const next = configModels.find((model) => model.id === selected)?.id
          ?? configModels[0]?.id
          ?? null;
        onNavReplace({ forecastConfigModelId: next });
      })
      .catch(() => onNavReplace({ forecastConfigModelId: configModels[0]?.id ?? null }));
  }, [configModels, navState.forecastConfigModelId, onNavReplace]);

  return (
    <div className="fs-page">
      <div className="fs-inner">
        <h2 className="fs-title">{t("forecast.title")}</h2>

        <div className="fs-subtabs">
          <button
            className={`ollama-subtab ${subTab === "config" ? "active" : ""}`}
            onClick={() => onNavChange({ forecastSubTab: "config" })}
          >
            {t("forecast.modelConfig.sidebarTitle")}
          </button>
          <button
            className={`ollama-subtab ${subTab === "models" ? "active" : ""}`}
            onClick={() => onNavChange({ forecastSubTab: "models" })}
          >
            {t("forecast.models.sidebarTitle")}
          </button>
        </div>

        {subTab === "config" ? (
          <ForecastConfigView
            models={configModels}
            selectedModel={selectedConfigModel}
            onSelectModel={(id) => onNavChange({ forecastConfigModelId: id })}
            onModelsChanged={refresh}
          />
        ) : (
          <ForecastModelsView
            families={families}
            providers={providers}
            selectedFamily={selectedFamily}
            selectedModel={selectedModel}
            onSelectFamily={(id) => onNavChange({ forecastFamilyId: id, forecastModelId: null })}
            onSelectModel={(id) => onNavChange({ forecastModelId: id })}
            onBackToFamily={() => onNavChange({ forecastModelId: null })}
            onBackToFamilies={() => onNavChange({ forecastFamilyId: null, forecastModelId: null })}
            onRefresh={refresh}
          />
        )}
      </div>
    </div>
  );
}
