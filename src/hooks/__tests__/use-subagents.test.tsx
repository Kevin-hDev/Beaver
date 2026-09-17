import { act, renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { records } from "../agent-stream-records";
import { useSubagents } from "../use-subagents";
import type { StreamEvent } from "@/types/agent";

let streamHandler: ((event: { payload: unknown }) => void) | null = null;
const invokeMock = vi.fn();

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]): Promise<unknown> => invokeMock(...args) as Promise<unknown>,
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn((_event: string, handler: (event: { payload: unknown }) => void) => {
    streamHandler = handler;
    return Promise.resolve(() => {});
  }),
}));

beforeEach(() => {
  records.clear();
  invokeMock.mockReset();
  invokeMock.mockResolvedValue([]);
});

describe("useSubagents", () => {
  it("partage les transitions de sous-agent entre plusieurs vues", async () => {
    const first = renderHook(() => useSubagents("parent"));
    const second = renderHook(() => useSubagents("parent"));
    await waitFor(() => expect(streamHandler).toBeTruthy());

    act(() => emit("parent", {
      event: "subagentSpawned",
      data: {
        subagentSessionId: "child",
        subagentName: "Explorer",
        subagentType: "explorer",
        subagentDescription: "Inspecter",
        subagentColorKey: "geminitor",
        promptPreview: "Cherche",
        runId: "run-1",
      },
    }));

    expect(first.result.current.active[0]?.sessionId).toBe("child");
    expect(second.result.current.active[0]?.sessionId).toBe("child");

    act(() => emit("parent", {
      event: "subagentCompleted",
      data: {
        subagentSessionId: "child",
        success: true,
        status: "completed",
        summary: "Terminé",
        runId: "run-1",
      },
    }));

    expect(first.result.current.active).toEqual([]);
    expect(first.result.current.completed[0]?.summary).toBe("Terminé");
    expect(second.result.current.completed[0]?.summary).toBe("Terminé");
  });

  it("retire immédiatement un sous-agent annulé de la projection commune", async () => {
    const { result } = renderHook(() => useSubagents("parent"));
    await waitFor(() => expect(streamHandler).toBeTruthy());
    act(() => emit("parent", {
      event: "subagentSpawned",
      data: {
        subagentSessionId: "child",
        subagentName: "Explorer",
        subagentType: "explorer",
        subagentDescription: "Inspecter",
        subagentColorKey: "geminitor",
        promptPreview: "Cherche",
      },
    }));

    await act(async () => result.current.cancelSubagent("child"));

    expect(invokeMock).toHaveBeenCalledWith("cancel_subagent", { subagentSessionId: "child" });
    expect(result.current.active).toEqual([]);
  });
});

function emit(sessionId: string, event: StreamEvent) {
  streamHandler?.({ payload: { sessionId, event } });
}
