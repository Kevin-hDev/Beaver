import { describe, expect, it } from "vitest";
import {
  clearInteractiveChoiceState,
  EMPTY_CHAT_STATE,
} from "@/hooks/agent-chat-stream-callbacks";

const planPreview = {
  id: "plan-1",
  title: "Plan",
  content: "Steps",
  status: "awaiting_approval" as const,
};

describe("clearInteractiveChoiceState", () => {
  it("masque immédiatement le plan après une décision ou Échap", () => {
    const state = clearInteractiveChoiceState({
      ...EMPTY_CHAT_STATE,
      planPreview,
      interactiveChoice: {
        sessionId: "session-1",
        id: "choice-1",
        kind: "plan_approval",
        questions: [],
        currentIndex: 0,
        total: 1,
      },
    });

    expect(state.interactiveChoice).toBeUndefined();
    expect(state.planPreview).toBeNull();
  });

  it("préserve un plan pour un autre choix interactif", () => {
    const state = clearInteractiveChoiceState({
      ...EMPTY_CHAT_STATE,
      planPreview,
      interactiveChoice: {
        sessionId: "session-1",
        id: "choice-1",
        kind: "general",
        questions: [],
        currentIndex: 0,
        total: 1,
      },
    });

    expect(state.planPreview).toEqual(planPreview);
  });
});
