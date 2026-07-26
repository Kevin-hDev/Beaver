import { act, renderHook } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { useAgentStream } from "../use-agent-stream";
import type { AgentMessage } from "@/types/agent";

const mocks = vi.hoisted(() => ({
  invoke: vi.fn(),
  startSession: vi.fn(),
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: mocks.invoke,
}));

vi.mock("@tauri-apps/plugin-fs", () => ({
  readFile: vi.fn(),
}));

vi.mock("../agent-stream-manager", () => ({
  agentStreamManager: {
    startSession: mocks.startSession,
    failSession: vi.fn(),
    stopSession: vi.fn(),
    setSessionGeneration: vi.fn(),
    subscribe: vi.fn(),
    getSnapshot: vi.fn(),
    isStreaming: vi.fn(),
    queueUserMessage: vi.fn(),
    removeQueuedUserMessage: vi.fn(),
  },
}));

function userMessage(content: string): AgentMessage {
  return {
    id: crypto.randomUUID(),
    role: "user",
    content,
    files: [],
    timestamp: "2026-07-20T00:00:00Z",
  };
}

describe("useAgentStream compression", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mocks.invoke.mockResolvedValue(1);
    mocks.startSession.mockResolvedValue(undefined);
  });

  it("initialise directement le flux manuel comme une compression", async () => {
    const command = userMessage(" /compress ");
    const { result } = renderHook(() => useAgentStream());

    await act(async () => {
      await result.current.startStream(
        "session-1",
        "model",
        "provider",
        [command],
        false,
        { displayMessages: [command], baseTokenCount: 123 },
      );
    });

    expect(mocks.startSession).toHaveBeenCalledWith(
      "session-1",
      [command],
      123,
      "compression",
    );
  });

  it("conserve le flux normal quand compress fait partie d'une phrase", async () => {
    const message = userMessage("Explique la commande /compress");
    const { result } = renderHook(() => useAgentStream());

    await act(async () => {
      await result.current.startStream(
        "session-1",
        "model",
        "provider",
        [message],
        false,
        { displayMessages: [message], baseTokenCount: 123 },
      );
    });

    expect(mocks.startSession).toHaveBeenCalledWith(
      "session-1",
      [message],
      123,
      "chat",
    );
  });
});
