import { useCallback } from "react";
import type { AgentLocalWorkspaceState } from "@/types/navigation";
import type { ForecastSection, PanelMode } from "@/types/forecast-panel";

export type { ForecastPanelState, ForecastSection, PanelMode } from "@/types/forecast-panel";
export {
  MAX_FORECAST_ANALYSIS_ID_LENGTH,
  normalizeForecastPanelState,
  normalizeForecastSection,
} from "@/types/forecast-panel";

type ForecastWorkspaceState = Pick<
  AgentLocalWorkspaceState,
  "forecastSection" | "forecastNavOpen" | "forecastAnalysisId" | "panelMode"
>;

type ForecastWorkspacePatch = Partial<ForecastWorkspaceState>;

export function useForecastPanel(
  workspace: ForecastWorkspaceState,
  onWorkspaceChange?: (partial: ForecastWorkspacePatch) => void,
) {
  const setSection = useCallback((section: ForecastSection) => {
    onWorkspaceChange?.({ forecastSection: section, forecastNavOpen: false });
  }, [onWorkspaceChange]);

  const toggleNav = useCallback(() => {
    onWorkspaceChange?.({ forecastNavOpen: !workspace.forecastNavOpen });
  }, [onWorkspaceChange, workspace.forecastNavOpen]);

  const loadAnalysis = useCallback((id: string) => {
    onWorkspaceChange?.({
      forecastAnalysisId: id,
      forecastSection: "view",
      panelMode: "forecast",
    });
  }, [onWorkspaceChange]);

  const focusAnalysis = useCallback((id: string) => {
    onWorkspaceChange?.({ forecastAnalysisId: id, panelMode: "forecast" });
  }, [onWorkspaceChange]);

  const closeAnalysis = useCallback(() => {
    onWorkspaceChange?.({ forecastAnalysisId: null });
  }, [onWorkspaceChange]);

  const setPanelMode = useCallback((mode: PanelMode) => {
    onWorkspaceChange?.({ panelMode: mode });
  }, [onWorkspaceChange]);

  return {
    activeSection: workspace.forecastSection,
    navOpen: workspace.forecastNavOpen,
    currentAnalysisId: workspace.forecastAnalysisId,
    panelMode: workspace.panelMode,
    setSection,
    toggleNav,
    loadAnalysis,
    focusAnalysis,
    closeAnalysis,
    setPanelMode,
  };
}
