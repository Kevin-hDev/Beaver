import i18n from "@/i18n";
import { scheduleCleanup } from "./agent-stream-cleanup";
import { flushFrameNotify } from "./agent-stream-notify";
import { notifyRecord, notifyRecordActivity } from "./agent-stream-notify-dispatch";
import { finishPersistenceRun } from "./agent-stream-persistence-owner";
import { getRecord, records } from "./agent-stream-records";

export function failSession(sessionId: string) {
  const record = getRecord(sessionId);
  if (!record) return;
  record.activeGeneration = null;
  record.state = {
    ...record.state,
    isStreaming: false,
    isCompressing: false,
    completed: true,
    activeStreamItem: null,
    error: i18n.t("errors.streamStartFailed"),
    updatedAt: Date.now(),
  };
  finishPersistenceRun(record);
  flushFrameNotify(record, notifyRecord);
  notifyRecordActivity(sessionId, record);
  scheduleCleanup(sessionId, record, records);
}
