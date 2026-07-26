import { describe, expect, it, vi } from "vitest";
import { applyStreamEvent, createManagedStreamState } from "@/hooks/agent-chat-stream-callbacks";
import type { ManagedStreamState } from "@/hooks/agent-chat-stream-callbacks";
import type { StreamEvent } from "@/types/agent";

vi.mock("@/i18n", () => ({ default: { t: (key: string) => key } }));

function makeState(overrides: Partial<ManagedStreamState> = {}): ManagedStreamState {
  return { ...createManagedStreamState([], 0), streamStartedAt: null, segmentStartedAt: null, ...overrides };
}

function doneEvent(): StreamEvent {
  return {
    event: "done",
    data: { evalCount: null, evalDurationNs: 0, finalTps: 0, promptTokens: null, contextTokens: null },
  };
}

describe("activeStreamItem", () => {
  it("active le thinking sur un fragment de réflexion", () => {
    const { state } = applyStreamEvent(makeState(), { event: "thinking", data: { content: "hmm" } });
    expect(state.activeStreamItem).toEqual({ kind: "thinking" });
  });

  it("active tous les tools lancés sans résultat", () => {
    let state = applyStreamEvent(makeState(), {
      event: "toolCall",
      data: { name: "bash", arguments: { command: "pwd" } },
    }).state;
    state = applyStreamEvent(state, {
      event: "toolCall",
      data: { name: "grep", arguments: { pattern: "x" } },
    }).state;
    expect(state.activeStreamItem).toEqual({ kind: "tools", toolIndices: [0, 1] });
  });

  it("garde le tool terminé actif jusqu'au prochain événement visible", () => {
    let state = applyStreamEvent(makeState(), {
      event: "toolCall",
      data: { name: "bash", arguments: { command: "pwd" } },
    }).state;
    state = applyStreamEvent(state, {
      event: "toolResult",
      data: { name: "bash", toolCallIndex: 0, content: "ok", isError: false },
    }).state;
    expect(state.activeStreamItem).toEqual({ kind: "tools", toolIndices: [0] });
  });

  it("revient aux tools encore en cours quand le plus récent finit", () => {
    let state = makeState();
    state = applyStreamEvent(state, { event: "toolCall", data: { name: "bash", arguments: {} } }).state;
    state = applyStreamEvent(state, { event: "toolCall", data: { name: "grep", arguments: {} } }).state;
    state = applyStreamEvent(state, {
      event: "toolResult",
      data: { name: "grep", toolCallIndex: 1, content: "ok", isError: false },
    }).state;
    expect(state.activeStreamItem).toEqual({ kind: "tools", toolIndices: [0] });
  });

  it("coupe l'animation sur un token texte visible", () => {
    const state = makeState({ activeStreamItem: { kind: "tools", toolIndices: [0] } });
    const result = applyStreamEvent(state, {
      event: "token",
      data: { content: "texte", tps: 1, tokenCount: 1 },
    });
    expect(result.state.activeStreamItem).toBeNull();
  });

  it("nettoie l'élément actif sur turnEnd, done et error", () => {
    const active = { kind: "tools" as const, toolIndices: [0] };
    expect(applyStreamEvent(makeState({ activeStreamItem: active }), {
      event: "turnEnd", data: {},
    }).state.activeStreamItem).toBeNull();
    expect(applyStreamEvent(makeState({ activeStreamItem: active }), doneEvent()).state.activeStreamItem).toBeNull();
    expect(applyStreamEvent(makeState({ activeStreamItem: active }), {
      event: "error", data: { message: "crash" },
    }).state.activeStreamItem).toBeNull();
  });
});
