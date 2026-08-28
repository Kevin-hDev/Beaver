import { beforeEach, describe, expect, it, vi } from "vitest";
import { agentStreamManager } from "../agent-stream-manager";
import { records } from "../agent-stream-records";
import type { AgentMessage, StreamEvent } from "@/types/agent";
import type { AgentMessageView } from "@/types/agent-session.generated";

const mocks = vi.hoisted(() => ({ invoke: vi.fn(), listen: vi.fn() }));
let streamHandler: ((event: {
  payload: { sessionId: string; event: StreamEvent };
}) => void) | null = null;

vi.mock("@tauri-apps/api/core", () => ({ invoke: mocks.invoke }));
vi.mock("@tauri-apps/api/event", () => ({ listen: mocks.listen }));

describe("autorité de persistance Rust", () => {
  beforeEach(() => {
    records.clear();
    vi.clearAllMocks();
    mocks.invoke.mockResolvedValue(undefined);
    mocks.listen.mockImplementation((_event: string, handler: typeof streamHandler) => {
      streamHandler = handler;
      return Promise.resolve(() => {});
    });
    vi.stubGlobal("requestAnimationFrame", vi.fn());
    vi.stubGlobal("cancelAnimationFrame", vi.fn());
  });

  it("ne persiste pas deux fois si startSession précède le snapshot enfant", async () => {
    await agentStreamManager.startSession("child-a", [message("u1", "user", "mission")], 0);
    emit("child-a", snapshotEvent([message("u1", "user", "mission")]));
    emit("child-a", tokenEvent("rapport"));
    emit("child-a", doneEvent());

    expect(mocks.invoke).not.toHaveBeenCalled();
  });

  it("ne persiste pas deux fois si le snapshot enfant précède startSession", async () => {
    agentStreamManager.subscribe("child-b", () => {});
    emit("child-b", snapshotEvent([message("u1", "user", "mission")]));
    await agentStreamManager.startSession("child-b", [message("u1", "user", "mission")], 0);
    emit("child-b", tokenEvent("rapport"));
    emit("child-b", doneEvent());

    expect(mocks.invoke).not.toHaveBeenCalled();
  });

  it("ne réactive jamais une persistance frontend après un gateway", async () => {
    agentStreamManager.subscribe("gateway", () => {});
    emit("gateway", tokenEvent("backend"));
    emit("gateway", doneEvent());
    expect(mocks.invoke).not.toHaveBeenCalled();

    await agentStreamManager.startSession("gateway", [message("u2", "user", "question")], 0);
    emit("gateway", {
      event: "contextUsage",
      data: {
        inputTokens: 1,
        outputTokens: 0,
        contextLimit: 372_000,
        estimated: true,
      },
    });
    emit("gateway", tokenEvent("frontend"));
    emit("gateway", doneEvent());

    expect(mocks.invoke).not.toHaveBeenCalled();
  });

  it("conserve l'affichage partiel sur erreur sans écriture frontend", async () => {
    await agentStreamManager.startSession("ui", [message("u1", "user", "question")], 0);
    emit("ui", tokenEvent("réponse partielle"));
    emit("ui", { event: "error", data: { message: "provider_error" } });

    const snapshot = agentStreamManager.getSnapshot("ui");
    const messages = snapshot?.messages ?? [];
    expect(messages[messages.length - 1]?.content).toBe("réponse partielle");
    expect(mocks.invoke).not.toHaveBeenCalled();
  });
});

function emit(sessionId: string, event: StreamEvent) {
  streamHandler?.({ payload: { sessionId, event } });
}

function message(
  id: string,
  role: AgentMessage["role"],
  content: string,
): AgentMessage & AgentMessageView {
  return {
    id, turn_id: `turn-${id}`, role, content,
    timestamp: "2026-07-11T10:00:00Z", files: [], tokens: 0,
    reasoning_replay_status: "unavailable",
  };
}

function snapshotEvent(messages: AgentMessageView[]): StreamEvent {
  return { event: "sessionSnapshot", data: { messages, tokenCount: 0 } };
}

function tokenEvent(content: string): StreamEvent {
  return { event: "token", data: { content, tokenCount: 1, tps: 1 } };
}

function doneEvent(): StreamEvent {
  return {
    event: "done",
    data: {
      evalCount: 1,
      evalDurationNs: 0,
      finalTps: 1,
      promptTokens: 1,
      contextTokens: 2,
    },
  };
}
