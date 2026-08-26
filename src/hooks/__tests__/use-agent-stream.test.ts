import { act, renderHook } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { useAgentStream } from "../use-agent-stream";
import type { AgentMessage } from "@/types/agent";
import type { NewUserTurnInput, TurnStart } from "@/types/agent-turn.generated";

const ADMISSION = {
  generation: 42,
  turnId: "00000000-0000-4000-8000-000000000001",
  userMessageId: "00000000-0000-4000-8000-000000000002",
  assistantMessageId: "00000000-0000-4000-8000-000000000003",
};

const mocks = vi.hoisted(() => ({
  invoke: vi.fn(), startSession: vi.fn(), failSession: vi.fn(), stopSession: vi.fn(),
  setSessionGeneration: vi.fn(), reconcileTurnAdmission: vi.fn(), subscribe: vi.fn(),
  getSnapshot: vi.fn(), isStreaming: vi.fn(), queueUserMessage: vi.fn(),
  removeQueuedUserMessage: vi.fn(), showToast: vi.fn(),
  discardPendingAdmission: vi.fn(), ownsRun: vi.fn(), ownsOwner: vi.fn(),
  matchesRun: vi.fn(), getDeferredStop: vi.fn(), adoptOwner: vi.fn(),
  getOwnedRunState: vi.fn(),
  claimStop: vi.fn(), releaseStop: vi.fn(), completeStop: vi.fn(),
  releaseOwner: vi.fn(), isOwnerStreaming: vi.fn(),
  runs: new Map<string, { owner: symbol; id: number }>(),
  generations: new Map<string, number>(),
  awaitPendingReasoning: vi.fn(),
}));

vi.mock("@tauri-apps/api/core", () => ({ invoke: mocks.invoke }));
vi.mock("@/lib/toast-emitter", () => ({ showToast: mocks.showToast }));
vi.mock("../session-reasoning-mutation", () => ({
  awaitPendingReasoning: mocks.awaitPendingReasoning,
}));
vi.mock("../agent-stream-manager", () => ({
  agentStreamManager: {
    startSession: mocks.startSession, failSession: mocks.failSession,
    stopSession: mocks.stopSession, setSessionGeneration: mocks.setSessionGeneration,
    reconcileTurnAdmission: mocks.reconcileTurnAdmission, subscribe: mocks.subscribe,
    getSnapshot: mocks.getSnapshot, isStreaming: mocks.isStreaming,
    queueUserMessage: mocks.queueUserMessage,
    removeQueuedUserMessage: mocks.removeQueuedUserMessage,
    discardPendingAdmission: mocks.discardPendingAdmission,
    ownsRun: mocks.ownsRun, matchesRun: mocks.matchesRun,
    getDeferredStop: mocks.getDeferredStop,
    ownsOwner: mocks.ownsOwner, adoptOwner: mocks.adoptOwner,
    getOwnedRunState: mocks.getOwnedRunState,
    claimStop: mocks.claimStop, releaseStop: mocks.releaseStop,
    completeStop: mocks.completeStop, releaseOwner: mocks.releaseOwner,
    isOwnerStreaming: mocks.isOwnerStreaming,
  },
}));

function input(content: string): NewUserTurnInput {
  return { content, files: [], skills: [] };
}

function turn(content: string): Extract<TurnStart, { type: "new" }> {
  return { type: "new", input: input(content) };
}

function userMessage(content: string, id = "optimistic"): AgentMessage {
  return { id, role: "user", content, files: [], timestamp: "2026-08-26T10:00:00Z" };
}

