import {
  createManagedStreamState,
  toChatState,
  type ChatState,
  type PermissionRequestState,
} from "./agent-chat-stream-callbacks";
import { clearCleanup, enforceSessionLimit, type StreamRecord } from "./agent-stream-cleanup";
import type { StreamKind } from "./agent-chat-stream-types";
import type { AgentMessage } from "@/types/agent";
import { assignStreamRun, type StreamRun } from "./agent-stream-run-ownership";
import type { ContextUsageRecord } from "@/types/agent-session.generated";
import { resolveContextUsage } from "./agent-token-estimate";

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
    state: {
      ...createManagedStreamState([], 0),
      isStreaming: false,
      isWorking: false,
    },
    subscribers: new Map(),
    nextSubscriberId: 1,
    cleanupTimer: null,
    notifyHandle: null,
    started: false,
    activeGeneration: null,
    awaitingAdmission: false,
    pendingAdmissionBuckets: [],
    cancelledGenerations: [],
    cancelledWithoutGeneration: false,
    runOwner: null,
    runOrigin: null,
    runId: 0,
    stopClaim: null,
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
  awaitingAdmission = false,
  run?: StreamRun,
  contextUsageRecord?: ContextUsageRecord,
): StreamRecord {
  const record = getOrCreateRecord(sessionId);
  clearCleanup(record);
  const previous = record.state;
  const next = createManagedStreamState(
    messages, sessionTokenCount, streamKind, contextUsageRecord ?? previous.contextUsageRecord,
  );
  record.state = streamKind === "compression"
    && resolveContextUsage(previous.contextUsageRecord).used !== null ? {
    ...next,
    contextUsageRecord: previous.contextUsageRecord,
    contextLimitTokens: previous.contextLimitTokens,
    contextUsageBuckets: previous.contextUsageBuckets,
    contextUsageBaseSegments: previous.contextUsageBaseSegments,
    contextUsageIncludesReasoning: previous.contextUsageIncludesReasoning,
    contextUsageVisible: previous.contextUsageVisible,
  } : next;
  record.started = true;
  if (awaitingAdmission && record.activeGeneration !== null) {
    record.cancelledGenerations = [
      ...record.cancelledGenerations,
      record.activeGeneration,
    ].slice(-16);
  }
  record.activeGeneration = null;
  record.awaitingAdmission = awaitingAdmission;
  record.pendingAdmissionBuckets = [];
  record.cancelledWithoutGeneration = false;
  assignStreamRun(record, run);
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
