import { useCallback, useEffect, useState, type Dispatch, type SetStateAction } from "react";
import { invoke } from "@tauri-apps/api/core";
import i18n from "@/i18n";
import { useLatestRequest } from "@/hooks/use-latest-request";
import {
  FORECAST_ANALYSIS_UPDATED,
  listenForecastAnalysisEvents,
} from "@/lib/forecast-analysis-events";
import type { ForecastLayerItem, ForecastLayerState } from "./forecast-layer-matrix";

interface ForecastLayerAnalysis {
  covariates_used?: string[];
  scenarios?: { id: string; name: string }[];
  ensemble?: object | null;
  annotations?: unknown[];
  advanced_analytics?: { anomalies?: unknown[] } | null;
  data_profile?: { issues?: Array<{ count?: number }> } | null;
}

interface ForecastLayerSources {
  scenarioLayers: ForecastLayerItem[];
  comparisonLayers: ForecastLayerItem[];
  covariateNames: string[];
  eventLayers: ForecastLayerItem[];
  anomalyLayers: ForecastLayerItem[];
  qualityLayers: ForecastLayerItem[];
}

const EMPTY_SOURCES: ForecastLayerSources = {
  scenarioLayers: [],
  comparisonLayers: [],
  covariateNames: [],
  eventLayers: [],
  anomalyLayers: [],
  qualityLayers: [],
};

export function useForecastLayerSources(
  analysisId: string | null,
  setLayers: Dispatch<SetStateAction<ForecastLayerState>>
) {
  const [sources, setSources] = useState<ForecastLayerSources>(EMPTY_SOURCES);
  const runLatest = useLatestRequest();

  const refresh = useCallback(async () => {
    try {
      const nextSources = await runLatest(() => loadSources(analysisId));
      if (nextSources === undefined) return;
      applySources(nextSources, setSources, setLayers);
    } catch {
      applySources(EMPTY_SOURCES, setSources, setLayers);
    }
  }, [analysisId, runLatest, setLayers]);

  useEffect(() => {
    void refresh();
    return listenForecastAnalysisEvents([FORECAST_ANALYSIS_UPDATED], (event) => {
      if (event.analysis_id === analysisId) void refresh();
    });
  }, [analysisId, refresh]);

  return { sources, refresh };
}

async function loadSources(analysisId: string | null): Promise<ForecastLayerSources> {
  if (!analysisId) return EMPTY_SOURCES;
  const analysis = await invoke<ForecastLayerAnalysis>("get_forecast_analysis", { id: analysisId });
  const scenarioLayers: ForecastLayerItem[] = (analysis.scenarios ?? []).map((scenario) => ({
    id: `scenario-${scenario.id}`,
    label: scenario.name,
    interactive: true,
  }));
  return {
    scenarioLayers,
    comparisonLayers: analysis.ensemble ? [{
      id: "scenario-ensemble",
      label: i18n.t("forecast.view.ensembleSeries"),
      interactive: true,
    }] : [],
    covariateNames: analysis.covariates_used ?? [],
    eventLayers: analysis.annotations?.length ? [{
      id: "annotations",
      label: i18n.t("forecast.view.filters.annotations"),
      interactive: true,
    }] : [],
    anomalyLayers: analysis.advanced_analytics?.anomalies?.length ? [{
      id: "anomalies",
      label: i18n.t("forecast.view.filters.residualAnomalies"),
      interactive: true,
    }] : [],
    qualityLayers: analysis.data_profile?.issues?.some((issue) => Number(issue.count) > 0) ? [{
      id: "quality",
      label: i18n.t("forecast.view.filters.dataQualityIssues"),
      interactive: true,
    }] : [],
  };
}

function applySources(
  sources: ForecastLayerSources,
  setSources: Dispatch<SetStateAction<ForecastLayerSources>>,
  setLayers: Dispatch<SetStateAction<ForecastLayerState>>
) {
  setSources(sources);
  setLayers((current) => {
    const next = { ...current };
    const enabledByDefault = [
      ...sources.scenarioLayers,
      ...sources.comparisonLayers,
      ...sources.eventLayers,
      ...sources.anomalyLayers,
      ...sources.qualityLayers,
    ];
    for (const layer of enabledByDefault) {
      if (next[layer.id] === undefined) next[layer.id] = true;
    }
    for (const name of sources.covariateNames) {
      const id = `variable-${name}`;
      if (next[id] === undefined) next[id] = false;
    }
    return next;
  });
}
