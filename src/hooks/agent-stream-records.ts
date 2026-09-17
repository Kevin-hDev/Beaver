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
import { takePendingAdmission } from "./agent-stream-generations";
import { toStreamActivity } from "./agent-stream-activity";
import type { ContextUsageRecord } from "@/types/agent-session.generated";
import { resolveContextUsage } from "./agent-token-estimate";
import type { StreamProjectionState } from "./agent-stream-projections";

export interface StreamSnapshot extends ChatState {
  pendingPermissions: PermissionRequestState[];
  projection: StreamProjectionState;
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
  record.state.projection = previous.projection;
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
    projection: {
      ...state.projection,
      todos: state.projection.todos ? [...state.projection.todos] : null,
      subagents: {
        ...state.projection.subagents,
        active: [...state.projection.subagents.active],
        completed: [...state.projection.subagents.completed],
      },
    },
    completed: state.completed, error: state.error,
    isConnectionError: state.isConnectionError,
    diagnosticSummary: state.diagnosticSummary,
  };
}

export function setSessionGeneration(sessionId: string, generation: number) {
  const record = getRecord(sessionId);
  return record ? takePendingAdmission(record, generation) : null;
}

export function getSnapshot(sessionId: string): StreamSnapshot | null {
  const record = getRecord(sessionId);
  return record?.started ? snapshot(record.state) : null;
}

export function getActivity(sessionId: string) {
  const record = getRecord(sessionId);
  return record?.started ? toStreamActivity(sessionId, record.state) : null;
}

export function isStreaming(sessionId: string): boolean {
  return getRecord(sessionId)?.state.isStreaming ?? false;
}

export function clearStreamPermission(permissionId: string): void {
  for (const record of records.values()) {
    const nextPending = record.state.pendingPermissions.filter((item) => item.id !== permissionId);
    if (nextPending.length === record.state.pendingPermissions.length) continue;
    record.state = { ...record.state, pendingPermissions: nextPending };
  }
}
