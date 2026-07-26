import { useCallback, useState } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import type { ForecastSection } from "@/hooks/use-forecast-panel";
import { ForecastHeader } from "./forecast-header";
import { ForecastEmpty } from "./forecast-empty";
import {
  buildForecastLayerGroups,
  createInitialLayerState,
  type ForecastLayerState,
} from "./forecast-layer-matrix";
import {
  ForecastViewFilters,
} from "./forecast-view-filters";
import { ExportDropdown } from "./widgets/export-dropdown";
import { ForecastModelSelector } from "./widgets/forecast-model-selector";
import { ForecastConfig, type LaunchConfig } from "./forecast-config";
import { loadForecastDraftFromFile, type ForecastDraftData } from "./forecast-data";
import { useForecastLayerSources } from "./use-forecast-layer-sources";
import { ForecastSectionRouter } from "./forecast-section-router";
import { useCurrentForecastAnalysisName } from "./use-current-forecast-analysis-name";
import { useForecastExport } from "./use-forecast-export";
import { useForecastSelectionPolicy } from "./model-selection/use-forecast-selection-policy";
import "./forecast-panel.css";

interface ForecastPanelProps {
  activeSection: ForecastSection;
  navOpen: boolean;
  currentAnalysisId: string | null;
  onSectionChange: (section: ForecastSection) => void;
  onToggleNav: () => void;
  onLoadAnalysis: (id: string) => void;
  onCloseAnalysis: () => void;
  onOpenWorkbench: () => void;
}

export function ForecastPanel({
  activeSection, navOpen, currentAnalysisId,
  onSectionChange, onToggleNav, onLoadAnalysis, onCloseAnalysis, onOpenWorkbench,
}: ForecastPanelProps) {
  const { t } = useTranslation();
  const hasAnalysis = currentAnalysisId !== null;
  const [draft, setDraft] = useState<ForecastDraftData | null>(null);
  const [launching, setLaunching] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [layers, setLayers] = useState<ForecastLayerState>(createInitialLayerState);
  const handleExport = useForecastExport();
  const {
    policy,
    selectedModelId,
    selectModel,
    setMode,
    setCloudAllowed,
    ready: selectedModelReady,
  } = useForecastSelectionPolicy();
  const currentAnalysisName = useCurrentForecastAnalysisName(currentAnalysisId);
  const layerSources = useForecastLayerSources(currentAnalysisId, setLayers);
  const filterGroups = buildForecastLayerGroups(
    layerSources.sources,
    t
  );

  const handleImportFile = async (path: string) => {
    setError(null);
    try {
      setDraft(await loadForecastDraftFromFile(path));
    } catch {
      setError(t("forecast.errors.importFailed"));
    }
  };

  const handleSelectModel = useCallback((modelId: string) => {
    selectModel(modelId);
  }, [selectModel]);

  const handleLaunch = async (config: LaunchConfig) => {
    if (!draft) return;
    setLaunching(true);
    setError(null);
    try {
      const result = await invoke<{ id: string }>("run_forecast", {
        request: {
          data: draft.dataJson,
          file_path: null,
          target_column: config.targetColumn,
          date_column: config.dateColumn,
          series_column: config.seriesColumn,
          covariate_columns: config.covariates,
          horizon: config.horizon,
          frequency: config.frequency,
          model: config.model,
          confidence_level: config.confidence,
        },
      });
      setDraft(null);
      onLoadAnalysis(result.id);
    } catch {
      setError(t("forecast.errors.launchFailed"));
    } finally {
      setLaunching(false);
    }
  };

  return (
    <div className="fc-panel">
      <ForecastHeader
        activeSection={activeSection}
        navOpen={navOpen}
        hasAnalysis={hasAnalysis}
        contextLabel={activeSection === "comparisons" ? currentAnalysisName : null}
        filterSlot={
          hasAnalysis && activeSection === "view" ? (
            <ForecastViewFilters
              groups={filterGroups}
              layers={layers}
              onChange={setLayers}
            />
          ) : null
        }
        rightSlot={null}
        onToggleNav={onToggleNav}
        onSectionChange={onSectionChange}
        onCloseAnalysis={onCloseAnalysis}
        onOpenWorkbench={onOpenWorkbench}
      />
      <div className="fc-body">
        {draft ? (
          <ForecastConfig
            draft={draft}
            launching={launching}
            error={error}
            defaultModelId={policy.mode === "manual" ? selectedModelId : ""}
            selectFallbackModel={policy.mode === "manual"}
            onModelChange={handleSelectModel}
            onLaunch={(config) => void handleLaunch(config)}
            onBack={() => setDraft(null)}
          />
        ) : !hasAnalysis ? (
          <ForecastEmpty
            error={error}
            onLoadAnalysis={onLoadAnalysis}
            onImportFile={(path) => void handleImportFile(path)}
          />
        ) : (
          <ForecastSectionRouter
            section={activeSection}
            analysisId={currentAnalysisId}
            layers={layers}
            onLoadAnalysis={onLoadAnalysis}
          />
        )}
      </div>
      {hasAnalysis && (
        <div className={`fc-footer ${activeSection === "view" ? "" : "fc-footer-end"}`}>
          {activeSection === "view" && (
            <ForecastModelSelector
              selectedModelId={selectedModelId}
              selectionMode={policy.mode}
              allowCloudInAuto={policy.allow_cloud_in_auto}
              selectionReady={selectedModelReady}
              onSelectModel={handleSelectModel}
              onModeChange={setMode}
              onCloudAllowedChange={setCloudAllowed}
            />
          )}
          <ExportDropdown
            analysisId={currentAnalysisId}
            onExport={handleExport}
          />
        </div>
      )}
    </div>
  );
}
