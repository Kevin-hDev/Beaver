import type { StreamRecord } from "./agent-stream-cleanup";

export interface StreamRun {
  owner: symbol;
  id: number;
}

export type StopClaim =
  | { kind: "pending" }
  | { kind: "ready"; generation: number };

export function assignStreamRun(record: StreamRecord, run?: StreamRun) {
  record.runOwner = run?.owner ?? null;
  record.runOrigin = run?.owner ?? null;
  record.runId = run?.id ?? 0;
  record.stopRequested = false;
  record.stoppingGeneration = null;
}

export function ownsStreamRun(record: StreamRecord, run: StreamRun): boolean {
  return record.runOrigin === run.owner
    && record.runId === run.id
    && !record.stopRequested
    && record.stoppingGeneration === null;
}

export function matchesStreamRun(record: StreamRecord, run: StreamRun): boolean {
  return record.runOrigin === run.owner && record.runId === run.id;
}

export function adoptStreamOwner(record: StreamRecord, owner: symbol): boolean {
  if (record.runOwner !== null && record.runOwner !== owner) return false;
  record.runOwner = owner;
  return true;
}

export function ownedGeneration(record: StreamRecord, owner: symbol): number | null {
  if (record.runOwner !== owner
      || record.stopRequested
      || record.stoppingGeneration !== null) return null;
  return record.activeGeneration;
}

export function claimOwnedStop(record: StreamRecord, owner: symbol): StopClaim | null {
  if (!record.state.isStreaming
      || record.stopRequested
      || !adoptStreamOwner(record, owner)) return null;
  record.stopRequested = true;
  const generation = record.activeGeneration;
  if (generation === null) return { kind: "pending" };
  record.stoppingGeneration = generation;
  return { kind: "ready", generation };
}

export function releaseOwnedStop(record: StreamRecord, owner: symbol, generation: number) {
  if (record.runOwner === owner && record.stoppingGeneration === generation) {
    record.stopRequested = false;
    record.stoppingGeneration = null;
  }
}

export function consumeOwnedStop(record: StreamRecord, owner: symbol, generation: number) {
  if (record.runOwner !== owner
      || record.activeGeneration !== generation
      || !record.stopRequested
      || record.stoppingGeneration !== generation) return false;
  clearStreamRun(record);
  return true;
}

export function clearStreamRun(record: StreamRecord) {
  record.runOwner = null;
  record.runOrigin = null;
  record.runId = 0;
  record.stopRequested = false;
  record.stoppingGeneration = null;
}
