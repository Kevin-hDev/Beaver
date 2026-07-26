import { describe, it, expect, vi } from "vitest";
import { applyStreamEvent, finishPartialStream, createManagedStreamState } from "@/hooks/agent-chat-stream-callbacks";
import type { ManagedStreamState } from "@/hooks/agent-chat-stream-callbacks";
import type { StreamEvent } from "@/types/agent";

vi.mock("@/i18n", () => ({ default: { t: (key: string) => key } }));

function makeState(overrides: Partial<ManagedStreamState> = {}): ManagedStreamState {
  return { ...createManagedStreamState([], 0), streamStartedAt: null, segmentStartedAt: null, ...overrides };
}

function doneEvent(overrides: Partial<StreamEvent & { event: "done" } extends { data: infer D } ? D : never> = {}): StreamEvent {
  return { event: "done", data: { evalCount: null, evalDurationNs: 0, finalTps: 5, promptTokens: null, contextTokens: null, ...overrides } };
}

describe("done", () => {
  it("ajoute la durée de travail au message final", () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date("2026-07-01T12:00:10Z"));
    const result = applyStreamEvent(
      makeState({ streamStartedAt: Date.now() - 10_000, currentContent: "réponse" }),
      doneEvent(),
    );
    expect(result.assistantMessage?.work_duration_ms).toBe(10_000);
    vi.useRealTimers();
  });

  it("finalise le stream (isStreaming=false, completed=true)", () => {
    const { state: s } = applyStreamEvent(makeState(), doneEvent({ finalTps: 12 }));
    expect(s.isStreaming).toBe(false);
    expect(s.completed).toBe(true);
  });

  it("crée un assistantMessage si du contenu existe", () => {
    const result = applyStreamEvent(makeState({ currentContent: "réponse finale" }), doneEvent({ finalTps: 10 }));
    expect(result.assistantMessage).toBeDefined();
    expect(result.assistantMessage?.role).toBe("assistant");
    expect(result.assistantMessage?.content).toBe("réponse finale");
  });

  it("ne crée pas de message si aucun contenu", () => {
    const result = applyStreamEvent(makeState(), doneEvent());
    expect(result.assistantMessage).toBeUndefined();
  });

  it("respecte MAX_MESSAGES_PER_SESSION (2000)", () => {
    const messages = Array.from({ length: 2000 }, (_, i) => ({
      id: `msg-${i}`, role: "user" as const, content: `msg ${i}`,
      files: [], timestamp: new Date().toISOString(), tokens: 0,
    }));
    const result = applyStreamEvent(makeState({ messages, currentContent: "nouveau" }), doneEvent());
    expect(result.state.messages.length).toBeLessThanOrEqual(2000);
  });

  it("utilise contextTokens pour le total contexte session", () => {
    const result = applyStreamEvent(
      makeState({ sessionTokenCount: 100, currentContent: "réponse" }),
      doneEvent({ evalCount: 5, promptTokens: 10, contextTokens: 999 }),
    );
    expect(result.state.sessionTokenCount).toBe(999);
    expect(result.state.sessionTokenCountEstimated).toBe(false);
    expect(result.assistantMessage?.tokens).toBe(5);
  });

  it("retombe sur l'estimation si contextTokens est absent", () => {
    const result = applyStreamEvent(
      makeState({ sessionTokenCount: 100, currentContent: "你".repeat(1000) }),
      doneEvent({ evalCount: null, promptTokens: null, contextTokens: null }),
    );
    expect(result.state.sessionTokenCount).toBe(1250);
    expect(result.state.sessionTokenCountEstimated).toBe(true);
  });
});

describe("events neutres", () => {
  it("sessionSnapshot ne modifie pas l'état observable", () => {
    const state = makeState({ currentContent: "contenu existant" });
    const { state: s } = applyStreamEvent(state, {
      event: "sessionSnapshot", data: { messages: [], tokenCount: 0 },
    });
    expect(s.currentContent).toBe("contenu existant");
  });

  it("subagentSpawned ne modifie pas l'état observable", () => {
    const state = makeState({ currentContent: "contenu existant" });
    const { state: s } = applyStreamEvent(state, {
      event: "subagentSpawned",
      data: {
        subagentSessionId: "s1",
        subagentName: "test",
        subagentType: "explorer",
        subagentDescription: "Analyse",
        subagentColorKey: "geminitor",
        promptPreview: "...",
      },
    });
    expect(s.currentContent).toBe("contenu existant");
  });

  it("notice ne modifie pas l'état observable", () => {
    const state = makeState({ currentContent: "contenu existant" });
    const { state: s } = applyStreamEvent(state, {
      event: "notice", data: { messageKey: "vision.unsupportedModel" },
    });
    expect(s.currentContent).toBe("contenu existant");
  });
});

