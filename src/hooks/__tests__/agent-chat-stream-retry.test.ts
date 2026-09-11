import { describe, expect, it, vi } from "vitest";
import { applyStreamEvent, createManagedStreamState } from "@/hooks/agent-chat-stream-callbacks";
import type { ManagedStreamState } from "@/hooks/agent-chat-stream-callbacks";

vi.mock("@/i18n", () => ({ default: { t: (key: string) => key } }));

function makeState(overrides: Partial<ManagedStreamState> = {}): ManagedStreamState {
  return { ...createManagedStreamState([], 0), ...overrides };
}

describe("retryIndicator", () => {
  it("stocke l'indicateur de retry", () => {
    const { state } = applyStreamEvent(makeState(), {
      event: "retryIndicator",
      data: { reasonKey: "agentLocal.retry.server", attempt: 2, maxAttempts: 10 },
    });
    expect(state.retryIndicator).toEqual({
      reasonKey: "agentLocal.retry.server",
      attempt: 2,
      maxAttempts: 10,
    });
  });

  it("disparaît au premier vrai token", () => {
    const state = makeState({
      retryIndicator: { reasonKey: "agentLocal.retry.server", attempt: 1, maxAttempts: 10 },
    });
    const { state: next } = applyStreamEvent(state, {
      event: "token",
      data: { content: "ok", tokenCount: 1, tps: 1 },
    });
    expect(next.retryIndicator).toBeNull();
  });

  it("disparaît sur erreur", () => {
    const state = makeState({
      retryIndicator: { reasonKey: "agentLocal.retry.server", attempt: 1, maxAttempts: 10 },
    });
    const { state: next } = applyStreamEvent(state, {
      event: "error",
      data: { message: "crash" },
    });
    expect(next.retryIndicator).toBeNull();
  });

  it("efface seulement la tentative provider incomplète avant de rejouer", () => {
    const completedSegments = [
      { thinking: "", tools: [], content: "segment terminé" },
      { thinking: "travail partiel", tools: [], content: "" },
    ];
    const state = makeState({
      completedSegments,
      contextUsageRecord: {
        activeRequestId: "request-1",
        currentPreparation: {
          identity: { requestId: "request-1", turnId: "turn-1", turn: 0, attempt: 1, providerId: "openai", model: "gpt-5" },
          contextLimit: 200_000,
          input: { tokens: 100, capacityTokens: 100, source: "heuristic", coverage: "complete" },
          state: "in_flight", breakdown: null, updatedAt: "2026-09-11T00:00:00Z",
        },
        lastMeasurement: null,
        lastOutput: null,
      },
      contextUsageBaseSegments: 1,
      currentContent: "réponse partielle",
      currentContentPhase: "work",
      currentThinking: "raisonnement partiel",
      currentTools: [{ name: "read_file", args: { path: "test" } }],
      activeStreamItem: { kind: "thinking" },
      tps: 12,
      tpsEstimated: true,
      liveTokenCount: 35,
      requestOutputTokens: 20,
      sessionTokenCount: 120,
    });

    const { state: next } = applyStreamEvent(state, {
      event: "retryIndicator",
      data: { reasonKey: "agentLocal.retry.provider", attempt: 1, maxAttempts: 3 },
    });

    expect(next.completedSegments).toEqual([completedSegments[0]]);
    expect(next.currentContent).toBe("");
    expect(next.currentContentPhase).toBeUndefined();
    expect(next.currentThinking).toBe("");
    expect(next.currentTools).toEqual([]);
    expect(next.activeStreamItem).toBeNull();
    expect(next.tps).toBe(0);
    expect(next.liveTokenCount).toBe(15);
    expect(next.sessionTokenCount).toBe(100);
  });
});
