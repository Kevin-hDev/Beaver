import { describe, expect, it, vi } from "vitest";
import { applyStreamEvent, createManagedStreamState } from "@/hooks/agent-chat-stream-callbacks";
import type { ManagedStreamState } from "@/hooks/agent-chat-stream-callbacks";

vi.mock("@/i18n", () => ({ default: { t: (key: string) => key } }));

function makeState(overrides: Partial<ManagedStreamState> = {}): ManagedStreamState {
  return { ...createManagedStreamState([], 0), ...overrides };
}

describe("plan mode stream events", () => {
  it("stocke et retire la preview de plan", () => {
    const plan = { id: "p1", title: "Plan", content: "Do it", status: "awaiting_approval" as const };
    let state = applyStreamEvent(makeState(), { event: "planPreviewUpdated", data: { plan } }).state;
    expect(state.planPreview).toEqual(plan);

    state = applyStreamEvent(state, { event: "planPreviewUpdated", data: { plan: null } }).state;
    expect(state.planPreview).toBeNull();
  });

  it("désactive le plan mode et retire la preview", () => {
    const state = makeState({
      planPreview: { id: "p1", title: "Plan", content: "Do it", status: "awaiting_approval" },
    });
    const result = applyStreamEvent(state, { event: "planModeUpdated", data: { enabled: false } });

    expect(result.state.planModeEnabled).toBe(false);
    expect(result.state.planPreview).toBeNull();
  });
});
