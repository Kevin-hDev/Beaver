import { useState, useCallback } from "react";
import {
  loadForecastPanelValue,
  saveForecastPanelValue,
  withBoundedPanelState,
} from "./forecast-panel-storage";

export type ForecastSection = "view" | "comparisons" | "history";
export type PanelMode = "preview" | "forecast" | "browser";

export interface ForecastPanelState {
  activeSection: ForecastSection;
  navOpen: boolean;
  currentAnalysisId: string | null;
  panelMode: PanelMode;
}

const DEFAULT_PANEL_STATE = {
  activeSection: "view" as ForecastSection,
  navOpen: false,
  currentAnalysisId: null,
  panelMode: "preview" as PanelMode,
};

const SECTIONS: ForecastSection[] = ["view", "comparisons", "history"];

export function normalizeForecastSection(value: unknown): ForecastSection {
  return SECTIONS.includes(value as ForecastSection)
    ? value as ForecastSection
    : DEFAULT_PANEL_STATE.activeSection;
}

function normalizePanelState(value: unknown): ForecastPanelState {
  if (!value || typeof value !== "object") return DEFAULT_PANEL_STATE;
  const raw = value as Partial<ForecastPanelState>;
  return {
    activeSection: normalizeForecastSection(raw.activeSection),
    navOpen: typeof raw.navOpen === "boolean" ? raw.navOpen : DEFAULT_PANEL_STATE.navOpen,
    currentAnalysisId: typeof raw.currentAnalysisId === "string" ? raw.currentAnalysisId : null,
    panelMode: raw.panelMode === "forecast" || raw.panelMode === "browser"
      ? raw.panelMode
      : "preview",
  };
}

function samePanelState(a: ForecastPanelState, b: ForecastPanelState): boolean {
  return a.activeSection === b.activeSection &&
    a.navOpen === b.navOpen &&
    a.currentAnalysisId === b.currentAnalysisId &&
    a.panelMode === b.panelMode;
}

export function useForecastPanel(sessionId: string | null) {
  const stateKey = sessionId ?? "__no_session__";

  const [states, setStates] = useState<Record<string, ForecastPanelState>>(() => {
    const saved = sessionId
      ? normalizePanelState(loadForecastPanelValue(sessionId))
      : DEFAULT_PANEL_STATE;
    return { [stateKey]: saved };
  });
  const state = states[stateKey] ?? (
    sessionId
      ? normalizePanelState(loadForecastPanelValue(sessionId))
      : DEFAULT_PANEL_STATE
  );

  const persist = useCallback((next: ForecastPanelState) => {
    setStates((prev) => withBoundedPanelState(prev, stateKey, next));
    if (sessionId) saveForecastPanelValue(sessionId, next);
  }, [sessionId, stateKey]);

  const setSection = useCallback((section: ForecastSection) => {
    persist({ ...state, activeSection: section, navOpen: false });
  }, [state, persist]);

  const toggleNav = useCallback(() => {
    persist({ ...state, navOpen: !state.navOpen });
  }, [state, persist]);

  const loadAnalysis = useCallback((id: string) => {
    persist({ ...state, currentAnalysisId: id, activeSection: "view", panelMode: "forecast" });
  }, [state, persist]);

  const focusAnalysis = useCallback((id: string) => {
    persist({ ...state, currentAnalysisId: id, panelMode: "forecast" });
  }, [state, persist]);

  const closeAnalysis = useCallback(() => {
    persist({ ...state, currentAnalysisId: null });
  }, [state, persist]);

  const setPanelMode = useCallback((mode: PanelMode) => {
    persist({ ...state, panelMode: mode });
  }, [state, persist]);

  const restorePanelState = useCallback((next: ForecastPanelState) => {
    const normalized = {
      ...next,
      activeSection: normalizeForecastSection(next.activeSection),
    };
    if (samePanelState(state, normalized)) return;
    persist(normalized);
  }, [state, persist]);

  return {
    ...state,
    setSection,
    toggleNav,
    loadAnalysis,
    focusAnalysis,
    closeAnalysis,
    setPanelMode,
    restorePanelState,
  };
}
