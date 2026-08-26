import type { StreamRecord } from "./agent-stream-cleanup";
import { MAX_CANCELLED_GENERATIONS } from "./agent-stream-cleanup";
import type { StreamEvent } from "@/types/agent";

const MAX_PENDING_ADMISSION_EVENTS = 64;
const MAX_PENDING_ADMISSION_CHARS = 512 * 1024;

const STREAM_CONTINUATION_EVENTS = new Set<StreamEvent["event"]>([
  "token",
  "contentPhase",
  "thinking",
  "contextUsage",
  "generationStarted",
  "turnAdmitted",
  "turnCommitted",
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
  record.awaitingAdmission = false;
  clearPendingAdmission(record);
}

export function acceptsStreamEvent(
  record: StreamRecord,
  generation: number | null,
  event: StreamEvent,
): boolean {
  if (typeof generation === "number") {
    if (record.cancelledWithoutGeneration) {
      quarantineGeneration(record, generation);
      return false;
    }
    if (record.cancelledGenerations.includes(generation)) return false;
    if (record.awaitingAdmission) {
      stagePendingAdmission(record, generation, event);
      return false;
    }
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

export function takePendingAdmission(
  record: StreamRecord,
  generation: number,
) {
  const overflowed = record.pendingAdmissionOverflowed;
  const events = record.pendingAdmissionEvents.filter(
    (pending) => pending.generation === generation,
  );
  clearPendingAdmission(record);
  record.awaitingAdmission = false;
  return { events, overflowed, accepted: setStreamGeneration(record, generation) };
}

function stagePendingAdmission(
  record: StreamRecord,
  generation: number,
  event: StreamEvent,
) {
  if (record.pendingAdmissionOverflowed) return;
  const chars = JSON.stringify(event).length;
  if (record.pendingAdmissionEvents.length >= MAX_PENDING_ADMISSION_EVENTS
      || record.pendingAdmissionChars + chars > MAX_PENDING_ADMISSION_CHARS) {
    record.pendingAdmissionEvents = [];
    record.pendingAdmissionChars = 0;
    record.pendingAdmissionOverflowed = true;
    return;
  }
  record.pendingAdmissionEvents.push({ generation, event });
  record.pendingAdmissionChars += chars;
}

export function clearPendingAdmission(record: StreamRecord) {
  record.pendingAdmissionEvents = [];
  record.pendingAdmissionChars = 0;
  record.pendingAdmissionOverflowed = false;
}
