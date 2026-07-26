import { getRecord, snapshot, type StreamSnapshot } from "./agent-stream-records";
import { setStreamGeneration } from "./agent-stream-generations";
import { toStreamActivity } from "./agent-stream-activity";

export function setSessionGeneration(sessionId: string, generation: number) {
  const record = getRecord(sessionId);
  if (!record) return;
  setStreamGeneration(record, generation);
}

export function getSnapshot(sessionId: string): StreamSnapshot | null {
  const record = getRecord(sessionId);
  if (!record?.started) return null;
  return snapshot(record.state);
}

export function getActivity(sessionId: string) {
  const record = getRecord(sessionId);
  if (!record?.started) return null;
  return toStreamActivity(sessionId, record.state);
}

export function isStreaming(sessionId: string): boolean {
  return getRecord(sessionId)?.state.isStreaming ?? false;
}
