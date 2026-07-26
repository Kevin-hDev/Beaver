import { useCallback, useEffect, useMemo } from "react";
import { listen } from "@tauri-apps/api/event";
import type { useFilePreview } from "@/hooks/use-file-preview";
import type { useFileTree } from "@/hooks/use-file-tree";
import type { ForecastSection, PanelMode, useForecastPanel } from "@/hooks/use-forecast-panel";
import { cleanupTauriListener } from "@/lib/tauri-listen";
import {
  FORECAST_ANALYSIS_CREATED,
  type ForecastAnalysisEvent,
} from "@/lib/forecast-analysis-events";
import type { AgentLocalNavState, DeepPartial } from "@/types/navigation";

const MAX_FORECAST_ANALYSIS_ID_LENGTH = 128;

interface Args {
  navState: AgentLocalNavState;
  sessionId: string | null;
  filePreview: Pick<ReturnType<typeof useFilePreview>, "setOpen" | "setFullscreen">;
  fileTree: ReturnType<typeof useFileTree>;
  forecast: ReturnType<typeof useForecastPanel>;
  onNavChange?: (partial: DeepPartial<AgentLocalNavState>) => void;
}

export function useAgentLocalControlledPanels({
  navState, sessionId, filePreview, fileTree, forecast, onNavChange,
}: Args) {
  const toggleFileTree = useCallback(() => {
    const nextOpen = !navState.fileTreeOpen;
    fileTree.setOpen(nextOpen);
    onNavChange?.({ fileTreeOpen: nextOpen });
  }, [fileTree, navState.fileTreeOpen, onNavChange]);

  const fileTreeNav = useMemo(() => ({
    ...fileTree,
    open: navState.fileTreeOpen,
    toggleOpen: toggleFileTree,
    closeTree: () => {
      fileTree.closeTree();
      onNavChange?.({ fileTreeOpen: false });
    },
  }), [fileTree, navState.fileTreeOpen, onNavChange, toggleFileTree]);

  const setPanelMode = useCallback((mode: PanelMode) => {
    forecast.setPanelMode(mode);
    if (mode === "forecast" && navState.previewFullscreen) {
      filePreview.setFullscreen(false);
    }
    onNavChange?.({
      panelMode: mode,
      ...(mode === "forecast" ? { previewFullscreen: false } : {}),
    });
  }, [filePreview, forecast, navState.previewFullscreen, onNavChange]);

  const setSection = useCallback((section: ForecastSection) => {
    forecast.setSection(section);
    onNavChange?.({ forecastSection: section });
  }, [forecast, onNavChange]);

  const loadAnalysis = useCallback((id: string) => {
    forecast.loadAnalysis(id);
    filePreview.setOpen(true);
    if (navState.previewFullscreen) filePreview.setFullscreen(false);
    onNavChange?.({
      previewOpen: true,
      forecastAnalysisId: id,
      forecastSection: "view",
      panelMode: "forecast",
      previewFullscreen: false,
    });
  }, [filePreview, forecast, navState.previewFullscreen, onNavChange]);

  useEffect(() => {
    if (!sessionId) return;
    let cancelled = false;
    const unlisten = listen<ForecastAnalysisEvent>(FORECAST_ANALYSIS_CREATED, (event) => {
      const { analysis_id: analysisId, session_id: eventSessionId } = event.payload;
      const validAnalysisId = typeof analysisId === "string"
        && analysisId.length > 0
        && analysisId.length <= MAX_FORECAST_ANALYSIS_ID_LENGTH;
      if (cancelled || eventSessionId !== sessionId || !validAnalysisId) return;
      loadAnalysis(analysisId);
    });
    return () => {
      cancelled = true;
      cleanupTauriListener(unlisten);
    };
  }, [loadAnalysis, sessionId]);

  const focusAnalysis = useCallback((id: string) => {
    forecast.focusAnalysis(id);
    if (navState.previewFullscreen) filePreview.setFullscreen(false);
    onNavChange?.({ forecastAnalysisId: id, panelMode: "forecast", previewFullscreen: false });
  }, [filePreview, forecast, navState.previewFullscreen, onNavChange]);

  const closeAnalysis = useCallback(() => {
    forecast.closeAnalysis();
    onNavChange?.({ forecastAnalysisId: null });
  }, [forecast, onNavChange]);

  const forecastNav = useMemo(() => ({
    ...forecast,
    activeSection: navState.forecastSection,
    currentAnalysisId: navState.forecastAnalysisId,
    panelMode: navState.panelMode,
    setSection,
    loadAnalysis,
    focusAnalysis,
    closeAnalysis,
    setPanelMode,
  }), [
    closeAnalysis, focusAnalysis, forecast, loadAnalysis,
    navState.forecastAnalysisId, navState.forecastSection,
    navState.panelMode, setPanelMode, setSection,
  ]);

  return { fileTreeNav, forecastNav };
}
