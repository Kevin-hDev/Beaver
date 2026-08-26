import type { StreamRecord } from "./agent-stream-cleanup";
import { MAX_CANCELLED_GENERATIONS } from "./agent-stream-cleanup";
import type { StreamEvent } from "@/types/agent";

const MAX_PENDING_ADMISSION_EVENTS = 64;
const MAX_PENDING_ADMISSION_CHARS = 512 * 1024;
export const MAX_PENDING_ADMISSION_BUCKETS = 32;

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
  const bucket = record.pendingAdmissionBuckets.find(
    (pending) => pending.generation === generation,
  );
  const knownOrUnsaturated = bucket !== undefined
    || record.pendingAdmissionBuckets.length < MAX_PENDING_ADMISSION_BUCKETS;
  clearPendingAdmission(record);
  record.awaitingAdmission = false;
  return {
    events: (bucket?.events ?? []).map((event) => ({ generation, event })),
    overflowed: bucket?.overflowed ?? false,
    accepted: knownOrUnsaturated && setStreamGeneration(record, generation),
  };
}

function stagePendingAdmission(
  record: StreamRecord,
  generation: number,
  event: StreamEvent,
) {
  let bucket = record.pendingAdmissionBuckets.find(
    (pending) => pending.generation === generation,
  );
  if (!bucket) {
    if (record.pendingAdmissionBuckets.length >= MAX_PENDING_ADMISSION_BUCKETS) {
      const disposable = record.pendingAdmissionBuckets.findIndex((item) => item.overflowed);
      if (disposable >= 0) record.pendingAdmissionBuckets.splice(disposable, 1);
      else {
        quarantineGeneration(record, generation);
        return;
      }
    }
    bucket = { generation, events: [], chars: 0, overflowed: false };
    record.pendingAdmissionBuckets.push(bucket);
  }
  if (bucket.overflowed) return;
  const chars = JSON.stringify(event).length;
  if (bucket.events.length >= MAX_PENDING_ADMISSION_EVENTS
      || bucket.chars + chars > MAX_PENDING_ADMISSION_CHARS) {
    bucket.events = [];
    bucket.chars = 0;
    bucket.overflowed = true;
    return;
  }
  bucket.events.push(event);
  bucket.chars += chars;
}

export function clearPendingAdmission(record: StreamRecord) {
  record.pendingAdmissionBuckets = [];
}
