import type { ManagedStreamState } from "./agent-chat-stream-callbacks";

const CLEANUP_DELAY_MS = 5 * 60 * 1000;

const MAX_SESSIONS = 64;
export const MAX_EVENTS_PER_SESSION = 4096;
const MAX_SUBSCRIBERS_PER_SESSION = 32;
export const MAX_CANCELLED_GENERATIONS = 16;

export interface StreamRecord {
  state: ManagedStreamState;
  history: import("@/types/agent").StreamEvent[];
  subscribers: Map<number, (snapshot: unknown) => void>;
  nextSubscriberId: number;
  cleanupTimer: ReturnType<typeof setTimeout> | null;
  notifyHandle: { cancel: () => void } | null;
  started: boolean;
  backendOwnsPersistence: boolean;
  isSubagentBackendStream: boolean;
  activeGeneration: number | null;
  cancelledGenerations: number[];
  cancelledWithoutGeneration: boolean;
  persistenceQueue: Promise<void>;
  persistencePending: boolean;
}

export function enforceSessionLimit(records: Map<string, StreamRecord>) {
  for (const [sessionId, record] of records) {
    if (records.size <= MAX_SESSIONS) return;
    if (record.state.isStreaming || record.subscribers.size > 0) continue;
    clearCleanup(record);
    clearScheduledNotify(record);
    records.delete(sessionId);
  }
}

export function scheduleCleanup(
  sessionId: string,
  record: StreamRecord,
  records: Map<string, StreamRecord>,
) {
  clearCleanup(record);
  record.cleanupTimer = setTimeout(() => {
    if (record.subscribers.size === 0 && !record.state.isStreaming) {
      clearScheduledNotify(record);
      records.delete(sessionId);
    }
  }, CLEANUP_DELAY_MS);
}

export function clearCleanup(record: StreamRecord) {
  if (record.cleanupTimer) clearTimeout(record.cleanupTimer);
  record.cleanupTimer = null;
}

export function clearScheduledNotify(record: StreamRecord) {
  record.notifyHandle?.cancel();
  record.notifyHandle = null;
}

export function trimSubscribers(record: StreamRecord) {
  while (record.subscribers.size > MAX_SUBSCRIBERS_PER_SESSION) {
    const first = record.subscribers.keys().next().value;
    if (first === undefined) break;
    record.subscribers.delete(first);
  }
}
