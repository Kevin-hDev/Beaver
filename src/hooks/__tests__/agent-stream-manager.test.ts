import { beforeEach, describe, expect, it, vi } from "vitest";
import { agentStreamManager } from "../agent-stream-manager";
import { records } from "../agent-stream-records";
import type { AgentMessage, StreamEvent } from "@/types/agent";
import type { AgentMessageView } from "@/types/agent-session.generated";

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

function message(
  id: string,
  role: AgentMessage["role"],
  content: string,
): AgentMessage & AgentMessageView {
  return {
    id, turn_id: `turn-${id}`, role, content,
    timestamp: "2026-06-24T10:00:00Z", files: [], tokens: 0,
    reasoning_replay_status: "unavailable",
  };
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
    expect(mocks.invoke).not.toHaveBeenCalled();
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

  it("réconcilie le user et l'assistant avec les IDs Rust seulement sur la génération active", async () => {
    const turn = {
      turnId: "00000000-0000-4000-8000-000000000001",
      userMessageId: "00000000-0000-4000-8000-000000000002",
      assistantMessageId: "00000000-0000-4000-8000-000000000003",
    };
    await agentStreamManager.startSession(
      "s1", [message("optimistic", "user", "Question")], 10,
    );
    agentStreamManager.setSessionGeneration("s1", 7);
    agentStreamManager.reconcileTurnAdmission("s1", { generation: 7, ...turn }, "optimistic");
    emit("s1", { event: "token", data: { content: "Réponse", tokenCount: 1, tps: 1 } }, 7);

    emit("s1", { event: "turnCommitted", data: turn }, 6);
    expect(agentStreamManager.getSnapshot("s1")?.messages).toHaveLength(1);
    emit("s1", { event: "turnCommitted", data: turn }, 7);

    const messages = agentStreamManager.getSnapshot("s1")?.messages ?? [];
    expect(messages[0]).toEqual(expect.objectContaining({
      id: turn.userMessageId, turn_id: turn.turnId,
    }));
    expect(messages[1]).toEqual(expect.objectContaining({
      id: turn.assistantMessageId, turn_id: turn.turnId, content: "Réponse",
    }));
  });

  it("replie la réponse terminée dans l'anneau avant de vider le stream", async () => {
    const turn = {
      turnId: "00000000-0000-4000-8000-000000000011",
      userMessageId: "00000000-0000-4000-8000-000000000012",
      assistantMessageId: "00000000-0000-4000-8000-000000000013",
    };
    await agentStreamManager.startSession(
      "context", [message("optimistic", "user", "Question")], 22,
    );
    const record = records.get("context");
    if (!record) throw new Error("missing stream record");
    record.state.contextUsageBuckets = {
      messages: 22,
      systemTools: 0,
      mcpConnectors: 0,
      skills: 0,
      memory: 0,
      metaContext: 0,
      systemPrompt: 0,
    };
    agentStreamManager.setSessionGeneration("context", 7);
    agentStreamManager.reconcileTurnAdmission(
      "context", { generation: 7, ...turn }, "optimistic",
    );
    emit("context", {
      event: "token", data: { content: "a".repeat(80), tokenCount: 20, tps: 1 },
    }, 7);
    emit("context", { event: "turnEnd", data: {} }, 7);
    emit("context", { event: "turnCommitted", data: turn }, 7);

    const after = agentStreamManager.getSnapshot("context");
    expect(after?.contextUsageBuckets?.messages).toBe(42);
    expect(after?.completedSegments).toEqual([]);
  });

  it("met en file une intention pendant l'attente de la génération Rust", async () => {
    await agentStreamManager.startSession(
      "s1", [message("u1", "user", "Question")], 10, "chat", true,
    );

    const queued = agentStreamManager.queueUserMessage(
      "s1", message("u2", "user", "Suite"),
    );

    expect(queued).toBe(true);
    expect(agentStreamManager.getSnapshot("s1")?.queuedUserMessages).toEqual([
      expect.objectContaining({ id: "u2", content: "Suite" }),
    ]);
  });

  it("rejoue dans l'ordre les événements arrivés avant la résolution IPC", async () => {
    const turn = {
      turnId: "00000000-0000-4000-8000-000000000021",
      userMessageId: "00000000-0000-4000-8000-000000000022",
      assistantMessageId: "00000000-0000-4000-8000-000000000023",
    };
    await agentStreamManager.startSession(
      "early", [message("optimistic", "user", "Question")], 0, "chat", true,
    );

    emit("early", { event: "turnAdmitted", data: turn }, 91);
    emit("early", {
      event: "token", data: { content: "précoce", tokenCount: 1, tps: 1 },
    }, 91);
    emit("early", {
      event: "done",
      data: {
        evalCount: 1, evalDurationNs: 0, finalTps: 1,
        promptTokens: 1, contextTokens: 2,
      },
    }, 91);

    expect(agentStreamManager.getSnapshot("early")?.currentContent).toBe("");
    agentStreamManager.reconcileTurnAdmission(
      "early", { generation: 91, ...turn }, "optimistic",
    );
    agentStreamManager.setSessionGeneration("early", 91);

    const after = agentStreamManager.getSnapshot("early");
    expect(after?.completed).toBe(true);
    expect(after?.messages[0]?.id).toBe(turn.userMessageId);
    const messages = after?.messages ?? [];
    expect(messages[messages.length - 1]?.content).toBe("précoce");
  });

  it("rejoue une erreur précoce sans exposer son contenu brut", async () => {
    await agentStreamManager.startSession(
      "early-error", [message("u1", "user", "Question")], 0, "chat", true,
    );
    emit("early-error", {
      event: "error",
      data: { message: "/private/provider/secret", isConnection: false },
    }, 92);

    agentStreamManager.setSessionGeneration("early-error", 92);

    const after = agentStreamManager.getSnapshot("early-error");
    expect(after?.completed).toBe(true);
    expect(after?.error).toBeTruthy();
    expect(after?.error).not.toContain("/private/provider/secret");
  });

  it("échoue proprement quand le buffer d'admission borné déborde", async () => {
    await agentStreamManager.startSession(
      "early-overflow", [message("u1", "user", "Question")], 0, "chat", true,
    );
    for (let index = 0; index < 65; index += 1) {
      emit("early-overflow", {
        event: "token",
        data: { content: `${index}`, tokenCount: 1, tps: 1 },
      }, 93);
    }

    agentStreamManager.setSessionGeneration("early-overflow", 93);

    const after = agentStreamManager.getSnapshot("early-overflow");
    expect(after?.completed).toBe(true);
    expect(after?.error).toBeTruthy();
    expect(after?.messages).toHaveLength(1);
  });

  it("ne rejoue jamais l'ancienne génération mise en quarantaine", async () => {
    await agentStreamManager.startSession(
      "renew", [message("u1", "user", "Première")], 0,
    );
    agentStreamManager.setSessionGeneration("renew", 7);
    await agentStreamManager.startSession(
      "renew", [message("u2", "user", "Seconde")], 0, "chat", true,
    );
    emit("renew", {
      event: "token", data: { content: "ancienne", tokenCount: 1, tps: 1 },
    }, 7);
    emit("renew", {
      event: "token", data: { content: "nouvelle", tokenCount: 1, tps: 1 },
    }, 8);

    agentStreamManager.setSessionGeneration("renew", 8);

    expect(agentStreamManager.getSnapshot("renew")?.currentContent).toBe("nouvelle");
  });

  it("borne les buffers précoces par génération sans évincer un bucket exploitable", async () => {
    await agentStreamManager.startSession(
      "bounded", [message("u1", "user", "Question")], 0, "chat", true,
    );
    for (let generation = 1; generation <= 40; generation += 1) {
      emit("bounded", {
        event: "token",
        data: { content: `${generation}`, tokenCount: 1, tps: 1 },
      }, generation);
    }

    expect(records.get("bounded")?.pendingAdmissionBuckets).toHaveLength(32);
  });

  it("refuse une génération écartée même après rotation de la quarantaine", async () => {
    await agentStreamManager.startSession(
      "saturated", [message("u1", "user", "Question")], 0, "chat", true,
    );
    for (let generation = 1; generation <= 80; generation += 1) {
      emit("saturated", {
        event: "token",
        data: { content: `${generation}`, tokenCount: 1, tps: 1 },
      }, generation);
    }

    expect(agentStreamManager.setSessionGeneration("saturated", 33)).toBe("rejected");
    expect(agentStreamManager.getSnapshot("saturated")?.currentContent).toBe("");
  });

  it("ignore les événements de la génération après un échec local", async () => {
    await agentStreamManager.startSession(
      "failed", [message("u1", "user", "Question")], 0,
    );
    agentStreamManager.setSessionGeneration("failed", 95);
    agentStreamManager.failSession("failed", "échec générique");

    emit("failed", {
      event: "token", data: { content: "tardif", tokenCount: 1, tps: 1 },
    }, 95);

    const after = agentStreamManager.getSnapshot("failed");
    expect(after?.isStreaming).toBe(false);
    expect(after?.currentContent).toBe("");
  });

  it("purge tous les buckets précoces après un échec local", async () => {
    await agentStreamManager.startSession(
      "failed-buckets", [message("u1", "user", "Question")], 0, "chat", true,
    );
    emit("failed-buckets", {
      event: "token", data: { content: "a", tokenCount: 1, tps: 1 },
    }, 101);
    emit("failed-buckets", {
      event: "token", data: { content: "b", tokenCount: 1, tps: 1 },
    }, 102);

    agentStreamManager.failSession("failed-buckets", "échec générique");

    expect(records.get("failed-buckets")?.pendingAdmissionBuckets).toHaveLength(0);
  });
});
