import { getRecord, records } from "./agent-stream-records";
import {
  adoptStreamOwner,
  claimOwnedStop,
  consumeOwnedStop,
  matchesStreamRun,
  ownedGeneration,
  ownsStreamRun,
  releaseOwnedStop,
  type StreamRun,
} from "./agent-stream-run-ownership";
import { stopStreamRecord } from "./agent-stream-stop";

export function ownsRun(sessionId: string, run: StreamRun) {
  const record = getRecord(sessionId);
  return record ? ownsStreamRun(record, run) : false;
}

export function ownsOwner(sessionId: string, owner: symbol) {
  const record = getRecord(sessionId);
  return record?.runOwner === owner && !record.stopRequested;
}

export function adoptOwner(sessionId: string, owner: symbol) {
  const record = getRecord(sessionId);
  return record ? adoptStreamOwner(record, owner) : false;
}

export function matchesRun(sessionId: string, run: StreamRun) {
  const record = getRecord(sessionId);
  return record ? matchesStreamRun(record, run) : false;
}

export function isStopRequested(sessionId: string, run: StreamRun) {
  const record = getRecord(sessionId);
  return record ? matchesStreamRun(record, run) && record.stopRequested : false;
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

export function claimStop(sessionId: string, owner: symbol) {
  const record = getRecord(sessionId);
  return record ? claimOwnedStop(record, owner) : null;
}

export function releaseStop(sessionId: string, owner: symbol, generation: number) {
  const record = getRecord(sessionId);
  if (record) releaseOwnedStop(record, owner, generation);
}

export function completeStop(sessionId: string, owner: symbol, generation: number) {
  const record = getRecord(sessionId);
  if (!record || !consumeOwnedStop(record, owner, generation)) return false;
  stopStreamRecord(sessionId, record, generation);
  return true;
}

export function completeDeferredStop(sessionId: string, run: StreamRun, generation: number) {
  const record = getRecord(sessionId);
  if (!record || !matchesStreamRun(record, run) || !record.stopRequested) return false;
  stopStreamRecord(sessionId, record, generation);
  return true;
}

export function releaseDeferredStop(sessionId: string, run: StreamRun) {
  const record = getRecord(sessionId);
  if (!record || !matchesStreamRun(record, run) || !record.stopRequested) return false;
  record.stopRequested = false;
  record.stoppingGeneration = null;
  return true;
}

export function releaseOwner(owner: symbol) {
  for (const record of records.values()) {
    if (record.runOwner !== owner) continue;
    if (!record.stopRequested) record.runOwner = null;
  }
}
