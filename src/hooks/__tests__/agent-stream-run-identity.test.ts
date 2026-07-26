import { beforeEach, describe, expect, it, vi } from "vitest";
import { agentStreamManager } from "../agent-stream-manager";
import { records } from "../agent-stream-records";
import type { AgentMessage, StreamEvent } from "@/types/agent";

const mocks = vi.hoisted(() => ({ invoke: vi.fn(), listen: vi.fn() }));

let handler: ((event: {
  payload: { sessionId: string; generation?: number; event: StreamEvent };
}) => void) | null = null;

vi.mock("@tauri-apps/api/core", () => ({ invoke: mocks.invoke }));
vi.mock("@tauri-apps/api/event", () => ({ listen: mocks.listen }));

function message(id: string, role: AgentMessage["role"], content: string): AgentMessage {
  return { id, role, content, timestamp: "2026-07-11T10:00:00Z", files: [] };
}

function emit(sessionId: string, generation: number, event: StreamEvent) {
  handler?.({ payload: { sessionId, generation, event } });
}

function done(): StreamEvent {
  return {
    event: "done",
    data: {
      evalCount: 1,
      evalDurationNs: 1,
      finalTps: 1,
      promptTokens: 1,
      contextTokens: 1,
    },
  };
}

describe("identité des runs backend", () => {
  beforeEach(() => {
    records.clear();
    vi.clearAllMocks();
    mocks.invoke.mockResolvedValue(undefined);
    vi.stubGlobal("requestAnimationFrame", vi.fn());
    vi.stubGlobal("cancelAnimationFrame", vi.fn());
    mocks.listen.mockImplementation((_name: string, listener: typeof handler) => {
      handler = listener;
      return Promise.resolve(() => {});
    });
  });

  it("rejette le snapshot et le token tardifs d'un ancien run terminé", async () => {
    await agentStreamManager.startSession("shared", [message("u1", "user", "Question")], 0);
    emit("shared", 101, {
      event: "sessionSnapshot",
      data: { messages: [message("u-old", "user", "Ancien")], tokenCount: 0 },
    });
    emit("shared", 101, { event: "token", data: { content: "ancien", tokenCount: 1, tps: 1 } });
    emit("shared", 101, done());
    emit("shared", 102, { event: "token", data: { content: "nouveau", tokenCount: 1, tps: 1 } });
    emit("shared", 102, done());

    emit("shared", 101, {
      event: "sessionSnapshot",
      data: { messages: [message("late", "user", "Intrus")], tokenCount: 0 },
    });
    emit("shared", 101, { event: "token", data: { content: "tardif", tokenCount: 1, tps: 1 } });

    const snapshot = agentStreamManager.getSnapshot("shared");
    const messages = snapshot?.messages ?? [];
    expect(snapshot?.isStreaming).toBe(false);
    expect(snapshot?.currentContent).toBe("");
    expect(messages[messages.length - 1]?.content).toBe("nouveau");
  });

  it("ne mélange pas un run UI actif et un run backend concurrent", async () => {
    await agentStreamManager.startSession("shared", [message("u1", "user", "Question")], 0);
    agentStreamManager.setSessionGeneration("shared", 7);
    emit("shared", 7, { event: "token", data: { content: "UI-1", tokenCount: 1, tps: 1 } });
    emit("shared", 201, { event: "token", data: { content: "backend", tokenCount: 1, tps: 1 } });
    emit("shared", 201, done());
    emit("shared", 7, { event: "token", data: { content: "UI-2", tokenCount: 1, tps: 1 } });
    emit("shared", 7, done());
    emit("shared", 201, {
      event: "token",
      data: { content: "backend-tardif", tokenCount: 1, tps: 1 },
    });

    const snapshot = agentStreamManager.getSnapshot("shared");
    const messages = snapshot?.messages ?? [];
    expect(snapshot?.isStreaming).toBe(false);
    expect(snapshot?.currentContent).toBe("");
    expect(messages[messages.length - 1]?.content).toBe("UI-1UI-2");
  });

  it("accepte le snapshot, les tokens et la fin d'un run subagent identifié", async () => {
    await agentStreamManager.startSession("child", [], 0);
    emit("child", 301, {
      event: "sessionSnapshot",
      data: { messages: [message("mission", "user", "Mission")], tokenCount: 1 },
    });
    emit("child", 301, { event: "token", data: { content: "réponse", tokenCount: 1, tps: 1 } });
    emit("child", 301, done());

    const snapshot = agentStreamManager.getSnapshot("child");
    const messages = snapshot?.messages ?? [];
    expect(snapshot?.completed).toBe(true);
    expect(messages[messages.length - 1]?.content).toBe("réponse");
  });

  it("reprend l'identifiant persistant d'un stream après rechargement", async () => {
    const runId = "7c8e3a14-8811-4d88-9a54-d234547d8d22";
    await agentStreamManager.startSession("reload", [], 0);
    emit("reload", 302, {
      event: "sessionSnapshot",
      data: {
        messages: [{
          ...message("checkpoint", "assistant", "Travail"),
          stream_run_id: runId,
          stream_part: "checkpoint",
        }],
        tokenCount: 1,
      },
    });

    expect(agentStreamManager.getSnapshot("reload")?.streamRunId).toBe(runId);
    emit("reload", 302, { event: "token", data: { content: "Fin", tokenCount: 1, tps: 1 } });
    emit("reload", 302, done());

    const saved = agentStreamManager.getSnapshot("reload")?.messages ?? [];
    const final = saved[saved.length - 1];
    expect(final?.stream_run_id).toBe(runId);
    expect(final?.stream_part).toBe("final");
  });

  it("accepte un run gateway sans snapshot après Stop", async () => {
    await agentStreamManager.startSession("gateway", [], 0);
    agentStreamManager.setSessionGeneration("gateway", 9);
    agentStreamManager.stopSession("gateway", 9);
    emit("gateway", 401, { event: "token", data: { content: "gateway", tokenCount: 1, tps: 1 } });
    emit("gateway", 401, done());

    const snapshot = agentStreamManager.getSnapshot("gateway");
    const messages = snapshot?.messages ?? [];
    expect(snapshot?.completed).toBe(true);
    expect(messages[messages.length - 1]?.content).toBe("gateway");
  });
});
