import { act, renderHook } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { useAgentStream } from "../use-agent-stream";
import { agentStreamManager } from "../agent-stream-manager";
import { records } from "../agent-stream-records";
import type { AgentMessage, StreamEvent } from "@/types/agent";
import type { ChatStreamAdmission, TurnStart } from "@/types/agent-turn.generated";

const mocks = vi.hoisted(() => ({ invoke: vi.fn(), listen: vi.fn() }));
let streamHandler: ((event: { payload: StreamEnvelope }) => void) | null = null;

interface StreamEnvelope {
  sessionId: string;
  generation?: number;
  event: StreamEvent;
}

vi.mock("@tauri-apps/api/core", () => ({ invoke: mocks.invoke }));
vi.mock("@tauri-apps/api/event", () => ({ listen: mocks.listen }));
vi.mock("../session-reasoning-mutation", () => ({
  awaitPendingReasoning: vi.fn().mockResolvedValue(undefined),
}));

function deferred<T>() {
  let resolve!: (value: T) => void;
  let reject!: (reason: unknown) => void;
  const promise = new Promise<T>((yes, no) => { resolve = yes; reject = no; });
  return { promise, resolve, reject };
}

function admission(generation: number): ChatStreamAdmission {
  return {
    generation,
    turnId: `00000000-0000-4000-8000-${generation.toString().padStart(12, "0")}`,
    userMessageId: `10000000-0000-4000-8000-${generation.toString().padStart(12, "0")}`,
    assistantMessageId: `20000000-0000-4000-8000-${generation.toString().padStart(12, "0")}`,
  };
}

function turn(content: string): TurnStart {
  return { type: "new", input: { content, files: [], skills: [] } };
}

function message(content: string): AgentMessage {
  return {
    id: `optimistic-${content}`,
    role: "user",
    content,
    files: [],
    timestamp: "2026-08-26T10:00:00Z",
  };
}

function emit(sessionId: string, generation: number, event: StreamEvent) {
  streamHandler?.({ payload: { sessionId, generation, event } });
}

function start(result: ReturnType<typeof renderHook<ReturnType<typeof useAgentStream>, unknown>>["result"], content: string) {
  return result.current.startStream(
    "same-session",
    "model",
    "provider",
    turn(content),
    false,
    { displayMessages: [message(content)], baseTokenCount: 0 },
    undefined,
    undefined,
    undefined,
    undefined,
    undefined,
    undefined,
    undefined,
    `optimistic-${content}`,
  );
}

describe("useAgentStream admission races", () => {
  beforeEach(() => {
    records.clear();
    vi.clearAllMocks();
    mocks.listen.mockImplementation((_name: string, handler: typeof streamHandler) => {
      streamHandler = handler;
      return Promise.resolve(() => {});
    });
  });

  it("ne laisse pas une ancienne résolution vider les événements précoces du nouveau run", async () => {
    const first = deferred<ChatStreamAdmission>();
    const second = deferred<ChatStreamAdmission>();
    let chatCalls = 0;
    mocks.invoke.mockImplementation((command: string) => {
      if (command === "chat_stream") {
        chatCalls += 1;
        return chatCalls === 1 ? first.promise : second.promise;
      }
      return Promise.resolve(undefined);
    });
    const { result } = renderHook(() => useAgentStream());
    let firstRun!: Promise<void>;
    let secondRun!: Promise<void>;

    await act(async () => {
      firstRun = start(result, "first");
      await vi.waitFor(() => expect(chatCalls).toBe(1));
      secondRun = start(result, "second");
      await vi.waitFor(() => expect(chatCalls).toBe(2));
      emit("same-session", 22, {
        event: "token",
        data: { content: "new early", tokenCount: 1, tps: 1 },
      });
      first.resolve(admission(11));
      await firstRun;
    });
    expect(agentStreamManager.getSnapshot("same-session")?.currentContent).toBe("");

    await act(async () => {
      second.resolve(admission(22));
      await secondRun;
    });
    expect(agentStreamManager.getSnapshot("same-session")?.currentContent).toBe("new early");
    expect(mocks.invoke.mock.calls.filter(([command]) => command === "cancel_agent_request"))
      .toEqual([["cancel_agent_request", { sessionId: "same-session", generation: 11 }]]);
  });

  it("ne laisse pas un ancien rejet échouer le nouveau run de la même session", async () => {
    const first = deferred<ChatStreamAdmission>();
    const second = deferred<ChatStreamAdmission>();
    let chatCalls = 0;
    mocks.invoke.mockImplementation((command: string) => {
      if (command === "chat_stream") {
        chatCalls += 1;
        return chatCalls === 1 ? first.promise : second.promise;
      }
      return Promise.resolve(undefined);
    });
    const { result } = renderHook(() => useAgentStream());
    let firstRun!: Promise<void>;
    let secondRun!: Promise<void>;

    await act(async () => {
      firstRun = start(result, "rejected");
      await vi.waitFor(() => expect(chatCalls).toBe(1));
      secondRun = start(result, "current");
      await vi.waitFor(() => expect(chatCalls).toBe(2));
      emit("same-session", 44, {
        event: "token",
        data: { content: "kept", tokenCount: 1, tps: 1 },
      });
      first.reject(new Error("stale"));
      await firstRun;
      second.resolve(admission(44));
      await secondRun;
    });

    const current = agentStreamManager.getSnapshot("same-session");
    expect(current?.currentContent).toBe("kept");
    expect(current?.error).toBeUndefined();
  });

  it("annule exactement la génération admise quand le buffer précoce déborde", async () => {
    const pending = deferred<ChatStreamAdmission>();
    mocks.invoke.mockImplementation((command: string) => {
      if (command === "chat_stream") return pending.promise;
      return Promise.resolve(undefined);
    });
    const { result } = renderHook(() => useAgentStream());
    let running!: Promise<void>;
    await act(async () => {
      running = start(result, "overflow");
      await vi.waitFor(() => expect(mocks.invoke).toHaveBeenCalledWith(
        "chat_stream", expect.anything(),
      ));
      for (let index = 0; index < 65; index += 1) {
        emit("same-session", 33, {
          event: "token",
          data: { content: `${index}`, tokenCount: 1, tps: 1 },
        });
      }
      pending.resolve(admission(33));
      await running;
    });

    const failed = agentStreamManager.getSnapshot("same-session");
    expect(failed?.completed).toBe(true);
    expect(failed?.error).toBeTruthy();
    expect(mocks.invoke.mock.calls.filter(([command]) => command === "cancel_agent_request"))
      .toEqual([["cancel_agent_request", { sessionId: "same-session", generation: 33 }]]);
    emit("same-session", 33, {
      event: "token",
      data: { content: "late", tokenCount: 1, tps: 1 },
    });
    expect(agentStreamManager.getSnapshot("same-session")?.currentContent).toBe("");
  });
});