describe("finishPartialStream", () => {
  it("finalise un stream incomplet (isStreaming=false, completed=true)", () => {
    const result = finishPartialStream(makeState({ currentContent: "partiel" }));
    expect(result.state.isStreaming).toBe(false);
    expect(result.state.completed).toBe(true);
  });

  it("crée un assistantMessage depuis le contenu partiel", () => {
    const result = finishPartialStream(makeState({ currentContent: "contenu partiel" }));
    expect(result.assistantMessage).toBeDefined();
    expect(result.assistantMessage?.content).toBe("contenu partiel");
  });

  it("ne crée pas de message si l'état est vide", () => {
    const result = finishPartialStream(makeState());
    expect(result.assistantMessage).toBeUndefined();
  });

  it("combine les segments complétés et le contenu courant", () => {
    const state = makeState({
      completedSegments: [{ thinking: "", tools: [], content: "segment 1" }],
      currentContent: "segment 2",
    });
    const result = finishPartialStream(state);
    expect(result.assistantMessage?.content).toContain("segment 1");
    expect(result.assistantMessage?.content).toContain("segment 2");
  });

  it("marque le contenu partiel non confirmé comme travail si une phase de travail existe", () => {
    const result = finishPartialStream(makeState({
      completedSegments: [{ thinking: "", tools: [{ name: "bash", args: {}, result: "ok" }], content: "" }],
      currentContent: "fragment interrompu",
    }));
    expect(result.assistantMessage?.segments?.map((seg) => seg.phase)).toEqual(["work", "work"]);
  });

  it("ne replie pas un simple chat partiel sans indice de phase de travail", () => {
    const result = finishPartialStream(makeState({ currentContent: "réponse interrompue" }));
    expect(result.assistantMessage?.segments?.[0].phase).toBeUndefined();
  });

  it("conserve la phase finale si elle était déjà confirmée avant l'arrêt", () => {
    const result = finishPartialStream(makeState({
      currentContent: "réponse finale partielle",
      currentContentPhase: "final",
    }));
    expect(result.assistantMessage?.segments?.[0].phase).toBe("final");
  });

  it("marque les outils encore en cours comme annulés", () => {
    const result = finishPartialStream(makeState({
      currentTools: [
        { name: "bash", args: { command: "sleep 10" } },
        { name: "read_file", args: { path: "a.txt" }, result: "ok" },
      ],
    }));
    expect(result.assistantMessage?.tool_activities?.[0]).toMatchObject({
      name: "bash",
      result: "Annulé.",
      is_error: true,
    });
    expect(result.assistantMessage?.tool_activities?.[1]).toMatchObject({
      name: "read_file",
      result: "ok",
    });
  });
});

describe("done — limite messages assertion précise", () => {
  it("le 2001ème message évince msg-0 et place le nouveau en dernière position", () => {
    const messages = Array.from({ length: 2000 }, (_, i) => ({
      id: `msg-${i}`, role: "user" as const, content: `msg ${i}`,
      files: [], timestamp: new Date().toISOString(), tokens: 0,
    }));
    const result = applyStreamEvent(
      makeState({ messages, currentContent: "message 2001" }),
      doneEvent({ finalTps: 5 }),
    );
    expect(result.state.messages).toHaveLength(2000);
    // msg-0 doit être évincé
    expect(result.state.messages.find((m) => m.id === "msg-0")).toBeUndefined();
    // le nouveau message est le dernier
    expect(result.state.messages[1999].content).toBe("message 2001");
  });

  it("le compteur de réponse ignore promptTokens", () => {
    const result = applyStreamEvent(
      makeState({ sessionTokenCount: 100, currentContent: "réponse courte" }),
      doneEvent({ evalCount: 50, promptTokens: 25_000, contextTokens: 25_050 }),
    );
    expect(result.assistantMessage?.tokens).toBe(50);
    expect(result.assistantTokens).toBe(result.assistantMessage?.tokens);
    expect(result.state.lastRequestTokens).toBe(result.assistantMessage?.tokens);
  });
});
