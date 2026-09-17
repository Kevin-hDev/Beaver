import type { StreamEvent } from "@/types/agent";
import type { StreamRecord } from "./agent-stream-cleanup";
import {
  applyStreamProjection,
  markSessionUpdate,
  refreshEvent,
  removeProjectedSubagent,
} from "./agent-stream-projections";

export function applyRecordProjection(
  record: StreamRecord,
  event: StreamEvent,
): string | null | undefined {
  if (event.event !== "todoUpdated" && event.event !== "subagentSpawned"
    && event.event !== "subagentCompleted") return undefined;
  record.state = {
    ...record.state,
    updatedAt: Date.now(),
    projection: applyStreamProjection(record.state.projection, event),
  };
  return event.event === "subagentCompleted" ? event.data.subagentSessionId : null;
}

export function markRecordSessionUpdate(record: StreamRecord, event: StreamEvent) {
  const kind = refreshEvent(event);
  if (!kind) return;
  record.state = {
    ...record.state,
    projection: markSessionUpdate(record.state.projection, kind),
  };
}

export function removeRecordSubagent(record: StreamRecord, sessionId: string) {
  record.state = {
    ...record.state,
    projection: removeProjectedSubagent(record.state.projection, sessionId),
  };
}
