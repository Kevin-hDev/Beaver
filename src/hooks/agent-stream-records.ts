import { invoke } from "@tauri-apps/api/core";
import {
  createManagedStreamState,
  toChatState,
  type ChatState,
  type PermissionRequestState,
} from "./agent-chat-stream-callbacks";
import { clearCleanup, enforceSessionLimit, type StreamRecord } from "./agent-stream-cleanup";
import { flushFrameNotify } from "./agent-stream-notify";
import { startUiPersistence } from "./agent-stream-persistence-owner";
import type { StreamKind } from "./agent-chat-stream-types";
import type { AgentMessage } from "@/types/agent";

export interface StreamSnapshot extends ChatState {
  pendingPermissions: PermissionRequestState[];
  completed: boolean;
  error?: string;
  isConnectionError?: boolean;
  diagnosticSummary?: string;
}

export const records = new Map<string, StreamRecord>();

export function getRecord(sessionId: string): StreamRecord | undefined {
  return records.get(sessionId);
}

export function getOrCreateRecord(sessionId: string): StreamRecord {
  let record = records.get(sessionId);
  if (record) return record;
  record = {
    state: { ...createManagedStreamState([], 0), isStreaming: false },
    history: [],
    subscribers: new Map(),
    nextSubscriberId: 1,
    cleanupTimer: null,
    notifyHandle: null,
    started: false,
    backendOwnsPersistence: false,
    isSubagentBackendStream: false,
    activeGeneration: null,
    cancelledGenerations: [],
    cancelledWithoutGeneration: false,
    persistenceQueue: Promise.resolve(),
    persistencePending: false,
  };
  records.set(sessionId, record);
  enforceSessionLimit(records);
  return record;
}

export function touchSession(sessionId: string, record: StreamRecord) {
  records.delete(sessionId);
  records.set(sessionId, record);
  enforceSessionLimit(records);
}

export function startStreamRecord(
  sessionId: string,
  messages: AgentMessage[],
  sessionTokenCount: number,
  streamKind: StreamKind,
): StreamRecord {
  const record = getOrCreateRecord(sessionId);
  clearCleanup(record);
  record.state = createManagedStreamState(messages, sessionTokenCount, streamKind);
  record.history = [];
  record.started = true;
  startUiPersistence(record);
  record.activeGeneration = null;
  record.cancelledWithoutGeneration = false;
  touchSession(sessionId, record);
  return record;
}

export function snapshot(state: StreamRecord["state"]): StreamSnapshot {
  return {
    ...toChatState(state), pendingPermissions: [...state.pendingPermissions],
    completed: state.completed, error: state.error,
    isConnectionError: state.isConnectionError,
    diagnosticSummary: state.diagnosticSummary,
  };
}

export function persistMessages(
  sessionId: string,
  record: StreamRecord,
  messages: AgentMessage[],
  tokens: number,
  final: boolean,
  notify: (record: StreamRecord) => void,
) {
  if (final) record.state = { ...record.state, persisted: true };
  const run = () => Promise.resolve(invoke("add_messages_to_session", {
    id: sessionId, messages, tokens,
  })).then(() => undefined).catch(() => {
    console.warn("Session persistence failed.");
    if (final) record.state = { ...record.state, persisted: false };
    flushFrameNotify(record, notify);
  });
  const persistence = record.persistencePending
    ? record.persistenceQueue.catch(() => undefined).then(run)
    : run();
  record.persistencePending = true;
  record.persistenceQueue = persistence;
  void persistence.finally(() => {
    if (record.persistenceQueue === persistence) record.persistencePending = false;
  });
}
