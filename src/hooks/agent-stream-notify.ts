import type { StreamEvent } from "@/types/agent";
import { clearScheduledNotify, type StreamRecord } from "./agent-stream-cleanup";
import { snapshot } from "./agent-stream-records";
import { emitStreamActivity } from "./agent-stream-activity";

type NotifyDispatch = (record: StreamRecord) => void;

export function shouldDeferStreamEvent(event: StreamEvent): boolean {
  return event.event === "generationStarted"
    || event.event === "token"
    || event.event === "thinking";
}

export function scheduleFrameNotify(record: StreamRecord, dispatch: NotifyDispatch) {
  if (record.notifyHandle) return;
  record.notifyHandle = scheduleNextFrame(() => {
    record.notifyHandle = null;
    dispatch(record);
  });
}

export function flushFrameNotify(record: StreamRecord, dispatch: NotifyDispatch) {
  clearScheduledNotify(record);
  dispatch(record);
}

function scheduleNextFrame(callback: () => void): { cancel: () => void } {
  if (typeof window !== "undefined" && typeof window.requestAnimationFrame === "function") {
    const id = window.requestAnimationFrame(callback);
    return { cancel: () => window.cancelAnimationFrame(id) };
  }
  const id = setTimeout(callback, 16);
  return { cancel: () => clearTimeout(id) };
}

type Subscriber = (value: ReturnType<typeof snapshot>) => void;

export function notifyRecord(record: StreamRecord) {
  if (!record.started) return;
  const value = snapshot(record.state);
  for (const subscriber of record.subscribers.values()) {
    (subscriber as Subscriber)(value);
  }
}

export function notifyRecordActivity(sessionId: string, record: StreamRecord) {
  if (!record.started) return;
  emitStreamActivity(sessionId, record.state);
}
