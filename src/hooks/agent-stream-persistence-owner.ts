import type { StreamRecord } from "./agent-stream-cleanup";

export function startUiPersistence(record: StreamRecord) {
  record.backendOwnsPersistence = record.isSubagentBackendStream;
}

export function markIncomingBackendRun(record: StreamRecord) {
  if (record.started && !record.state.completed) return;
  record.started = true;
  record.backendOwnsPersistence = true;
  record.isSubagentBackendStream = false;
}

export function markSubagentSnapshot(record: StreamRecord) {
  record.backendOwnsPersistence = true;
  record.isSubagentBackendStream = true;
}

export function frontendShouldPersist(record: StreamRecord) {
  return !record.backendOwnsPersistence;
}

export function finishPersistenceRun(record: StreamRecord) {
  record.backendOwnsPersistence = false;
  record.isSubagentBackendStream = false;
}
