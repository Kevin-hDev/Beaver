import type { StreamRecord } from "./agent-stream-cleanup";

export interface StreamRun {
  owner: symbol;
  id: number;
}

export type StopClaim =
  | { kind: "pending"; token: symbol; runId: number }
  | { kind: "ready"; token: symbol; runId: number; generation: number };

export type OwnedRunState =
  | { kind: "pendingAdmission"; runId: number }
  | { kind: "active"; runId: number; generation: number }
  | { kind: "stopping"; runId: number; generation: number | null }
  | { kind: "terminal" };

export type QueueStreamResult = "queued" | "start-new" | "stopping";

export function assignStreamRun(record: StreamRecord, run?: StreamRun) {
  record.runOwner = run?.owner ?? null;
  record.runOrigin = run?.owner ?? null;
  record.runId = run?.id ?? 0;
  record.stopClaim = null;
}

export function ownsStreamRun(record: StreamRecord, run: StreamRun): boolean {
  return record.runOrigin === run.owner
    && record.runId === run.id
    && record.stopClaim === null;
}

export function matchesStreamRun(record: StreamRecord, run: StreamRun): boolean {
  return record.runOrigin === run.owner && record.runId === run.id;
}

export function adoptStreamOwner(record: StreamRecord, owner: symbol): boolean {
  if (record.runOwner !== null && record.runOwner !== owner) return false;
  record.runOwner = owner;
  return true;
}

export function ownedRunState(record: StreamRecord, owner: symbol): OwnedRunState {
  if (record.runOwner !== owner || !record.state.isStreaming) return { kind: "terminal" };
  if (record.stopClaim) {
    return {
      kind: "stopping", runId: record.runId,
      generation: record.stopClaim.generation,
    };
  }
  if (record.awaitingAdmission && record.activeGeneration === null) {
    return { kind: "pendingAdmission", runId: record.runId };
  }
  return record.activeGeneration === null
    ? { kind: "terminal" }
    : { kind: "active", runId: record.runId, generation: record.activeGeneration };
}

export function claimOwnedStop(record: StreamRecord, owner: symbol): StopClaim | null {
  if (!record.state.isStreaming
      || record.stopClaim !== null
      || !adoptStreamOwner(record, owner)) return null;
  const token = Symbol("stream-stop-claim");
  const generation = record.activeGeneration;
  record.stopClaim = { token, generation };
  return generation === null
    ? { kind: "pending", token, runId: record.runId }
    : { kind: "ready", token, runId: record.runId, generation };
}

export function bindDeferredStop(
  record: StreamRecord,
  run: StreamRun,
  generation: number,
): StopClaim | null {
  if (!matchesStreamRun(record, run)
      || !record.stopClaim
      || record.stopClaim.generation !== null) return null;
  record.stopClaim.generation = generation;
  return { kind: "ready", token: record.stopClaim.token, runId: run.id, generation };
}

export function releaseStopClaim(record: StreamRecord, claim: StopClaim) {
  if (record.stopClaim?.token !== claim.token) return false;
  record.stopClaim = null;
  return true;
}

export function consumeStopClaim(record: StreamRecord, claim: StopClaim) {
  if (claim.kind !== "ready"
      || record.stopClaim?.token !== claim.token
      || record.stopClaim.generation !== claim.generation
      || (record.activeGeneration !== null
        && record.activeGeneration !== claim.generation)) return false;
  clearStreamRun(record);
  return true;
}

export function clearStreamRun(record: StreamRecord) {
  record.runOwner = null;
  record.runOrigin = null;
  record.runId = 0;
  record.stopClaim = null;
}
