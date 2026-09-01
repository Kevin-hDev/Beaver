import { useCallback, useEffect, useMemo, useState } from "react";
import {
  loadForecastPanelValue,
  removeForecastPanelValue,
  saveForecastPanelValue,
} from "@/hooks/forecast-panel-storage";
import { normalizeForecastPanelState } from "@/types/forecast-panel";
import {
  DEFAULT_AGENT_LOCAL_WORKSPACE,
  type AgentLocalWorkspaceState,
} from "@/types/navigation";

const MAX_TRACKED_SESSION_WORKSPACES = 32;
const FORECAST_PANEL_STORAGE_VERSION = 1;

interface WorkspaceEntry {
  sessionId: string | null;
  state: AgentLocalWorkspaceState;
}

function initialWorkspace(sessionId: string | null): AgentLocalWorkspaceState {
  if (!sessionId) return DEFAULT_AGENT_LOCAL_WORKSPACE;
  const forecast = normalizeForecastPanelState(loadForecastPanelValue(sessionId));
  return {
    ...DEFAULT_AGENT_LOCAL_WORKSPACE,
    panelMode: forecast.panelMode,
    forecastSection: forecast.activeSection,
    forecastNavOpen: forecast.navOpen,
    forecastAnalysisId: forecast.currentAnalysisId,
  };
}

function currentWorkspace(
  entries: WorkspaceEntry[],
  sessionId: string | null,
): AgentLocalWorkspaceState {
  return entries.find((entry) => entry.sessionId === sessionId)?.state
    ?? initialWorkspace(sessionId);
}

function changesWorkspace(
  current: AgentLocalWorkspaceState,
  partial: Partial<AgentLocalWorkspaceState>,
): boolean {
  return Object.entries(partial).some(([key, value]) => (
    current[key as keyof AgentLocalWorkspaceState] !== value
  ));
}

export function useAgentSessionWorkspace(sessionId: string | null) {
  const [entries, setEntries] = useState<WorkspaceEntry[]>([]);
  const tracked = entries.find((entry) => entry.sessionId === sessionId);
  const workspace = useMemo(
    () => tracked?.state ?? initialWorkspace(sessionId),
    [sessionId, tracked],
  );

  useEffect(() => {
    if (!sessionId || !tracked) return;
    saveForecastPanelValue(sessionId, {
      version: FORECAST_PANEL_STORAGE_VERSION,
      activeSection: tracked.state.forecastSection,
      navOpen: tracked.state.forecastNavOpen,
      currentAnalysisId: tracked.state.forecastAnalysisId,
      panelMode: tracked.state.panelMode,
    });
  }, [sessionId, tracked]);

  const updateWorkspace = useCallback((partial: Partial<AgentLocalWorkspaceState>) => {
    if (Object.keys(partial).length === 0) return;
    setEntries((previous) => {
      const current = currentWorkspace(previous, sessionId);
      if (!changesWorkspace(current, partial)) return previous;
      const next = { ...current, ...partial };
      const retained = previous.filter((entry) => entry.sessionId !== sessionId);
      return [...retained, { sessionId, state: next }].slice(-MAX_TRACKED_SESSION_WORKSPACES);
    });
  }, [sessionId]);

  const clearWorkspace = useCallback((clearedSessionId: string) => {
    removeForecastPanelValue(clearedSessionId);
    setEntries((previous) => {
      const retained = previous.filter((entry) => entry.sessionId !== clearedSessionId);
      return retained.length === previous.length ? previous : retained;
    });
  }, []);

  return { workspace, updateWorkspace, clearWorkspace };
}
