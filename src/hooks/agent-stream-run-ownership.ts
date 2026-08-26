import type { StreamRecord } from "./agent-stream-cleanup";

export interface StreamRun {
  owner: symbol;
  id: number;
}

export function assignStreamRun(record: StreamRecord, run?: StreamRun) {
  record.runOwner = run?.owner ?? null;
  record.runId = run?.id ?? 0;
  record.stoppingGeneration = null;
}

export function ownsStreamRun(record: StreamRecord, run: StreamRun): boolean {
  return record.runOwner === run.owner
    && record.runId === run.id
    && record.stoppingGeneration === null;
}

export function ownedGeneration(record: StreamRecord, owner: symbol): number | null {
  if (record.runOwner !== owner || record.stoppingGeneration !== null) return null;
  return record.activeGeneration;
}

export function claimOwnedStop(record: StreamRecord, owner: symbol): number | null {
  const generation = ownedGeneration(record, owner);
  if (generation === null || !record.state.isStreaming) return null;
  record.stoppingGeneration = generation;
  return generation;
}

export function releaseOwnedStop(record: StreamRecord, owner: symbol, generation: number) {
  if (record.runOwner === owner && record.stoppingGeneration === generation) {
    record.stoppingGeneration = null;
  }
}

export function consumeOwnedStop(record: StreamRecord, owner: symbol, generation: number) {
  if (record.runOwner !== owner
      || record.activeGeneration !== generation
      || record.stoppingGeneration !== generation) return false;
  clearStreamRun(record);
  return true;
}

export function clearStreamRun(record: StreamRecord) {
  record.runOwner = null;
  record.runId = 0;
  record.stoppingGeneration = null;
}
