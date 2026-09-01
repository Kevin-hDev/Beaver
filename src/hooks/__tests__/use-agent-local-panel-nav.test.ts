import { renderHook } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { useAgentLocalPanelNav } from "../use-agent-local-panel-nav";
import { DEFAULT_AGENT_LOCAL_NAV } from "@/types/navigation";
import type { useFileTree } from "../use-file-tree";
import type { useForecastPanel } from "../use-forecast-panel";

function fileTree(open: boolean) {
  return {
    open,
    setOpen: vi.fn(),
  } as unknown as ReturnType<typeof useFileTree>;
}

function forecast() {
  return {
    panelMode: "preview",
    activeSection: "view",
    currentAnalysisId: null,
    restorePanelState: vi.fn(),
  } as unknown as ReturnType<typeof useForecastPanel>;
}

describe("useAgentLocalPanelNav", () => {
  it("restaure file tree et forecast panel depuis la navigation", () => {
    const tree = fileTree(false);
    const panel = forecast();

    renderHook(() => useAgentLocalPanelNav({
      navState: {
        ...DEFAULT_AGENT_LOCAL_NAV,
        fileTreeOpen: true,
        panelMode: "forecast",
        forecastSection: "comparisons",
        forecastAnalysisId: "a1",
      },
      fileTree: tree,
      forecast: panel,
    }));

    expect(tree.setOpen).toHaveBeenCalledWith(true);
    expect(panel.restorePanelState).toHaveBeenCalledWith({
      activeSection: "comparisons",
      navOpen: false,
      currentAnalysisId: "a1",
      panelMode: "forecast",
    });
  });

  it("n'applique rien quand l'état local correspond déjà à la navigation", () => {
    const tree = fileTree(false);
    const panel = forecast();

    renderHook(() => useAgentLocalPanelNav({
      navState: DEFAULT_AGENT_LOCAL_NAV,
      fileTree: tree,
      forecast: panel,
    }));

    expect(tree.setOpen).not.toHaveBeenCalled();
    expect(panel.restorePanelState).not.toHaveBeenCalled();
  });

  it("ne referme pas un panneau ouvert localement avant le push nav", () => {
    const setOpen = vi.fn();
    const tree = { ...fileTree(false), setOpen };
    const panel = forecast();
    const navState = DEFAULT_AGENT_LOCAL_NAV;

    const { rerender } = renderHook(
      ({ open }) => useAgentLocalPanelNav({
        navState,
        fileTree: { ...tree, open },
        forecast: panel,
      }),
      { initialProps: { open: false } },
    );

    setOpen.mockClear();
    rerender({ open: true });

    expect(setOpen).not.toHaveBeenCalledWith(false);
  });

  it("resynchronise l'arborescence quand la session change avec les mêmes valeurs", () => {
    const setOpen = vi.fn();
    const panel = forecast();
    const { rerender } = renderHook(
      ({ sessionId, open }) => useAgentLocalPanelNav({
        navState: { ...DEFAULT_AGENT_LOCAL_NAV, sessionId },
        fileTree: { ...fileTree(open), setOpen },
        forecast: panel,
      }),
      { initialProps: { sessionId: "session-a", open: false } },
    );

    rerender({ sessionId: "session-b", open: true });

    expect(setOpen).toHaveBeenCalledWith(false);
  });
});
