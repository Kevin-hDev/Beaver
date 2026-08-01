import { describe, expect, it, vi } from "vitest";
import {
  applyStreamEvent,
  createManagedStreamState,
} from "@/hooks/agent-chat-stream-callbacks";
import type { ManagedStreamState } from "@/hooks/agent-chat-stream-callbacks";
import type { StreamEvent } from "@/types/agent";

vi.mock("@/i18n", () => ({ default: { t: (key: string) => key } }));

function makeState(overrides: Partial<ManagedStreamState>): ManagedStreamState {
  return {
    ...createManagedStreamState([], 0),
    streamStartedAt: null,
    segmentStartedAt: null,
    ...overrides,
  };
}

function doneEvent(
  overrides: Partial<
    StreamEvent & { event: "done" } extends { data: infer Data } ? Data : never
  >,
): StreamEvent {
  return {
    event: "done",
    data: {
      evalCount: null,
      evalDurationNs: 0,
      finalTps: 5,
      tpsEstimated: false,
      promptTokens: null,
      contextTokens: null,
      ...overrides,
    },
  };
}

describe("done — limite messages assertion précise", () => {
  it("le 2001ème message évince msg-0 et place le nouveau en dernière position", () => {
    const messages = Array.from({ length: 2000 }, (_, index) => ({
      id: `msg-${index}`,
      role: "user" as const,
      content: `msg ${index}`,
      files: [],
      timestamp: new Date().toISOString(),
      tokens: 0,
    }));
    const result = applyStreamEvent(
      makeState({ messages, currentContent: "message 2001" }),
      doneEvent({ finalTps: 5 }),
    );

    expect(result.state.messages).toHaveLength(2000);
    expect(result.state.messages.find((message) => message.id === "msg-0")).toBeUndefined();
    expect(result.state.messages[1999].content).toBe("message 2001");
  });

  it("le compteur de réponse ignore promptTokens", () => {
    const result = applyStreamEvent(
      makeState({ sessionTokenCount: 100, currentContent: "réponse courte" }),
      doneEvent({ evalCount: 50, promptTokens: 25_000, contextTokens: 25_050 }),
    );

    expect(result.assistantMessage?.tokens).toBe(50);
    expect(result.assistantTokens).toBe(result.assistantMessage?.tokens);
  });
});
