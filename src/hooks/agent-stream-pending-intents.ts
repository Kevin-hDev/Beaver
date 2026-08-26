import type { AgentMessage } from "@/types/agent";
import type { NewUserTurnInput } from "@/types/agent-turn.generated";

export interface PendingAdmission {
  sessionId: string;
  runId: number;
  input: NewUserTurnInput;
  displayMessage: AgentMessage;
}

export const MAX_PENDING_ADMISSION = 8;

export function takePendingForSession(
  items: PendingAdmission[],
  sessionId: string,
  runId?: number,
) {
  const matches = (item: PendingAdmission) =>
    item.sessionId === sessionId && (runId === undefined || item.runId === runId);
  const selected = items.filter(matches);
  const retained = items.filter((item) => !matches(item));
  items.splice(0, items.length, ...retained);
  return selected;
}
