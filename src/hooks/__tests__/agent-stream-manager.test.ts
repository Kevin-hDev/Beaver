import { beforeEach, describe, expect, it, vi } from "vitest";
import { agentStreamManager } from "../agent-stream-manager";
import { records } from "../agent-stream-records";
import type { AgentMessage, StreamEvent } from "@/types/agent";

const mocks = vi.hoisted(() => ({
  invoke: vi.fn(),
  listen: vi.fn(),
}));

let streamHandler: ((event: { payload: { sessionId: string; generation?: number; event: StreamEvent } }) => void) | null = null;

vi.mock("@tauri-apps/api/core", () => ({
  invoke: mocks.invoke,
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: mocks.listen,
}));

function message(id: string, role: AgentMessage["role"], content: string): AgentMessage {
  return { id, role, content, timestamp: "2026-06-24T10:00:00Z", files: [] };
}

function emit(sessionId: string, event: StreamEvent, generation?: number) {
  streamHandler?.({ payload: { sessionId, event, generation } });
}

describe("agentStreamManager", () => {
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

  it("recharge la session et vide les buffers après compressionComplete", async () => {
    const reloadedMessages = [
      message("m1", "user", "Résumé de compression"),
      message("m2", "assistant", "réponse partielle"),
    ];
    mocks.invoke.mockResolvedValue({
      messages: reloadedMessages,
      accumulated_tokens: 42,
    });

    await agentStreamManager.startSession("s1", [message("u1", "user", "Question")], 10);
    emit("s1", { event: "token", data: { content: "réponse partielle", tokenCount: 3, tps: 1 } });
    emit("s1", { event: "thinking", data: { content: "raisonnement" } });
    emit("s1", { event: "toolCall", data: { name: "bash", arguments: { cmd: "pwd" } } });
    emit("s1", { event: "turnEnd", data: {} });
    emit("s1", { event: "token", data: { content: "suite", tokenCount: 4, tps: 1 } });

    const before = agentStreamManager.getSnapshot("s1");
    expect(before?.completedSegments).toHaveLength(1);
    expect(before?.currentContent).toBe("suite");

    emit("s1", { event: "compressionComplete", data: {} });

    await vi.waitFor(() => {
      expect(agentStreamManager.getSnapshot("s1")?.messages).toEqual(reloadedMessages);
    });

    const after = agentStreamManager.getSnapshot("s1");
    expect(after?.messages[1]?.content).toBe("réponse partielle");
    expect(after?.completedSegments).toEqual([]);
    expect(after?.currentContent).toBe("");
    expect(after?.currentThinking).toBe("");
    expect(after?.currentTools).toEqual([]);
    expect(after?.isStreaming).toBe(false);
    expect(mocks.invoke).toHaveBeenCalledWith("get_agent_session", { id: "s1" });
  });

  it("ignore les events tardifs d'une génération annulée", async () => {
    await agentStreamManager.startSession("s1", [message("u1", "user", "Question")], 10);
    agentStreamManager.setSessionGeneration("s1", 7);
    emit("s1", { event: "token", data: { content: "début", tokenCount: 1, tps: 1 } }, 7);

    agentStreamManager.stopSession("s1", 7);
    emit("s1", { event: "token", data: { content: " fantôme", tokenCount: 2, tps: 1 } }, 7);

    const after = agentStreamManager.getSnapshot("s1");
    expect(after?.isStreaming).toBe(false);
    expect(after?.currentContent).toBe("");
    const lastMessage = after?.messages[after.messages.length - 1];
    expect(lastMessage?.content).toBe("début");
  });

  it("accepte une nouvelle génération après l'annulation de la précédente", async () => {
    await agentStreamManager.startSession("s1", [message("u1", "user", "Question")], 10);
    agentStreamManager.setSessionGeneration("s1", 7);
    agentStreamManager.stopSession("s1", 7);

    await agentStreamManager.startSession("s1", [message("u2", "user", "Suite")], 11);
    emit("s1", { event: "token", data: { content: "nouveau", tokenCount: 1, tps: 1 } }, 8);

    const after = agentStreamManager.getSnapshot("s1");
    expect(after?.isStreaming).toBe(true);
    expect(after?.currentContent).toBe("nouveau");
  });

  it("termine le loader enfant quand le parent reçoit sa fin", async () => {
    await agentStreamManager.startSession("child", [message("u1", "user", "Mission")], 10);

    emit("parent", {
      event: "subagentCompleted",
      data: {
        subagentSessionId: "child",
        success: false,
        status: "cancelled",
        summary: "Sous-agent annulé.",
      },
    });

    expect(agentStreamManager.getSnapshot("child")?.isStreaming).toBe(false);
  });

  it("accepte tout le nouveau run backend sans génération après Stop", async () => {
    await agentStreamManager.startSession("child", [message("u1", "user", "Mission")], 10);
    agentStreamManager.stopSession("child");

    emit("child", {
      event: "sessionSnapshot",
      data: { messages: [message("u2", "user", "Correction")], tokenCount: 11 },
    });
    emit("child", { event: "token", data: { content: "réponse", tokenCount: 1, tps: 1 } });
    emit("child", {
      event: "done",
      data: {
        evalCount: 1,
        evalDurationNs: 0,
        finalTps: 1,
        promptTokens: 1,
        contextTokens: 12,
      },
    });

    const after = agentStreamManager.getSnapshot("child");
    expect(after?.isStreaming).toBe(false);
    expect(after?.completed).toBe(true);
    const lastMessage = after?.messages[after.messages.length - 1];
    expect(lastMessage?.content).toBe("réponse");
  });

  it("retire une permission résolue du snapshot global", async () => {
    await agentStreamManager.startSession("s1", [message("u1", "user", "Question")], 10);
    emit("s1", {
      event: "permissionRequest",
      data: { id: "perm-1", toolName: "bash", arguments: { command: "sleep 8" } },
    });

    expect(agentStreamManager.getSnapshot("s1")?.pendingPermissions).toHaveLength(1);

    agentStreamManager.clearPermission("perm-1");

    expect(agentStreamManager.getSnapshot("s1")?.pendingPermissions).toEqual([]);
  });

  it("accepte une génération backend après une fin normale", async () => {
    await agentStreamManager.startSession("s1", [message("u1", "user", "Question")], 10);
    agentStreamManager.setSessionGeneration("s1", 7);
    emit("s1", { event: "token", data: { content: "réponse", tokenCount: 1, tps: 1 } }, 7);
    emit("s1", {
      event: "done",
      data: {
        evalCount: 1,
        evalDurationNs: 0,
        finalTps: 1,
        promptTokens: 1,
        contextTokens: 12,
      },
    }, 7);

    emit("s1", {
      event: "sessionSnapshot",
      data: { messages: [message("u2", "user", "Synthèse")], tokenCount: 12 },
    }, 8);
    emit("s1", { event: "token", data: { content: "suite", tokenCount: 1, tps: 1 } }, 8);

    const after = agentStreamManager.getSnapshot("s1");
    expect(after?.isStreaming).toBe(true);
    expect(after?.currentContent).toBe("suite");
  });
});
