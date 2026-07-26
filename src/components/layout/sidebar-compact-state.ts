export interface AgentSidebarLayoutState {
  sidebarOpen: boolean;
  manualReveal: boolean;
  panelsTight: boolean;
}

export const INITIAL_AGENT_SIDEBAR_LAYOUT_STATE: AgentSidebarLayoutState = {
  sidebarOpen: true,
  manualReveal: false,
  panelsTight: false,
};

export function toggleAgentSidebar(state: AgentSidebarLayoutState): AgentSidebarLayoutState {
  const sidebarOpen = !state.sidebarOpen;
  return {
    ...state,
    sidebarOpen,
    manualReveal: sidebarOpen,
  };
}

export function autoHideAgentSidebar(state: AgentSidebarLayoutState): AgentSidebarLayoutState {
  if (!state.sidebarOpen && state.panelsTight && !state.manualReveal) return state;
  return {
    ...state,
    sidebarOpen: false,
    manualReveal: false,
    panelsTight: true,
  };
}

export function setAgentPanelsTight(
  state: AgentSidebarLayoutState,
  panelsTight: boolean,
): AgentSidebarLayoutState {
  if (state.panelsTight === panelsTight) return state;
  return {
    ...state,
    panelsTight,
    manualReveal: panelsTight || state.sidebarOpen ? state.manualReveal : false,
  };
}

export function shouldCompactAgentChat(state: AgentSidebarLayoutState): boolean {
  return state.sidebarOpen && state.manualReveal && state.panelsTight;
}
