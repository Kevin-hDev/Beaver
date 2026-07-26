import type { StreamRecord } from "./agent-stream-cleanup";
import { MAX_CANCELLED_GENERATIONS } from "./agent-stream-cleanup";
import type { StreamEvent } from "@/types/agent";

const STREAM_CONTINUATION_EVENTS = new Set<StreamEvent["event"]>([
  "token",
  "contentPhase",
  "thinking",
  "toolCall",
  "toolResult",
  "turnEnd",
  "permissionRequest",
  "done",
  "error",
  "retryIndicator",
  "compressing",
  "compressionComplete",
  "interactiveChoiceRequest",
  "planPreviewUpdated",
  "planModeUpdated",
]);

export function setStreamGeneration(record: StreamRecord, generation: number): boolean {
  if (record.cancelledGenerations.includes(generation)) return false;
  record.activeGeneration = generation;
  record.cancelledWithoutGeneration = false;
  return true;
}

function quarantineGeneration(record: StreamRecord, generation: number) {
  record.cancelledGenerations = [
    ...record.cancelledGenerations.filter((item) => item !== generation),
    generation,
  ].slice(-MAX_CANCELLED_GENERATIONS);
}

export function markStreamCancelled(record: StreamRecord, generation?: number | null) {
  const resolved = typeof generation === "number" ? generation : record.activeGeneration;
  if (typeof resolved === "number") {
    quarantineGeneration(record, resolved);
  } else {
    record.cancelledWithoutGeneration = true;
  }
  record.activeGeneration = null;
}

export function acceptsStreamEvent(
  record: StreamRecord,
  generation: number | null,
  event: StreamEvent,
): boolean {
  if (typeof generation === "number") {
    if (record.cancelledGenerations.includes(generation)) return false;
    if (record.activeGeneration !== null && record.activeGeneration !== generation) {
      quarantineGeneration(record, generation);
      return false;
    }
    setStreamGeneration(record, generation);
    return true;
  }

  if (event.event === "sessionSnapshot") {
    record.cancelledWithoutGeneration = false;
    return true;
  }

  return !(record.cancelledWithoutGeneration && STREAM_CONTINUATION_EVENTS.has(event.event));
}
