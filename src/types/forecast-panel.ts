export type ForecastSection = "view" | "comparisons" | "history";
export type PanelMode = "preview" | "forecast" | "browser";

export interface ForecastPanelState {
  activeSection: ForecastSection;
  navOpen: boolean;
  currentAnalysisId: string | null;
  panelMode: PanelMode;
}

export const MAX_FORECAST_ANALYSIS_ID_LENGTH = 128;

export const DEFAULT_FORECAST_PANEL_STATE: ForecastPanelState = {
  activeSection: "view",
  navOpen: false,
  currentAnalysisId: null,
  panelMode: "preview",
};

const SECTIONS: ForecastSection[] = ["view", "comparisons", "history"];

export function normalizeForecastSection(value: unknown): ForecastSection {
  return SECTIONS.includes(value as ForecastSection)
    ? value as ForecastSection
    : DEFAULT_FORECAST_PANEL_STATE.activeSection;
}

export function normalizeForecastPanelState(value: unknown): ForecastPanelState {
  if (!value || typeof value !== "object") return DEFAULT_FORECAST_PANEL_STATE;
  const raw = value as Partial<ForecastPanelState>;
  const validAnalysisId = typeof raw.currentAnalysisId === "string"
    && raw.currentAnalysisId.length > 0
    && raw.currentAnalysisId.length <= MAX_FORECAST_ANALYSIS_ID_LENGTH;
  return {
    activeSection: normalizeForecastSection(raw.activeSection),
    navOpen: typeof raw.navOpen === "boolean"
      ? raw.navOpen
      : DEFAULT_FORECAST_PANEL_STATE.navOpen,
    currentAnalysisId: validAnalysisId ? raw.currentAnalysisId ?? null : null,
    panelMode: raw.panelMode === "forecast" || raw.panelMode === "browser"
      ? raw.panelMode
      : DEFAULT_FORECAST_PANEL_STATE.panelMode,
  };
}
