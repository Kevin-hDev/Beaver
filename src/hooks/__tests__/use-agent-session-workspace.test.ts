import { act, renderHook } from "@testing-library/react";
import { beforeEach, describe, expect, it } from "vitest";
import { useAgentSessionWorkspace } from "../use-agent-session-workspace";

describe("useAgentSessionWorkspace", () => {
  beforeEach(() => localStorage.clear());

  it("conserve des états indépendants pour deux sessions", () => {
    const { result, rerender } = renderHook(
      ({ sessionId }) => useAgentSessionWorkspace(sessionId),
      { initialProps: { sessionId: "session-a" } },
    );

    act(() => result.current.updateWorkspace({ terminalOpen: true, previewOpen: true }));
    rerender({ sessionId: "session-b" });
    expect(result.current.workspace.terminalOpen).toBe(false);
    expect(result.current.workspace.previewOpen).toBe(false);

    act(() => result.current.updateWorkspace({ previewOpen: true }));
    rerender({ sessionId: "session-a" });
    expect(result.current.workspace.terminalOpen).toBe(true);
    expect(result.current.workspace.previewOpen).toBe(true);

    rerender({ sessionId: "session-b" });
    expect(result.current.workspace.terminalOpen).toBe(false);
    expect(result.current.workspace.previewOpen).toBe(true);
  });

  it("évince l'état le plus ancien quand trop de sessions sont suivies", () => {
    const { result, rerender } = renderHook(
      ({ sessionId }) => useAgentSessionWorkspace(sessionId),
      { initialProps: { sessionId: "session-0" } },
    );

    for (let index = 0; index < 33; index += 1) {
      rerender({ sessionId: `session-${index}` });
      act(() => result.current.updateWorkspace({ terminalOpen: true }));
    }

    rerender({ sessionId: "session-0" });
    expect(result.current.workspace.terminalOpen).toBe(false);
    rerender({ sessionId: "session-32" });
    expect(result.current.workspace.terminalOpen).toBe(true);
  });

  it("hydrate le forecast sauvegardé puis le conserve comme état du workspace", () => {
    localStorage.setItem("fc-panel-session-a", JSON.stringify({
      activeSection: "history",
      navOpen: true,
      currentAnalysisId: "analysis-a",
      panelMode: "forecast",
    }));
    const { result } = renderHook(() => useAgentSessionWorkspace("session-a"));

    expect(result.current.workspace).toMatchObject({
      forecastSection: "history",
      forecastNavOpen: true,
      forecastAnalysisId: "analysis-a",
      panelMode: "forecast",
    });

    act(() => result.current.updateWorkspace({ previewOpen: true }));
    expect(JSON.parse(localStorage.getItem("fc-panel-session-a") ?? "null")).toEqual({
      version: 1,
      activeSection: "history",
      navOpen: true,
      currentAnalysisId: "analysis-a",
      panelMode: "forecast",
    });
  });

  it("ne crée aucune entrée pour un patch vide ou identique", () => {
    const { result } = renderHook(() => useAgentSessionWorkspace("session-a"));
    const initialWorkspace = result.current.workspace;

    act(() => result.current.updateWorkspace({}));
    expect(result.current.workspace).toBe(initialWorkspace);
    act(() => result.current.updateWorkspace({ terminalOpen: false }));
    expect(result.current.workspace).toBe(initialWorkspace);
  });

  it("purge l'état mémoire et sa sauvegarde quand une session est supprimée", () => {
    const { result, rerender } = renderHook(
      ({ sessionId }) => useAgentSessionWorkspace(sessionId),
      { initialProps: { sessionId: "session-a" } },
    );
    act(() => result.current.updateWorkspace({ terminalOpen: true, panelMode: "forecast" }));
    expect(localStorage.getItem("fc-panel-session-a")).not.toBeNull();

    act(() => result.current.clearWorkspace("session-a"));
    rerender({ sessionId: "session-b" });
    rerender({ sessionId: "session-a" });

    expect(result.current.workspace.terminalOpen).toBe(false);
    expect(result.current.workspace.panelMode).toBe("preview");
    expect(localStorage.getItem("fc-panel-session-a")).toBeNull();
  });
});
