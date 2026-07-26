import { describe, expect, it } from "vitest";
import {
  INITIAL_AGENT_SIDEBAR_LAYOUT_STATE,
  autoHideAgentSidebar,
  setAgentPanelsTight,
  shouldCompactAgentChat,
  toggleAgentSidebar,
} from "../sidebar-compact-state";

describe("sidebar compact state", () => {
  it("garde le chat compact sur les reouvertures successives quand l'espace reste serre", () => {
    let state = autoHideAgentSidebar(INITIAL_AGENT_SIDEBAR_LAYOUT_STATE);

    state = toggleAgentSidebar(state);
    expect(state).toMatchObject({ sidebarOpen: true, manualReveal: true, panelsTight: true });
    expect(shouldCompactAgentChat(state)).toBe(true);

    state = toggleAgentSidebar(state);
    expect(state).toMatchObject({ sidebarOpen: false, manualReveal: false, panelsTight: true });

    state = toggleAgentSidebar(state);
    expect(state).toMatchObject({ sidebarOpen: true, manualReveal: true, panelsTight: true });
    expect(shouldCompactAgentChat(state)).toBe(true);
  });

  it("revient au comportement normal quand l'espace n'est plus serre", () => {
    let state = autoHideAgentSidebar(INITIAL_AGENT_SIDEBAR_LAYOUT_STATE);
    state = toggleAgentSidebar(state);
    state = setAgentPanelsTight(state, false);

    expect(state).toMatchObject({ sidebarOpen: true, manualReveal: true, panelsTight: false });
    expect(shouldCompactAgentChat(state)).toBe(false);
  });

  it("active l'override utilisateur meme si la projection disait que ca rentrait", () => {
    let state = {
      ...INITIAL_AGENT_SIDEBAR_LAYOUT_STATE,
      sidebarOpen: false,
      panelsTight: false,
    };

    state = toggleAgentSidebar(state);
    expect(state).toMatchObject({ sidebarOpen: true, manualReveal: true, panelsTight: false });
    expect(shouldCompactAgentChat(state)).toBe(false);

    state = setAgentPanelsTight(state, true);
    expect(state).toMatchObject({ sidebarOpen: true, manualReveal: true, panelsTight: true });
    expect(shouldCompactAgentChat(state)).toBe(true);
  });

  it("ne perd pas l'override pendant un tick large de transition", () => {
    let state = autoHideAgentSidebar(INITIAL_AGENT_SIDEBAR_LAYOUT_STATE);
    state = toggleAgentSidebar(state);
    expect(state).toMatchObject({ sidebarOpen: true, manualReveal: true, panelsTight: true });

    state = setAgentPanelsTight(state, false);
    expect(state).toMatchObject({ sidebarOpen: true, manualReveal: true, panelsTight: false });

    state = setAgentPanelsTight(state, true);
    expect(state).toMatchObject({ sidebarOpen: true, manualReveal: true, panelsTight: true });
    expect(shouldCompactAgentChat(state)).toBe(true);
  });
});
