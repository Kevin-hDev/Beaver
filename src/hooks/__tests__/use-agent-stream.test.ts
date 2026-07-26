import { act, renderHook } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { useAgentStream } from "../use-agent-stream";
import type { AgentMessage } from "@/types/agent";

const mocks = vi.hoisted(() => ({
  invoke: vi.fn(),
  readFile: vi.fn(),
  startSession: vi.fn(),
  failSession: vi.fn(),
  stopSession: vi.fn(),
  setSessionGeneration: vi.fn(),
  subscribe: vi.fn(),
  getSnapshot: vi.fn(),
  isStreaming: vi.fn(),
  queueUserMessage: vi.fn(),
  removeQueuedUserMessage: vi.fn(),
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: mocks.invoke,
}));

vi.mock("@tauri-apps/plugin-fs", () => ({
  readFile: mocks.readFile,
}));

vi.mock("../agent-stream-manager", () => ({
  agentStreamManager: {
    startSession: mocks.startSession,
    failSession: mocks.failSession,
    stopSession: mocks.stopSession,
    setSessionGeneration: mocks.setSessionGeneration,
    subscribe: mocks.subscribe,
    getSnapshot: mocks.getSnapshot,
    isStreaming: mocks.isStreaming,
    queueUserMessage: mocks.queueUserMessage,
    removeQueuedUserMessage: mocks.removeQueuedUserMessage,
  },
}));

function userMessage(files: AgentMessage["files"]): AgentMessage {
  return {
    id: "m1",
    role: "user",
    content: "Tu vois l'image ?",
    files,
    timestamp: "2026-06-24T10:00:00Z",
  };
}

describe("useAgentStream", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mocks.invoke.mockResolvedValue(1);
    mocks.startSession.mockResolvedValue(undefined);
    mocks.queueUserMessage.mockReturnValue(true);
  });

  it("renvoie une image depuis son thumbnail quand le chemin n'est plus lisible", async () => {
    mocks.readFile.mockRejectedValue(new Error("missing file"));
    const message = userMessage([{
      name: "logo.png",
      path: "/tmp/logo.png",
      mime_type: "image/png",
      size: 123,
      thumbnail: "data:image/png;base64,abc123",
    }]);
    const { result } = renderHook(() => useAgentStream());

    await act(async () => {
      await result.current.startStream(
        "session-1",
        "model",
        "provider",
        [message],
        false,
        { displayMessages: [message], baseTokenCount: 0 },
      );
    });

    expect(mocks.invoke).toHaveBeenCalledWith("chat_stream", expect.objectContaining({
      messages: [expect.objectContaining({ images: ["abc123"] })],
    }));
  });

  it("envoie une image depuis son thumbnail quand elle n'a pas de chemin fichier", async () => {
    const message = userMessage([{
      name: "paste.png",
      path: "",
      mime_type: "image/png",
      size: 123,
      thumbnail: "data:image/png;base64,pasted123",
    }]);
    const { result } = renderHook(() => useAgentStream());

    await act(async () => {
      await result.current.startStream(
        "session-1",
        "model",
        "provider",
        [message],
        false,
        { displayMessages: [message], baseTokenCount: 0 },
      );
    });

    expect(mocks.readFile).not.toHaveBeenCalled();
    expect(mocks.invoke).toHaveBeenCalledWith("chat_stream", expect.objectContaining({
      messages: [expect.objectContaining({ images: ["pasted123"] })],
    }));
  });

  it("enregistre la génération retournée par le backend", async () => {
    const message = userMessage([]);
    mocks.invoke.mockResolvedValue(42);
    const { result } = renderHook(() => useAgentStream());

    await act(async () => {
      await result.current.startStream(
        "session-1",
        "model",
        "provider",
        [message],
        false,
        { displayMessages: [message], baseTokenCount: 0 },
      );
    });

    expect(mocks.setSessionGeneration).toHaveBeenCalledWith("session-1", 42);
  });

  it("stoppe localement avec la génération active", async () => {
    const message = userMessage([]);
    mocks.invoke.mockResolvedValue(42);
    const { result } = renderHook(() => useAgentStream());

    await act(async () => {
      await result.current.startStream(
        "session-1",
        "model",
        "provider",
        [message],
        false,
        { displayMessages: [message], baseTokenCount: 0 },
      );
      await result.current.stopStream("session-1");
    });

    expect(mocks.stopSession).toHaveBeenCalledWith("session-1", 42);
    expect(mocks.invoke).toHaveBeenLastCalledWith("cancel_agent_request", {
      sessionId: "session-1",
      generation: 42,
    });
  });

  it("ajoute un message au stream courant sans en démarrer un autre", async () => {
    const first = userMessage([]);
    const queued = { ...userMessage([]), id: "m2", content: "Ajoute une comparaison" };
    mocks.invoke.mockResolvedValueOnce(42).mockResolvedValueOnce(true);
    const { result } = renderHook(() => useAgentStream());

    await act(async () => {
      await result.current.startStream(
        "session-1",
        "model",
        "provider",
        [first],
        false,
        { displayMessages: [first], baseTokenCount: 0 },
      );
      await result.current.queueStreamMessage("session-1", [queued], queued);
    });

    expect(mocks.startSession).toHaveBeenCalledTimes(1);
    expect(mocks.queueUserMessage).toHaveBeenCalledWith("session-1", queued);
    expect(mocks.invoke).toHaveBeenLastCalledWith("queue_agent_message", {
      sessionId: "session-1",
      generation: 42,
      messages: [expect.objectContaining({ role: "user", content: "Ajoute une comparaison" })],
    });
    expect(mocks.stopSession).not.toHaveBeenCalled();
  });
});
