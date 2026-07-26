import { snapshot } from "./agent-stream-records";
import { emitStreamActivity } from "./agent-stream-activity";
import type { StreamRecord } from "./agent-stream-cleanup";

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
