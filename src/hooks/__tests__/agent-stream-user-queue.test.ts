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

  it("conserve le travail visible avant le nouveau message", async () => {
    await agentStreamManager.startSession("s1", [message("u1", "Question")], 10);
    agentStreamManager.setSessionGeneration("s1", 7);
    emit({ event: "token", data: { content: "Travail en cours", tokenCount: 3, tps: 1 } });

    expect(agentStreamManager.queueUserMessage("s1", message("u2", "Ajoute ceci"))).toBe(true);
    expect(agentStreamManager.getSnapshot("s1")?.currentContent).toBe("Travail en cours");
    expect(agentStreamManager.getSnapshot("s1")?.queuedUserMessages).toHaveLength(1);
    emit({ event: "turnEnd", data: {} });

    const after = agentStreamManager.getSnapshot("s1");
    expect(after?.isStreaming).toBe(true);
    expect(after?.messages.map((item) => item.content)).toEqual([
      "Question", "Travail en cours", "Ajoute ceci",
    ]);
    expect(after?.completedSegments).toEqual([]);
    expect(after?.queuedUserMessages).toEqual([]);
    const checkpoint = after?.messages[1] as StreamGroupedMessage;
    const input = after?.messages[2] as StreamGroupedMessage;
    expect(checkpoint.stream_run_id).toMatch(UUID_RE);
    expect(checkpoint.stream_part).toBe("checkpoint");
    expect(input.stream_run_id).toBe(checkpoint.stream_run_id);
    expect(input.stream_part).toBe("input");
    await vi.waitFor(() => expect(mocks.invoke).toHaveBeenCalledWith(
      "add_messages_to_session",
      expect.objectContaining({
        id: "s1",
        messages: [
          expect.objectContaining({ role: "assistant", content: "Travail en cours" }),
          expect.objectContaining({ role: "user", content: "Ajoute ceci" }),
        ],
      }),
    ));
    emit({ event: "token", data: { content: "Réponse suivante", tokenCount: 2, tps: 1 } });
    emit({
      event: "done",
      data: {
        evalCount: 2, evalDurationNs: 0, finalTps: 1, promptTokens: 5, contextTokens: 20,
      },
    });
    await vi.waitFor(() => expect(mocks.invoke).toHaveBeenCalledTimes(2));
    expect(mocks.invoke.mock.calls[1]?.[1]).toEqual(expect.objectContaining({
      messages: [expect.objectContaining({
        content: "Réponse suivante",
        stream_run_id: checkpoint.stream_run_id,
        stream_part: "final",
      })],
    }));
  });
});

function message(id: string, content: string): AgentMessage {
  return { id, role: "user", content, files: [], timestamp: "2026-07-12T10:00:00Z" };
}

interface StreamGroupedMessage extends AgentMessage {
  stream_run_id?: string;
  stream_part?: "checkpoint" | "input" | "final";
}

const UUID_RE = /^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i;

function emit(event: StreamEvent) {
  streamHandler?.({ payload: { sessionId: "s1", generation: 7, event } });
}
