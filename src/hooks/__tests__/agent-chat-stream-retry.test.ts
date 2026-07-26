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
});
