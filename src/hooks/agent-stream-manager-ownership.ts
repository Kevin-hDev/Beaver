import { getRecord, records } from "./agent-stream-records";
import {
  claimOwnedStop,
  consumeOwnedStop,
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
  return getRecord(sessionId)?.runOwner === owner;
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

export function discardOwner(owner: symbol) {
  for (const [sessionId, record] of records) {
    if (record.runOwner !== owner) continue;
    stopStreamRecord(sessionId, record, record.activeGeneration);
  }
}
