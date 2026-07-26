import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { AgentSession, AgentTodoItem, StreamEvent } from "@/types/agent";

interface StreamEnvelope {
  sessionId: string;
  event: StreamEvent;
}

type Subscriber = (todos: AgentTodoItem[]) => void;

const MAX_STORE_ENTRIES = 32;
const globalStore = new Map<string, AgentTodoItem[]>();
const subscribers = new Map<string, Set<Subscriber>>();
let globalListenerPromise: Promise<void> | null = null;

function evictGlobalStore() {
  while (globalStore.size > MAX_STORE_ENTRIES) {
    const oldest = globalStore.keys().next().value;
    if (oldest) globalStore.delete(oldest);
  }
}

function ensureGlobalListener() {
  if (globalListenerPromise) return globalListenerPromise;
  globalListenerPromise = listen<StreamEnvelope>("agent-stream-event", (event) => {
    const payload = event.payload;
    if (payload.event.event !== "todoUpdated") return;
    globalStore.set(payload.sessionId, payload.event.data.todos);
    evictGlobalStore();
    for (const subscriber of subscribers.get(payload.sessionId) ?? []) {
      subscriber(payload.event.data.todos);
    }
  }).then(() => { /* listener active */ });
  return globalListenerPromise;
}

export function useTodos(sessionId: string | undefined) {
  const [todos, setTodos] = useState<AgentTodoItem[]>([]);

  useEffect(() => {
    if (!sessionId) {
      let cancelled = false;
      queueMicrotask(() => {
        if (!cancelled) setTodos([]);
      });
      return () => { cancelled = true; };
    }

    let cancelled = false;
    const cached = globalStore.get(sessionId);
    if (cached) {
      queueMicrotask(() => {
        if (!cancelled) setTodos(cached);
      });
    }

    const unsubscribe = subscribeToTodos(sessionId, setTodos);
    return () => {
      cancelled = true;
      unsubscribe();
    };
  }, [sessionId]);

  return todos;
}

function subscribeToTodos(sessionId: string, setTodos: Subscriber) {
  let alive = true;
  void ensureGlobalListener();
  void invoke<AgentSession>("get_agent_session", { id: sessionId })
    .then((session) => {
      if (!alive) return;
      const next = globalStore.get(sessionId) ?? session.todos ?? [];
      setTodos(next);
    })
    .catch(() => {
      if (alive) setTodos([]);
    });

  const set = subscribers.get(sessionId) ?? new Set<Subscriber>();
  set.add(setTodos);
  subscribers.set(sessionId, set);
  return () => {
    alive = false;
    const current = subscribers.get(sessionId);
    current?.delete(setTodos);
    if (current?.size === 0) subscribers.delete(sessionId);
  };
}
