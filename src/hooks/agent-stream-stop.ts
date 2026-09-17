import { finishPartialStream } from "./agent-chat-stream-callbacks";
import type { StreamRecord } from "./agent-stream-cleanup";
import { markStreamCancelled } from "./agent-stream-generations";
import { flushFrameNotify, notifyRecord, notifyRecordActivity } from "./agent-stream-notify";
import { clearStreamRun } from "./agent-stream-run-ownership";

export function stopStreamRecord(
  sessionId: string,
  record: StreamRecord,
  generation?: number | null,
) {
  clearStreamRun(record);
  markStreamCancelled(record, generation);
  const result = finishPartialStream(record.state);
  record.state = result.state;
  flushFrameNotify(record, notifyRecord);
  notifyRecordActivity(sessionId, record);
}
