import { beforeEach, describe, expect, it, vi } from "vitest";
import { agentStreamManager } from "../agent-stream-manager";
import { records } from "../agent-stream-records";
import type { AgentMessage, StreamEvent } from "@/types/agent";

const mocks = vi.hoisted(() => ({ invoke: vi.fn(), listen: vi.fn() }));
let streamHandler: ((event: {
  payload: { sessionId: string; event: StreamEvent };
}) => void) | null = null;

vi.mock("@tauri-apps/api/core", () => ({ invoke: mocks.invoke }));
vi.mock("@tauri-apps/api/event", () => ({ listen: mocks.listen }));

describe("subagent backend persistence ownership", () => {
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

    expect(persistCalls()).toHaveLength(0);
  });

  it("ne persiste pas deux fois si le snapshot enfant précède startSession", async () => {
    agentStreamManager.subscribe("child-b", () => {});
    emit("child-b", snapshotEvent([message("u1", "user", "mission")]));
    await agentStreamManager.startSession("child-b", [message("u1", "user", "mission")], 0);
    emit("child-b", tokenEvent("rapport"));
    emit("child-b", doneEvent());

    expect(persistCalls()).toHaveLength(0);
  });

  it("réactive la persistance frontend pour un nouveau stream UI après un gateway", async () => {
    agentStreamManager.subscribe("gateway", () => {});
    emit("gateway", tokenEvent("backend"));
    emit("gateway", doneEvent());
    expect(persistCalls()).toHaveLength(0);

    await agentStreamManager.startSession("gateway", [message("u2", "user", "question")], 0);
    emit("gateway", tokenEvent("frontend"));
    emit("gateway", doneEvent());

    expect(persistCalls()).toHaveLength(1);
  });
});

function emit(sessionId: string, event: StreamEvent) {
  streamHandler?.({ payload: { sessionId, event } });
}

function persistCalls() {
  return mocks.invoke.mock.calls.filter(([command]) => command === "add_messages_to_session");
}

function message(id: string, role: AgentMessage["role"], content: string): AgentMessage {
  return { id, role, content, timestamp: "2026-07-11T10:00:00Z", files: [] };
}

function snapshotEvent(messages: AgentMessage[]): StreamEvent {
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
