import { useSyncExternalStore } from "react";

export interface CloneSummaryRun {
  sessionId: string;
  messageId: string;
  operationId: string;
  visible: boolean;
}

const MAX_CLONE_SUMMARY_RUNS = 32;

let runs = new Map<string, CloneSummaryRun>();
let nextListenerId = 1;
const listeners = new Map<number, () => void>();

export function useCloneSummaryRun(sessionId: string): CloneSummaryRun | null {
  return useSyncExternalStore(
    subscribe,
    () => getCloneSummaryRun(sessionId),
    () => getCloneSummaryRun(sessionId),
  );
}

export function getCloneSummaryRun(sessionId: string): CloneSummaryRun | null {
  return runs.get(sessionId) ?? null;
}

export function startCloneSummaryRun(run: CloneSummaryRun) {
  const next = new Map(runs);
  next.set(run.sessionId, run);
  while (next.size > MAX_CLONE_SUMMARY_RUNS) {
    const first = next.keys().next().value;
    if (first === undefined) break;
    next.delete(first);
  }
  runs = next;
  notify();
}

export function setCloneSummaryRunVisible(sessionId: string, visible: boolean) {
  const run = runs.get(sessionId);
  if (!run || run.visible === visible) return;
  runs = new Map(runs).set(sessionId, { ...run, visible });
  notify();
}

export function finishCloneSummaryRun(sessionId: string, operationId: string) {
  const run = runs.get(sessionId);
  if (!run || run.operationId !== operationId) return;
  const next = new Map(runs);
  next.delete(sessionId);
  runs = next;
  notify();
}

function subscribe(listener: () => void): () => void {
  while (listeners.size >= 64) {
    const first = listeners.keys().next().value;
    if (first === undefined) break;
    listeners.delete(first);
  }
  const id = nextListenerId++;
  listeners.set(id, listener);
  return () => listeners.delete(id);
}

function notify() {
  for (const listener of listeners.values()) listener();
}
