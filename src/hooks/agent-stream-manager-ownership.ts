import { getRecord, records } from "./agent-stream-records";
import {
  adoptStreamOwner,
  bindDeferredStop,
  claimOwnedStop,
  consumeStopClaim,
  matchesStreamRun,
  ownedGeneration,
  ownsStreamRun,
  releaseStopClaim,
  type StopClaim,
  type StreamRun,
} from "./agent-stream-run-ownership";
import { stopStreamRecord } from "./agent-stream-stop";

export function ownsRun(sessionId: string, run: StreamRun) {
  const record = getRecord(sessionId);
  return record ? ownsStreamRun(record, run) : false;
}

export function ownsOwner(sessionId: string, owner: symbol) {
  const record = getRecord(sessionId);
  return record?.runOwner === owner && record.stopClaim === null;
}

export function adoptOwner(sessionId: string, owner: symbol) {
  const record = getRecord(sessionId);
  return record ? adoptStreamOwner(record, owner) : false;
}

export function matchesRun(sessionId: string, run: StreamRun) {
  const record = getRecord(sessionId);
  return record ? matchesStreamRun(record, run) : false;
}

export function getDeferredStop(sessionId: string, run: StreamRun, generation: number) {
  const record = getRecord(sessionId);
  return record ? bindDeferredStop(record, run, generation) : null;
}

export function isOwnerStreaming(owner: symbol) {
  for (const record of records.values()) {
    if (record.runOwner === owner && record.state.isStreaming) return true;
  }
  return false;
}

export function getOwnedGeneration(sessionId: string, owner: symbol) {
  const record = getRecord(sessionId);
  return record ? ownedGeneration(record, owner) : null;
}

export function getOwnedRunId(sessionId: string, owner: symbol) {
  const record = getRecord(sessionId);
  return record?.runOwner === owner ? record.runId : null;
}

export function claimStop(sessionId: string, owner: symbol) {
  const record = getRecord(sessionId);
  return record ? claimOwnedStop(record, owner) : null;
}

export function releaseStop(sessionId: string, claim: StopClaim) {
  const record = getRecord(sessionId);
  return record ? releaseStopClaim(record, claim) : false;
}

export function completeStop(sessionId: string, claim: StopClaim) {
  const record = getRecord(sessionId);
  if (claim.kind !== "ready" || !record || !consumeStopClaim(record, claim)) return false;
  stopStreamRecord(sessionId, record, claim.generation);
  return true;
}

export function releaseOwner(owner: symbol) {
  for (const record of records.values()) {
    if (record.runOwner !== owner) continue;
    record.runOwner = null;
  }
}
