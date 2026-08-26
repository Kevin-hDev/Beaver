import { beforeEach, describe, expect, it, vi } from "vitest";
import { agentStreamManager } from "../agent-stream-manager";
import { records } from "../agent-stream-records";
import type { AgentMessage, StreamEvent } from "@/types/agent";

const mocks = vi.hoisted(() => ({ invoke: vi.fn(), listen: vi.fn() }));
let streamHandler: ((event: {
  payload: { sessionId: string; generation?: number; event: StreamEvent };
}) => void) | null = null;

vi.mock("@tauri-apps/api/core", () => ({ invoke: mocks.invoke }));
vi.mock("@tauri-apps/api/event", () => ({ listen: mocks.listen }));

describe("active stream user queue", () => {
  beforeEach(() => {
    records.clear();
    vi.clearAllMocks();
    vi.stubGlobal("requestAnimationFrame", vi.fn());
    vi.stubGlobal("cancelAnimationFrame", vi.fn());
    mocks.listen.mockImplementation((_event: string, handler: typeof streamHandler) => {
      streamHandler = handler;
      return Promise.resolve(() => {});
    });
  });

  it("garde l'intention en attente tant qu'aucun commit durable n'est signalé", async () => {
    await agentStreamManager.startSession("s1", [message("u1", "Question")], 10);
    agentStreamManager.setSessionGeneration("s1", 7);
    emit({ event: "token", data: { content: "Travail en cours", tokenCount: 3, tps: 1 } });

    expect(agentStreamManager.queueUserMessage("s1", message("u2", "Ajoute ceci"))).toBe(true);
    expect(agentStreamManager.getSnapshot("s1")?.currentContent).toBe("Travail en cours");
    expect(agentStreamManager.getSnapshot("s1")?.queuedUserMessages).toHaveLength(1);
    emit({ event: "turnEnd", data: {} });

    const after = agentStreamManager.getSnapshot("s1");
    expect(after?.isStreaming).toBe(true);
    expect(after?.messages.map((item) => item.content)).toEqual(["Question"]);
    expect(after?.completedSegments).toHaveLength(1);
    expect(after?.queuedUserMessages.map((item) => item.content)).toEqual(["Ajoute ceci"]);
    expect(mocks.invoke).not.toHaveBeenCalled();
    emit({ event: "token", data: { content: "Réponse suivante", tokenCount: 2, tps: 1 } });
    emit({
      event: "done",
      data: {
        evalCount: 2, evalDurationNs: 0, finalTps: 1, promptTokens: 5, contextTokens: 20,
      },
    });
    expect(agentStreamManager.getSnapshot("s1")?.queuedUserMessages).toHaveLength(1);
    expect(mocks.invoke).not.toHaveBeenCalled();
  });
});

function message(id: string, content: string): AgentMessage {
  return { id, role: "user", content, files: [], timestamp: "2026-07-12T10:00:00Z" };
}

function emit(event: StreamEvent) {
  streamHandler?.({ payload: { sessionId: "s1", generation: 7, event } });
}
