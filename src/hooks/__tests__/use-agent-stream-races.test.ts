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
  return startForSession(result, "same-session", content);
}

function startForSession(
  result: ReturnType<typeof renderHook<ReturnType<typeof useAgentStream>, unknown>>["result"],
  sessionId: string,
  content: string,
) {
  return result.current.startStream(
    sessionId,
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

  it("arrête la génération exacte de A sans toucher au stream B", async () => {
    mocks.invoke.mockImplementation((command: string, args: unknown) => {
      if (command === "chat_stream") {
        const sessionId = (args as { request: { sessionId: string } }).request.sessionId;
        return Promise.resolve(admission(sessionId === "session-a" ? 11 : 22));
      }
      return Promise.resolve(undefined);
    });
    const { result } = renderHook(() => useAgentStream());

    await act(async () => {
      await startForSession(result, "session-a", "alpha");
      await startForSession(result, "session-b", "beta");
      emit("session-a", 11, {
        event: "token",
        data: { content: "A", tokenCount: 1, tps: 1 },
      });
      emit("session-b", 22, {
        event: "token",
        data: { content: "B", tokenCount: 1, tps: 1 },
      });
    });

    let stopped = "ignored";
    await act(async () => {
      stopped = await result.current.stopStream("session-a");
    });

    expect(stopped).toBe("stopped");
    expect(mocks.invoke).toHaveBeenCalledWith("cancel_agent_request", {
      sessionId: "session-a", generation: 11,
    });
    expect(agentStreamManager.getSnapshot("session-a")?.isStreaming).toBe(false);
    expect(agentStreamManager.getSnapshot("session-b")?.isStreaming).toBe(true);
    expect(agentStreamManager.getSnapshot("session-b")?.currentContent).toBe("B");

    await act(async () => {
      stopped = await result.current.stopStream("unknown-session");
    });
    expect(stopped).toBe("ignored");
    expect(agentStreamManager.getSnapshot("session-b")?.currentContent).toBe("B");
  });

  it("isole le débordement ancien des événements précoces du run courant", async () => {
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
      for (let index = 0; index < 65; index += 1) {
        emit("same-session", 11, {
          event: "token",
          data: { content: `old-${index}`, tokenCount: 1, tps: 1 },
        });
      }
      emit("same-session", 22, {
        event: "token",
        data: { content: "new-1", tokenCount: 1, tps: 1 },
      });
      emit("same-session", 22, {
        event: "token",
        data: { content: "new-2", tokenCount: 1, tps: 1 },
      });
      second.resolve(admission(22));
      await secondRun;
    });

    expect(agentStreamManager.getSnapshot("same-session")?.currentContent).toBe("new-1new-2");
    expect(mocks.invoke.mock.calls.filter(([command]) => command === "cancel_agent_request"))
      .toEqual([]);

    await act(async () => {
      first.resolve(admission(11));
      await firstRun;
    });
    expect(mocks.invoke.mock.calls.filter(([command]) => command === "cancel_agent_request"))
      .toEqual([["cancel_agent_request", { sessionId: "same-session", generation: 11 }]]);
  });

  it("conserve l'état visible si l'annulation backend échoue", async () => {
    mocks.invoke.mockImplementation((command: string) => {
      if (command === "chat_stream") return Promise.resolve(admission(71));
      if (command === "cancel_agent_request") return Promise.reject(new Error("internal"));
      return Promise.resolve(undefined);
    });
    const { result } = renderHook(() => useAgentStream());
    await act(async () => {
      await start(result, "visible");
      emit("same-session", 71, {
        event: "token",
        data: { content: "encore visible", tokenCount: 1, tps: 1 },
      });
    });

    let stopped = "stopped";
    await act(async () => {
      stopped = await result.current.stopStream("same-session");
    });

    expect(stopped).toBe("ignored");
    expect(agentStreamManager.getSnapshot("same-session")?.isStreaming).toBe(true);
    expect(agentStreamManager.getSnapshot("same-session")?.currentContent)
      .toBe("encore visible");
  });

  it("préserve les buckets précoces au démontage du propriétaire", async () => {
    const pending = deferred<ChatStreamAdmission>();
    mocks.invoke.mockImplementation((command: string) => {
      if (command === "chat_stream") return pending.promise;
      return Promise.resolve(undefined);
    });
    const { result, unmount } = renderHook(() => useAgentStream());
    void start(result, "unmount");
    await vi.waitFor(() => expect(mocks.invoke).toHaveBeenCalledWith(
      "chat_stream", expect.anything(),
    ));
    emit("same-session", 81, {
      event: "token",
      data: { content: "précoce", tokenCount: 1, tps: 1 },
    });
    expect(records.get("same-session")?.pendingAdmissionBuckets).toHaveLength(1);

    unmount();

    expect(records.get("same-session")?.pendingAdmissionBuckets).toHaveLength(1);
  });

  it("mémorise un stop pendant l'admission puis annule sa génération exacte", async () => {
    const first = deferred<ChatStreamAdmission>();
    let chatCalls = 0;
    mocks.invoke.mockImplementation((command: string) => {
      if (command === "chat_stream") {
        chatCalls += 1;
        return chatCalls === 1 ? first.promise : Promise.resolve(admission(62));
      }
      return Promise.resolve(undefined);
    });
    const { result } = renderHook(() => useAgentStream());
    let firstRun!: Promise<void>;

    await act(async () => {
      firstRun = start(result, "stopped-before-admission");
      await vi.waitFor(() => expect(chatCalls).toBe(1));
      expect(await result.current.stopStream("same-session")).toBe("stopping");
      emit("same-session", 61, {
        event: "token",
        data: { content: "never replayed", tokenCount: 1, tps: 1 },
      });
      first.resolve(admission(61));
      await firstRun;
    });

    expect(mocks.invoke.mock.calls.filter(([command]) => command === "cancel_agent_request"))
      .toEqual([["cancel_agent_request", { sessionId: "same-session", generation: 61 }]]);
    expect(agentStreamManager.getSnapshot("same-session")?.currentContent).toBe("");
    expect(agentStreamManager.getSnapshot("same-session")?.isStreaming).toBe(false);

    await act(async () => {
      await start(result, "next-run");
      emit("same-session", 62, {
        event: "token", data: { content: "B", tokenCount: 1, tps: 1 },
      });
    });
    expect(agentStreamManager.getSnapshot("same-session")?.currentContent).toBe("B");
  });

  it("nettoie un stop en attente quand l'admission est refusée", async () => {
    const pending = deferred<ChatStreamAdmission>();
    mocks.invoke.mockImplementation((command: string) => {
      if (command === "chat_stream") return pending.promise;
      return Promise.resolve(undefined);
    });
    const { result } = renderHook(() => useAgentStream());
    let running!: Promise<void>;
    await act(async () => {
      running = start(result, "rejected-after-stop");
      await vi.waitFor(() => expect(mocks.invoke).toHaveBeenCalledWith(
        "chat_stream", expect.anything(),
      ));
      expect(await result.current.stopStream("same-session")).toBe("stopping");
      pending.reject(new Error("internal"));
      await running;
    });

    const snapshot = agentStreamManager.getSnapshot("same-session");
    expect(snapshot?.completed).toBe(true);
    expect(snapshot?.error).toBeTruthy();
    expect(records.get("same-session")?.pendingAdmissionBuckets).toHaveLength(0);
    expect(mocks.invoke.mock.calls.filter(([command]) => command === "cancel_agent_request"))
      .toEqual([]);
  });

  it("laisse un stream admis adoptable après un démontage puis l'arrête exactement", async () => {
    mocks.invoke.mockImplementation((command: string) => {
      if (command === "chat_stream") return Promise.resolve(admission(71));
      return Promise.resolve(undefined);
    });
    const firstHook = renderHook(() => useAgentStream());
    await act(async () => {
      await start(firstHook.result, "admitted-before-unmount");
      emit("same-session", 71, {
        event: "token", data: { content: "continues", tokenCount: 1, tps: 1 },
      });
    });

    firstHook.unmount();
    expect(agentStreamManager.getSnapshot("same-session")?.isStreaming).toBe(true);
    expect(agentStreamManager.getSnapshot("same-session")?.currentContent).toBe("continues");

    const remounted = renderHook(() => useAgentStream());
    await act(async () => {
      expect(await remounted.result.current.stopStream("same-session")).toBe("stopped");
    });
    expect(mocks.invoke).toHaveBeenCalledWith("cancel_agent_request", {
      sessionId: "same-session", generation: 71,
    });
  });

  it("termine une admission globale après démontage et la rend adoptable", async () => {
    const pending = deferred<ChatStreamAdmission>();
    mocks.invoke.mockImplementation((command: string) => {
      if (command === "chat_stream") return pending.promise;
      return Promise.resolve(undefined);
    });
    const firstHook = renderHook(() => useAgentStream());
    let running!: Promise<void>;
    await act(async () => {
      running = start(firstHook.result, "pending-unmount");
      await vi.waitFor(() => expect(mocks.invoke).toHaveBeenCalledWith(
        "chat_stream", expect.anything(),
      ));
      emit("same-session", 81, {
        event: "token", data: { content: "early survives", tokenCount: 1, tps: 1 },
      });
    });
    firstHook.unmount();

    await act(async () => {
      pending.resolve(admission(81));
      await running;
    });
    expect(agentStreamManager.getSnapshot("same-session")?.isStreaming).toBe(true);
    expect(agentStreamManager.getSnapshot("same-session")?.currentContent).toBe("early survives");
    expect(mocks.invoke.mock.calls.filter(([command]) => command === "cancel_agent_request"))
      .toEqual([]);

    const remounted = renderHook(() => useAgentStream());
    await act(async () => {
      expect(await remounted.result.current.stopStream("same-session")).toBe("stopped");
    });
    expect(mocks.invoke).toHaveBeenCalledWith("cancel_agent_request", {
      sessionId: "same-session", generation: 81,
    });
  });

  it("libère un stop admis rejeté après démontage puis permet de le reprendre", async () => {
    const firstCancel = deferred<void>();
    let cancelCalls = 0;
    mocks.invoke.mockImplementation((command: string) => {
      if (command === "chat_stream") return Promise.resolve(admission(91));
      if (command === "cancel_agent_request") {
        cancelCalls += 1;
        return cancelCalls === 1 ? firstCancel.promise : Promise.resolve(undefined);
      }
      return Promise.resolve(undefined);
    });
    const mounted = renderHook(() => useAgentStream());
    await act(async () => { await start(mounted.result, "admitted-stop-reject"); });

    let rejectedStop!: Promise<string>;
    await act(async () => {
      rejectedStop = mounted.result.current.stopStream("same-session");
      await vi.waitFor(() => expect(cancelCalls).toBe(1));
    });
    mounted.unmount();
    await act(async () => {
      firstCancel.reject(new Error("internal"));
      expect(await rejectedStop).toBe("ignored");
    });

    const remounted = renderHook(() => useAgentStream());
    await act(async () => {
      expect(await remounted.result.current.stopStream("same-session")).toBe("stopped");
    });
    expect(mocks.invoke.mock.calls.filter(([command]) => command === "cancel_agent_request"))
      .toEqual([
        ["cancel_agent_request", { sessionId: "same-session", generation: 91 }],
        ["cancel_agent_request", { sessionId: "same-session", generation: 91 }],
      ]);
  });

  it("libère un stop différé rejeté après démontage puis permet de le reprendre", async () => {
    const admissionPending = deferred<ChatStreamAdmission>();
    const firstCancel = deferred<void>();
    let cancelCalls = 0;
    mocks.invoke.mockImplementation((command: string) => {
      if (command === "chat_stream") return admissionPending.promise;
      if (command === "cancel_agent_request") {
        cancelCalls += 1;
        return cancelCalls === 1 ? firstCancel.promise : Promise.resolve(undefined);
      }
      return Promise.resolve(undefined);
    });
    const mounted = renderHook(() => useAgentStream());
    let running!: Promise<void>;
    await act(async () => {
      running = start(mounted.result, "deferred-stop-reject");
      await vi.waitFor(() => expect(mocks.invoke).toHaveBeenCalledWith(
        "chat_stream", expect.anything(),
      ));
      expect(await mounted.result.current.stopStream("same-session")).toBe("stopping");
    });
    mounted.unmount();
    await act(async () => {
      admissionPending.resolve(admission(92));
      await vi.waitFor(() => expect(cancelCalls).toBe(1));
      firstCancel.reject(new Error("internal"));
      await running;
    });

    const remounted = renderHook(() => useAgentStream());
    await act(async () => {
      expect(await remounted.result.current.stopStream("same-session")).toBe("stopped");
    });
    expect(mocks.invoke.mock.calls.filter(([command]) => command === "cancel_agent_request"))
      .toEqual([
        ["cancel_agent_request", { sessionId: "same-session", generation: 92 }],
        ["cancel_agent_request", { sessionId: "same-session", generation: 92 }],
      ]);
  });

  it("refuse un second envoi tant que l'admission Rust est en attente", async () => {
    const pending = deferred<ChatStreamAdmission>();
    mocks.invoke.mockImplementation((command: string) => {
      if (command === "chat_stream") return pending.promise;
      return Promise.resolve(command === "queue_agent_message");
    });
    const { result } = renderHook(() => useAgentStream());
    let running!: Promise<void>;
    await act(async () => {
      running = start(result, "deferred-with-queue");
      await vi.waitFor(() => expect(mocks.invoke).toHaveBeenCalledWith(
        "chat_stream", expect.anything(),
      ));
      expect(await result.current.queueStreamMessage(
        "same-session",
        { content: "queued", files: [], skills: [] },
        message("queued"),
      )).toBe("unavailable");
      expect(agentStreamManager.getSnapshot("same-session")?.queuedUserMessages)
        .toHaveLength(0);
      expect(await result.current.stopStream("same-session")).toBe("stopping");
      pending.resolve(admission(93));
      await running;
    });

    expect(agentStreamManager.getSnapshot("same-session")?.queuedUserMessages).toEqual([]);
    expect(mocks.invoke.mock.calls.filter(([command]) => command === "queue_agent_message"))
      .toEqual([]);
  });

  it("ne fabrique pas de file pendant l'admission du run suivant", async () => {
    const firstAdmission = deferred<ChatStreamAdmission>();
    const secondAdmission = deferred<ChatStreamAdmission>();
    const firstCancel = deferred<void>();
    let chatCalls = 0;
    mocks.invoke.mockImplementation((command: string) => {
      if (command === "chat_stream") {
        chatCalls += 1;
        return chatCalls === 1 ? firstAdmission.promise : secondAdmission.promise;
      }
      if (command === "cancel_agent_request") return firstCancel.promise;
      return Promise.resolve(undefined);
    });
    const { result } = renderHook(() => useAgentStream());
    let firstRun!: Promise<void>;
    let secondRun!: Promise<void>;
    await act(async () => {
      firstRun = start(result, "run-a");
      await vi.waitFor(() => expect(chatCalls).toBe(1));
      expect(await result.current.stopStream("same-session")).toBe("stopping");
      firstAdmission.resolve(admission(94));
      await vi.waitFor(() => expect(mocks.invoke).toHaveBeenCalledWith(
        "cancel_agent_request", { sessionId: "same-session", generation: 94 },
      ));
      secondRun = start(result, "run-b");
      await vi.waitFor(() => expect(chatCalls).toBe(2));
      expect(await result.current.queueStreamMessage(
        "same-session",
        { content: "queued-b", files: [], skills: [] },
        message("queued-b"),
      )).toBe("unavailable");
      firstCancel.resolve();
      await firstRun;
    });

    expect(agentStreamManager.getSnapshot("same-session")?.queuedUserMessages)
      .toEqual([]);
    secondAdmission.reject(new Error("test cleanup"));
    await act(async () => { await secondRun; });
  });

  it("refuse l'envoi remount pendant un stop admis puis reste terminal au succès", async () => {
    const cancel = deferred<void>();
    mocks.invoke.mockImplementation((command: string) => {
      if (command === "chat_stream") return Promise.resolve(admission(101));
      if (command === "cancel_agent_request") return cancel.promise;
      return Promise.resolve(true);
    });
    const mounted = renderHook(() => useAgentStream());
    await act(async () => { await start(mounted.result, "admitted-success"); });
    let stopping!: Promise<string>;
    act(() => { stopping = mounted.result.current.stopStream("same-session"); });
    await vi.waitFor(() => expect(mocks.invoke).toHaveBeenCalledWith(
      "cancel_agent_request", { sessionId: "same-session", generation: 101 },
    ));
    mounted.unmount();

    const remounted = renderHook(() => useAgentStream());
    expect(await remounted.result.current.queueStreamMessage(
      "same-session", { content: "blocked", files: [], skills: [] }, message("blocked"),
    )).toBe("stopping");
    expect(agentStreamManager.getSnapshot("same-session")?.queuedUserMessages).toEqual([]);
    await act(async () => { cancel.resolve(); await stopping; });
    expect(await remounted.result.current.queueStreamMessage(
      "same-session", { content: "terminal", files: [], skills: [] }, message("terminal"),
    )).toBe("start-new");
  });

  it("refuse l'envoi remount pendant un stop admis puis l'accepte après rejet", async () => {
    const cancel = deferred<void>();
    mocks.invoke.mockImplementation((command: string) => {
      if (command === "chat_stream") return Promise.resolve(admission(102));
      if (command === "cancel_agent_request") return cancel.promise;
      if (command === "queue_agent_message") return Promise.resolve(true);
      return Promise.resolve(undefined);
    });
    const mounted = renderHook(() => useAgentStream());
    await act(async () => { await start(mounted.result, "admitted-reject"); });
    let stopping!: Promise<string>;
    act(() => { stopping = mounted.result.current.stopStream("same-session"); });
    await vi.waitFor(() => expect(mocks.invoke).toHaveBeenCalledWith(
      "cancel_agent_request", { sessionId: "same-session", generation: 102 },
    ));
    mounted.unmount();
    const remounted = renderHook(() => useAgentStream());
    expect(await remounted.result.current.queueStreamMessage(
      "same-session", { content: "blocked", files: [], skills: [] }, message("blocked"),
    )).toBe("stopping");
    await act(async () => { cancel.reject(new Error("internal")); await stopping; });

    expect(await remounted.result.current.queueStreamMessage(
      "same-session", { content: "accepted", files: [], skills: [] }, message("accepted"),
    )).toBe("queued");
    expect(mocks.invoke).toHaveBeenCalledWith("queue_agent_message", {
      sessionId: "same-session", generation: 102,
      input: { content: "accepted", files: [], skills: [] },
    });
  });

  it("refuse l'envoi remount pendant un stop différé puis reste terminal au succès", async () => {
    const pending = deferred<ChatStreamAdmission>();
    const cancel = deferred<void>();
    mocks.invoke.mockImplementation((command: string) => {
      if (command === "chat_stream") return pending.promise;
      if (command === "cancel_agent_request") return cancel.promise;
      return Promise.resolve(undefined);
    });
    const mounted = renderHook(() => useAgentStream());
    let running!: Promise<void>;
    await act(async () => {
      running = start(mounted.result, "deferred-success");
      await vi.waitFor(() => expect(mocks.invoke).toHaveBeenCalledWith(
        "chat_stream", expect.anything(),
      ));
      expect(await mounted.result.current.stopStream("same-session")).toBe("stopping");
    });
    mounted.unmount();
    pending.resolve(admission(103));
    await vi.waitFor(() => expect(mocks.invoke).toHaveBeenCalledWith(
      "cancel_agent_request", { sessionId: "same-session", generation: 103 },
    ));
    const remounted = renderHook(() => useAgentStream());
    expect(await remounted.result.current.queueStreamMessage(
      "same-session", { content: "blocked", files: [], skills: [] }, message("blocked"),
    )).toBe("stopping");
    await act(async () => { cancel.resolve(); await running; });
    expect(await remounted.result.current.queueStreamMessage(
      "same-session", { content: "terminal", files: [], skills: [] }, message("terminal"),
    )).toBe("start-new");
  });

  it("refuse l'envoi remount pendant un stop différé puis l'accepte après rejet", async () => {
    const pending = deferred<ChatStreamAdmission>();
    const cancel = deferred<void>();
    mocks.invoke.mockImplementation((command: string) => {
      if (command === "chat_stream") return pending.promise;
      if (command === "cancel_agent_request") return cancel.promise;
      if (command === "queue_agent_message") return Promise.resolve(true);
      return Promise.resolve(undefined);
    });
    const mounted = renderHook(() => useAgentStream());
    let running!: Promise<void>;
    await act(async () => {
      running = start(mounted.result, "deferred-reject");
      await vi.waitFor(() => expect(mocks.invoke).toHaveBeenCalledWith(
        "chat_stream", expect.anything(),
      ));
      expect(await mounted.result.current.stopStream("same-session")).toBe("stopping");
    });
    mounted.unmount();
    pending.resolve(admission(104));
    await vi.waitFor(() => expect(mocks.invoke).toHaveBeenCalledWith(
      "cancel_agent_request", { sessionId: "same-session", generation: 104 },
    ));
    const remounted = renderHook(() => useAgentStream());
    expect(await remounted.result.current.queueStreamMessage(
      "same-session", { content: "blocked", files: [], skills: [] }, message("blocked"),
    )).toBe("stopping");
    await act(async () => { cancel.reject(new Error("internal")); await running; });

    expect(await remounted.result.current.queueStreamMessage(
      "same-session", { content: "accepted", files: [], skills: [] }, message("accepted"),
    )).toBe("queued");
    expect(mocks.invoke).toHaveBeenCalledWith("queue_agent_message", {
      sessionId: "same-session", generation: 104,
      input: { content: "accepted", files: [], skills: [] },
    });
  });
});
