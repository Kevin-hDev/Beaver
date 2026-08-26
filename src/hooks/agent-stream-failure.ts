import i18n from "@/i18n";
import { scheduleCleanup } from "./agent-stream-cleanup";
import { flushFrameNotify } from "./agent-stream-notify";
import { notifyRecord, notifyRecordActivity } from "./agent-stream-notify-dispatch";
import { getRecord, records } from "./agent-stream-records";
import { markStreamCancelled } from "./agent-stream-generations";
import { clearStreamRun } from "./agent-stream-run-ownership";

export function failSession(sessionId: string, message = i18n.t("errors.streamStartFailed")) {
  const record = getRecord(sessionId);
  if (!record) return;
  clearStreamRun(record);
  markStreamCancelled(record, record.activeGeneration);
  record.state = {
    ...record.state,
    isStreaming: false,
    isCompressing: false,
    completed: true,
    activeStreamItem: null,
    error: message,
    updatedAt: Date.now(),
  };
  flushFrameNotify(record, notifyRecord);
  notifyRecordActivity(sessionId, record);
  scheduleCleanup(sessionId, record, records);
}
