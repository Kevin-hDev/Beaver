import { MAX_QUEUED_USER_MESSAGES } from "./agent-chat-stream-types";
import { flushFrameNotify } from "./agent-stream-notify";
import { notifyRecord, notifyRecordActivity } from "./agent-stream-notify-dispatch";
import { getRecord } from "./agent-stream-records";
import type { AgentMessage } from "@/types/agent";

export function queueUserMessage(sessionId: string, message: AgentMessage): boolean {
  const record = getRecord(sessionId);
  if (!record?.state.isStreaming || record.activeGeneration === null) return false;
  if (
    record.state.queuedUserMessages.length >= MAX_QUEUED_USER_MESSAGES
    || message.role !== "user"
  ) return false;
  record.state = {
    ...record.state,
    queuedUserMessages: [...record.state.queuedUserMessages, {
      ...message,
      stream_run_id: record.state.streamRunId,
      stream_part: "input",
    }],
    updatedAt: Date.now(),
  };
  flushFrameNotify(record, notifyRecord);
  notifyRecordActivity(sessionId, record);
  return true;
}

export function removeQueuedUserMessage(sessionId: string, messageId: string) {
  const record = getRecord(sessionId);
  if (!record) return;
  record.state = {
    ...record.state,
    queuedUserMessages: record.state.queuedUserMessages.filter((item) => item.id !== messageId),
    updatedAt: Date.now(),
  };
  flushFrameNotify(record, notifyRecord);
}
