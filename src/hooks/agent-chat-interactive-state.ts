import type { ChatState } from "./agent-chat-stream-types";

export function clearInteractiveChoiceState(state: ChatState): ChatState {
  const hidePlan = state.interactiveChoice?.kind === "plan_approval";
  return {
    ...state,
    interactiveChoice: undefined,
    planPreview: hidePlan ? null : state.planPreview,
  };
}
