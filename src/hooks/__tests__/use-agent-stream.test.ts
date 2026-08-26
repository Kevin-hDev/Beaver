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
  discardPendingAdmission: vi.fn(),
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
    mocks.startSession.mockResolvedValue(undefined);
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

  it("borne et diffère un second envoi pendant l'admission Rust", async () => {
    let resolveAdmission = (_value: typeof ADMISSION) => {};
    mocks.invoke.mockImplementationOnce(() => new Promise<typeof ADMISSION>((resolve) => {
      resolveAdmission = resolve;
    })).mockResolvedValueOnce(true);
    const first = userMessage("Question");
    const queued = userMessage("Suite", "queued");
    const { result } = renderHook(() => useAgentStream());

    let starting!: Promise<void>;
    let queuedDuringAdmission = false;
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
    expect(queuedDuringAdmission).toBe(true);
    expect(mocks.invoke).toHaveBeenCalledTimes(2);
    expect(mocks.queueUserMessage).toHaveBeenCalledWith("session-1", queued);

    await act(async () => {
      await Promise.resolve();
    });
    expect(mocks.invoke).toHaveBeenLastCalledWith("queue_agent_message", {
      sessionId: "session-1", generation: 42, input: input("Suite"),
    });
  });

  it("ne mélange jamais le staging de deux sessions", async () => {
    let resolveAdmission = (_value: typeof ADMISSION) => {};
    mocks.invoke.mockImplementationOnce(() => new Promise<typeof ADMISSION>((resolve) => {
      resolveAdmission = resolve;
    }));
    const { result } = renderHook(() => useAgentStream());
    let starting!: Promise<void>;
    let crossSession = true;
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

    expect(crossSession).toBe(false);
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
      )).toBe(true);
      resolveAdmissionB(admissionB);
      await startingB;
    });

    expect(mocks.invoke).toHaveBeenLastCalledWith("queue_agent_message", {
      sessionId: "session-b", generation: 84, input: input("B2"),
    });
  });

  it("retire le staging transitoire au démontage sans relancer un stream", async () => {
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
    expect(mocks.removeQueuedUserMessage).toHaveBeenCalledWith(
      "session-1", "queued-unmount",
    );
    resolveAdmission(ADMISSION);
    await starting;
    expect(mocks.invoke.mock.calls.filter(([command]) => command === "chat_stream")).toHaveLength(1);
    expect(mocks.invoke).toHaveBeenCalledWith("cancel_agent_request", {
      sessionId: "session-1", generation: 42,
    });
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
