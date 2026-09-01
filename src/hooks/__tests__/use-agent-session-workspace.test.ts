import { act, renderHook } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { useAgentSessionWorkspace } from "../use-agent-session-workspace";

describe("useAgentSessionWorkspace", () => {
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
});
