import { useState, useEffect, useCallback, useRef } from "react";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import type { AgentSessionMeta, SubagentInfo, StreamEvent } from "@/types/agent";

interface StreamEnvelope {
  sessionId: string;
  event: StreamEvent;
}

interface StoreEntry {
  active: SubagentInfo[];
  completed: SubagentInfo[];
  runId?: string;
}

const MAX_STORE_ENTRIES = 32;
const globalStore = new Map<string, StoreEntry>();

function evictGlobalStore() {
  while (globalStore.size > MAX_STORE_ENTRIES) {
    const oldest = globalStore.keys().next().value;
    if (oldest) globalStore.delete(oldest);
  }
}

function subagentSignature(items: SubagentInfo[]) {
  return items
    .map((item) => `${item.sessionId}:${item.status}:${item.runId ?? ""}:${item.summary ?? ""}`)
    .join("|");
}

let globalListenerPromise: Promise<void> | null = null;

function ensureGlobalListener() {
  if (globalListenerPromise) return globalListenerPromise;
  globalListenerPromise = listen<StreamEnvelope>("agent-stream-event", (event) => {
    const parentId = event.payload.sessionId;
    const e = event.payload.event;

    if (e.event === "subagentSpawned") {
      const runId = e.data.runId;
      const store = globalStore.get(parentId) ?? { active: [], completed: [], runId };
      if (runId && store.runId && store.runId !== runId) {
        store.completed = [];
        store.active = [];
      }
      store.runId = runId;
      store.completed = store.completed.filter(
        (s) => s.sessionId !== e.data.subagentSessionId,
      );
      store.active = [
        ...store.active.filter((s) => s.sessionId !== e.data.subagentSessionId),
        {
          sessionId: e.data.subagentSessionId,
          name: e.data.subagentName,
          type: e.data.subagentType as "explorer" | "coder",
          status: "running",
          promptPreview: e.data.promptPreview,
          description: e.data.subagentDescription,
          colorKey: e.data.subagentColorKey,
          runId,
          spawnedAt: Date.now(),
        },
      ];
      globalStore.set(parentId, store);
      evictGlobalStore();
    }

    if (e.event === "subagentCompleted") {
      const store = globalStore.get(parentId) ?? { active: [], completed: [] };
      const found = store.active.find(
        (s) => s.sessionId === e.data.subagentSessionId,
      );
      store.active = store.active.filter(
        (s) => s.sessionId !== e.data.subagentSessionId,
      );
      store.completed = [
        ...store.completed,
        {
          sessionId: e.data.subagentSessionId,
          name: found?.name ?? "agent",
          type: found?.type ?? "explorer",
          status: e.data.status,
          promptPreview: found?.promptPreview ?? "",
          description: found?.description ?? "",
          colorKey: found?.colorKey,
          summary: e.data.summary,
          lastActivity: found?.lastActivity,
          runId: e.data.runId ?? found?.runId ?? store.runId,
        },
      ];
      globalStore.set(parentId, store);
      evictGlobalStore();
    }
  }).then(() => { /* listener active */ });
  return globalListenerPromise;
}

export function useSubagents(parentSessionId: string | undefined) {
  const [active, setActive] = useState<SubagentInfo[]>([]);
  const [completed, setCompleted] = useState<SubagentInfo[]>([]);
  const intervalRef = useRef<ReturnType<typeof setInterval> | null>(null);

  useEffect(() => {
    if (!parentSessionId) {
      let cancelled = false;
      queueMicrotask(() => {
        if (cancelled) return;
        setActive([]);
        setCompleted([]);
      });
      return () => { cancelled = true; };
    }

    void ensureGlobalListener();
    void invoke<AgentSessionMeta[]>("list_subagents", {
      parentSessionId,
      runId: null,
    }).then((items) => {
      const mapped = items.map((item) => ({
        sessionId: item.id,
        name: item.name,
        type: item.subagent_type ?? "explorer",
        status: item.subagent_status ?? "completed",
        promptPreview: "",
        description: item.subagent_description,
        colorKey: item.subagent_color_key,
        summary: item.subagent_summary,
        lastActivity: item.subagent_last_activity,
        runId: item.subagent_run_id,
      }));
      setCompleted(mapped.filter((item) => item.status !== "running"));
      setActive(mapped.filter((item) => item.status === "running"));
    }).catch((e) => console.warn("list_subagents failed:", e));

    const sync = () => {
      const store = globalStore.get(parentSessionId);
      if (store) {
        setActive([...store.active]);
        setCompleted((prev) => {
          if (
            store.completed.length > 0
            && subagentSignature(store.completed) !== subagentSignature(prev)
          ) {
            return [...store.completed];
          }
          return prev;
        });
      }
    };

    sync();
    intervalRef.current = setInterval(sync, 500);

    return () => {
      if (intervalRef.current) clearInterval(intervalRef.current);
    };
  }, [parentSessionId]);

  const cancelSubagent = useCallback(
    async (sessionId: string) => {
      await invoke("cancel_subagent", { subagentSessionId: sessionId });
      if (parentSessionId) {
        const store = globalStore.get(parentSessionId);
        if (store) {
          store.active = store.active.filter((s) => s.sessionId !== sessionId);
        }
      }
      setActive((prev) => prev.filter((s) => s.sessionId !== sessionId));
    },
    [parentSessionId],
  );

  return { active, completed, cancelSubagent };
}
