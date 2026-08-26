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
}));

vi.mock("@tauri-apps/api/core", () => ({ invoke: mocks.invoke }));
vi.mock("@/lib/toast-emitter", () => ({ showToast: mocks.showToast }));
vi.mock("../agent-stream-manager", () => ({
  agentStreamManager: {
    startSession: mocks.startSession, failSession: mocks.failSession,
    stopSession: mocks.stopSession, setSessionGeneration: mocks.setSessionGeneration,
    reconcileTurnAdmission: mocks.reconcileTurnAdmission, subscribe: mocks.subscribe,
    getSnapshot: mocks.getSnapshot, isStreaming: mocks.isStreaming,
    queueUserMessage: mocks.queueUserMessage,
    removeQueuedUserMessage: mocks.removeQueuedUserMessage,
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
    mocks.invoke.mockResolvedValue(ADMISSION);
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

    expect(mocks.invoke).toHaveBeenCalledWith("chat_stream", expect.objectContaining({
      turn: next,
    }));
    expect(mocks.invoke.mock.calls[0]?.[1]).not.toHaveProperty("messages");
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