describe("useAgentStream", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mocks.invoke.mockReset().mockResolvedValue(ADMISSION);
    mocks.awaitPendingReasoning.mockResolvedValue(undefined);
    mocks.runs.clear();
    mocks.generations.clear();
    mocks.startSession.mockImplementation((sessionId: string, ...args: unknown[]) => {
      mocks.runs.set(sessionId, args[4] as { owner: symbol; id: number });
      return Promise.resolve();
    });
    mocks.ownsRun.mockImplementation((
      sessionId: string,
      run: { owner: symbol; id: number },
    ) => mocks.runs.get(sessionId) === run);
    mocks.matchesRun.mockImplementation((
      sessionId: string,
      run: { owner: symbol; id: number },
    ) => mocks.runs.get(sessionId) === run);
    mocks.getDeferredStop.mockReturnValue(null);
    mocks.ownsOwner.mockImplementation(
      (sessionId: string, owner: symbol) => mocks.runs.get(sessionId)?.owner === owner,
    );
    mocks.adoptOwner.mockImplementation(
      (sessionId: string) => mocks.runs.has(sessionId),
    );
    mocks.getOwnedRunState.mockImplementation((sessionId: string, owner: symbol) => {
      const run = mocks.runs.get(sessionId);
      if (run?.owner !== owner) return { kind: "terminal" };
      const generation = mocks.generations.get(sessionId);
      return generation === undefined
        ? { kind: "pendingAdmission", runId: run.id }
        : { kind: "active", runId: run.id, generation };
    });
    mocks.setSessionGeneration.mockImplementation((sessionId: string, generation: number) => {
      mocks.generations.set(sessionId, generation);
      return "accepted";
    });
    mocks.claimStop.mockImplementation(
      (sessionId: string, owner: symbol) => mocks.runs.get(sessionId)?.owner === owner
        && mocks.generations.has(sessionId)
        ? {
          kind: "ready", token: Symbol("test-stop"),
          runId: mocks.runs.get(sessionId)?.id,
          generation: mocks.generations.get(sessionId),
        } : null,
    );
    mocks.completeStop.mockImplementation((
      sessionId: string,
      claim: { runId: number; generation: number },
    ) => {
      if (mocks.runs.get(sessionId)?.id !== claim.runId
          || mocks.generations.get(sessionId) !== claim.generation) return false;
      mocks.stopSession(sessionId, claim.generation);
      mocks.runs.delete(sessionId);
      mocks.generations.delete(sessionId);
      return true;
    });
    mocks.releaseOwner.mockImplementation(() => undefined);
    mocks.queueUserMessage.mockReturnValue(true);
  });

  it("envoie une seule intention générée sans historique", async () => {
    const message = userMessage("Lis ceci");
    const next = turn("Lis ceci");
    next.input.files = [{
      name: "note.txt", path: "/tmp/note.txt", mime_type: "text/plain", size: 4,
      access_grant: "grant-local",
    }];
    const { result } = renderHook(() => useAgentStream());

    await act(() => result.current.startStream(
      "session-1", "model", "provider", next, false,
      { displayMessages: [message], baseTokenCount: 0 },
    ));

    expect(mocks.invoke).toHaveBeenCalledWith("chat_stream", { request: {
      sessionId: "session-1", model: "model", provider: "provider", turn: next,
      workingDir: null, permissionMode: null, planMode: null,
    } });
  });

  it("adopte la génération et les trois identifiants Rust", async () => {
    const message = userMessage("Question");
    const { result } = renderHook(() => useAgentStream());

    await act(() => result.current.startStream(
      "session-1", "model", "provider", turn("Question"), false,
      { displayMessages: [message], baseTokenCount: 0 },
      undefined, undefined, undefined, undefined, undefined, undefined, undefined,
      message.id,
    ));

    expect(mocks.setSessionGeneration).toHaveBeenCalledWith("session-1", 42);
    expect(mocks.reconcileTurnAdmission).toHaveBeenCalledWith(
      "session-1", ADMISSION, "optimistic",
    );
  });

  it("attend la persistance d'un changement de mode avant l'admission", async () => {
    let releaseMode = () => {};
    mocks.awaitPendingReasoning.mockImplementationOnce(() => new Promise<void>((resolve) => {
      releaseMode = resolve;
    }));
    const message = userMessage("Question");
    const { result } = renderHook(() => useAgentStream());

    let starting!: Promise<void>;
    await act(async () => {
      starting = result.current.startStream(
        "session-1", "model", "provider", turn("Question"), true,
        { displayMessages: [message], baseTokenCount: 0 },
      );
      await Promise.resolve();
    });
    expect(mocks.invoke).not.toHaveBeenCalled();

    await act(async () => {
      releaseMode();
      await starting;
    });
    expect(mocks.awaitPendingReasoning).toHaveBeenCalledWith("session-1");
    expect(mocks.invoke).toHaveBeenCalledTimes(1);
  });

  it("met en file une intention unique sans historique", async () => {
    const first = userMessage("Question");
    const queued = userMessage("Suite", "queued");
    mocks.invoke.mockResolvedValueOnce(ADMISSION).mockResolvedValueOnce(true);
    const { result } = renderHook(() => useAgentStream());

    await act(async () => {
      await result.current.startStream(
        "session-1", "model", "provider", turn("Question"), false,
        { displayMessages: [first], baseTokenCount: 0 },
      );
      await result.current.queueStreamMessage("session-1", input("Suite"), queued);
    });

    expect(mocks.invoke).toHaveBeenLastCalledWith("queue_agent_message", {
      sessionId: "session-1", generation: 42, input: input("Suite"),
    });
    expect(mocks.invoke.mock.calls[1]?.[1]).not.toHaveProperty("messages");
  });

  it("refuse un second envoi pendant l'admission Rust sans perdre le brouillon", async () => {
    let resolveAdmission = (_value: typeof ADMISSION) => {};
    mocks.invoke.mockImplementationOnce(() => new Promise<typeof ADMISSION>((resolve) => {
      resolveAdmission = resolve;
    })).mockResolvedValueOnce(true);
    const first = userMessage("Question");
    const queued = userMessage("Suite", "queued");
    const { result } = renderHook(() => useAgentStream());

    let starting!: Promise<void>;
    let queuedDuringAdmission = "start-new";
    await act(async () => {
      starting = result.current.startStream(
        "session-1", "model", "provider", turn("Question"), false,
        { displayMessages: [first], baseTokenCount: 0 },
      );
      await vi.waitFor(() => expect(mocks.invoke).toHaveBeenCalledTimes(1));
      queuedDuringAdmission = await result.current.queueStreamMessage(
        "session-1", input("Suite"), queued,
      );
      resolveAdmission(ADMISSION);
      await starting;
    });
    expect(queuedDuringAdmission).toBe("unavailable");
    expect(mocks.invoke).toHaveBeenCalledTimes(1);
    expect(mocks.queueUserMessage).not.toHaveBeenCalled();

    await act(async () => {
      await Promise.resolve();
    });
    expect(mocks.invoke).toHaveBeenLastCalledWith("chat_stream", expect.anything());
  });

  it("ne mélange jamais le staging de deux sessions", async () => {
    let resolveAdmission = (_value: typeof ADMISSION) => {};
    mocks.invoke.mockImplementationOnce(() => new Promise<typeof ADMISSION>((resolve) => {
      resolveAdmission = resolve;
    }));
    const { result } = renderHook(() => useAgentStream());
    let starting!: Promise<void>;
    let crossSession = "queued";
    await act(async () => {
      starting = result.current.startStream(
        "session-1", "model", "provider", turn("Question"), false,
        { displayMessages: [userMessage("Question")], baseTokenCount: 0 },
      );
      await vi.waitFor(() => expect(mocks.invoke).toHaveBeenCalledTimes(1));
      crossSession = await result.current.queueStreamMessage(
        "session-2", input("Autre"), userMessage("Autre", "other"),
      );
      resolveAdmission(ADMISSION);
      await starting;
    });

    expect(crossSession).toBe("start-new");
    expect(mocks.queueUserMessage).not.toHaveBeenCalled();
    expect(mocks.invoke).toHaveBeenCalledTimes(1);
  });

  it("n'utilise jamais la génération de la session précédente pendant une nouvelle admission", async () => {
    const admissionB = {
      ...ADMISSION,
      generation: 84,
      turnId: "00000000-0000-4000-8000-000000000011",
      userMessageId: "00000000-0000-4000-8000-000000000012",
      assistantMessageId: "00000000-0000-4000-8000-000000000013",
    };
    let resolveAdmissionB = (_value: typeof admissionB) => {};
    mocks.invoke
      .mockResolvedValueOnce(ADMISSION)
      .mockImplementationOnce(() => new Promise<typeof admissionB>((resolve) => {
        resolveAdmissionB = resolve;
      }))
      .mockResolvedValueOnce(true);
    const { result } = renderHook(() => useAgentStream());

    await act(async () => {
      await result.current.startStream(
        "session-a", "model", "provider", turn("A"), false,
        { displayMessages: [userMessage("A")], baseTokenCount: 0 },
      );
    });
    let startingB!: Promise<void>;
    await act(async () => {
      startingB = result.current.startStream(
        "session-b", "model", "provider", turn("B"), false,
        { displayMessages: [userMessage("B")], baseTokenCount: 0 },
      );
      await vi.waitFor(() => expect(mocks.invoke).toHaveBeenCalledTimes(2));
      expect(await result.current.queueStreamMessage(
        "session-b", input("B2"), userMessage("B2", "queued-b"),
      )).toBe("unavailable");
      resolveAdmissionB(admissionB);
      await startingB;
    });

    expect(mocks.invoke.mock.calls.filter(([command]) => command === "queue_agent_message"))
      .toEqual([]);
  });

  it("préserve l'admission globale au démontage sans relancer un stream", async () => {
    let resolveAdmission = (_value: typeof ADMISSION) => {};
    mocks.invoke.mockImplementationOnce(() => new Promise<typeof ADMISSION>((resolve) => {
      resolveAdmission = resolve;
    }));
    const { result, unmount } = renderHook(() => useAgentStream());
    let starting!: Promise<void>;
    await act(async () => {
      starting = result.current.startStream(
        "session-1", "model", "provider", turn("Question"), false,
        { displayMessages: [userMessage("Question")], baseTokenCount: 0 },
      );
      await vi.waitFor(() => expect(mocks.invoke).toHaveBeenCalledTimes(1));
      await result.current.queueStreamMessage(
        "session-1", input("Suite"), userMessage("Suite", "queued-unmount"),
      );
      unmount();
    });
    expect(mocks.removeQueuedUserMessage).not.toHaveBeenCalled();
    resolveAdmission(ADMISSION);
    await starting;
    expect(mocks.invoke.mock.calls.filter(([command]) => command === "chat_stream")).toHaveLength(1);
    expect(mocks.invoke.mock.calls.filter(([command]) => command === "cancel_agent_request"))
      .toEqual([]);
  });

  it("annule avec la génération Rust active", async () => {
    const message = userMessage("Question");
    const { result } = renderHook(() => useAgentStream());
    await act(async () => {
      await result.current.startStream(
        "session-1", "model", "provider", turn("Question"), false,
        { displayMessages: [message], baseTokenCount: 0 },
      );
      await result.current.stopStream("session-1");
    });
    expect(mocks.stopSession).toHaveBeenCalledWith("session-1", 42);
  });

  it("arrête A avec sa génération même après le démarrage de B", async () => {
    const admissionB = { ...ADMISSION, generation: 84 };
    mocks.invoke.mockResolvedValueOnce(ADMISSION).mockResolvedValueOnce(admissionB);
    const { result } = renderHook(() => useAgentStream());
    await act(async () => {
      await result.current.startStream(
        "session-a", "model", "provider", turn("A"), false,
        { displayMessages: [userMessage("A")], baseTokenCount: 0 },
      );
      await result.current.startStream(
        "session-b", "model", "provider", turn("B"), false,
        { displayMessages: [userMessage("B")], baseTokenCount: 0 },
      );
      await result.current.stopStream("session-a");
    });

    expect(mocks.stopSession).toHaveBeenCalledWith("session-a", 42);
    expect(mocks.invoke).toHaveBeenCalledWith("cancel_agent_request", {
      sessionId: "session-a", generation: 42,
    });
  });

  it("traduit un refus de démarrage", async () => {
    mocks.invoke.mockRejectedValueOnce("app-shutting-down");
    const message = userMessage("Question");
    const { result } = renderHook(() => useAgentStream());
    await act(() => result.current.startStream(
      "session-1", "model", "provider", turn("Question"), false,
      { displayMessages: [message], baseTokenCount: 0 },
    ));
    expect(mocks.failSession).toHaveBeenCalledWith(
      "session-1", "Beaver is closing. Try again after restarting the application.",
    );
  });
});
