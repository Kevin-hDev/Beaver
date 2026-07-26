import { useCallback, useEffect, useMemo, useSyncExternalStore } from "react";
import { agentStreamManager } from "./agent-stream-manager";
import type { StreamActivity } from "./agent-stream-activity";

export interface SessionActivityState {
  runningIds: Set<string>;
  unreadIds: Set<string>;
}

const EMPTY_STATE: SessionActivityState = {
  runningIds: new Set(),
  unreadIds: new Set(),
};
const MAX_TRACKED_SESSION_IDS = 128;

let storeState: SessionActivityState = EMPTY_STATE;
let currentSelectedId: string | null = null;
let managerUnsubscribe: (() => void) | null = null;
let nextListenerId = 1;
const listeners = new Map<number, () => void>();

export function reduceSessionActivity(
  state: SessionActivityState,
  activity: StreamActivity,
  selectedId: string | null,
  _visibleIds: Set<string>,
): SessionActivityState {
  const runningIds = new Set(state.runningIds);
  const unreadIds = new Set(state.unreadIds);

  if (activity.isStreaming) {
    runningIds.add(activity.sessionId);
    unreadIds.delete(activity.sessionId);
    return { runningIds, unreadIds };
  }

  const wasRunning = runningIds.delete(activity.sessionId);
  if (activity.completed && wasRunning && selectedId !== activity.sessionId) {
    unreadIds.add(activity.sessionId);
  }
  if (selectedId === activity.sessionId) unreadIds.delete(activity.sessionId);
  return trimSessionActivity({ runningIds, unreadIds });
}

export function cleanupSessionActivity(
  state: SessionActivityState,
  visibleIds: Set<string>,
  _selectedId: string | null,
): SessionActivityState {
  const runningIds = filterVisible(state.runningIds, visibleIds);
  const unreadIds = trimSet(state.unreadIds);
  return { runningIds, unreadIds };
}

export function useSessionActivityIndicators(sessionIds: string[], selectedId: string | null) {
  const visibleIds = useMemo(() => new Set(sessionIds), [sessionIds]);

  useEffect(() => {
    currentSelectedId = selectedId;
    updateStore((current) => cleanupSessionActivity(current, visibleIds, selectedId));
    for (const id of visibleIds) {
      const activity = agentStreamManager.getActivity(id);
      if (activity) {
        updateStore((current) =>
          reduceSessionActivity(current, activity, selectedId, visibleIds));
      }
    }
  }, [selectedId, visibleIds]);

  const state = useSyncExternalStore(subscribeStore, getStoreState, getStoreState);

  const markViewed = useCallback((sessionId: string) => {
    updateStore((current) => {
      if (!current.unreadIds.has(sessionId)) return current;
      const unreadIds = new Set(current.unreadIds);
      unreadIds.delete(sessionId);
      return { runningIds: current.runningIds, unreadIds };
    });
  }, []);

  return { ...state, markViewed };
}

export function markSessionRunning(sessionId: string) {
  updateStore((current) => {
    if (!sessionId) return current;
    const runningIds = new Set(current.runningIds);
    const unreadIds = new Set(current.unreadIds);
    runningIds.add(sessionId);
    unreadIds.delete(sessionId);
    return trimSessionActivity({ runningIds, unreadIds });
  });
}

export function markSessionComplete(sessionId: string) {
  updateStore((current) => {
    if (!sessionId) return current;
    const runningIds = new Set(current.runningIds);
    const unreadIds = new Set(current.unreadIds);
    runningIds.delete(sessionId);
    if (currentSelectedId !== sessionId) unreadIds.add(sessionId);
    return trimSessionActivity({ runningIds, unreadIds });
  });
}

export function clearSessionRunning(sessionId: string) {
  updateStore((current) => {
    if (!sessionId || !current.runningIds.has(sessionId)) return current;
    const runningIds = new Set(current.runningIds);
    runningIds.delete(sessionId);
    return trimSessionActivity({ runningIds, unreadIds: current.unreadIds });
  });
}

function filterVisible(ids: Set<string>, visibleIds: Set<string>): Set<string> {
  const next = new Set<string>();
  for (const id of ids) {
    if (visibleIds.has(id)) next.add(id);
  }
  return next;
}

function subscribeStore(listener: () => void): () => void {
  ensureActivitySubscription();
  while (listeners.size >= 16) {
    const first = listeners.keys().next().value;
    if (first === undefined) break;
    listeners.delete(first);
  }
  const id = nextListenerId++;
  listeners.set(id, listener);
  return () => {
    listeners.delete(id);
  };
}

function getStoreState(): SessionActivityState {
  return storeState;
}

function ensureActivitySubscription() {
  if (managerUnsubscribe) return;
  managerUnsubscribe = agentStreamManager.subscribeActivity((activity) => {
    updateStore((current) =>
      reduceSessionActivity(current, activity, currentSelectedId, new Set([activity.sessionId])));
  });
}

function updateStore(reducer: (state: SessionActivityState) => SessionActivityState) {
  const next = reducer(storeState);
  if (next === storeState) return;
  storeState = next;
  for (const listener of listeners.values()) listener();
}

function trimSessionActivity(state: SessionActivityState): SessionActivityState {
  return {
    runningIds: trimSet(state.runningIds),
    unreadIds: trimSet(state.unreadIds),
  };
}

function trimSet(ids: Set<string>): Set<string> {
  const next = new Set(ids);
  while (next.size > MAX_TRACKED_SESSION_IDS) {
    const first = next.keys().next().value;
    if (first === undefined) break;
    next.delete(first);
  }
  return next;
}
