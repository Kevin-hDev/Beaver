import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { cleanupTauriListener } from "@/lib/tauri-listen";
import type { AgentSessionMeta } from "@/types/agent";
import { AGENT_SESSIONS_CHANGED, notifyAgentSessionsChanged } from "./agent-session-events";

export function useAgentSessions() {
  const [sessions, setSessions] = useState<AgentSessionMeta[]>([]);
  const [loading, setLoading] = useState(true);

  const refresh = useCallback(async () => {
    try {
      const list = await invoke<AgentSessionMeta[]>("list_agent_sessions");
      setSessions(list);
    } catch {
      setSessions([]);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    // eslint-disable-next-line react-hooks/set-state-in-effect -- fetch→setState is intentional
    void refresh();
  }, [refresh]);

  useEffect(() => {
    const refreshFromEvent = () => {
      void refresh();
    };
    window.addEventListener(AGENT_SESSIONS_CHANGED, refreshFromEvent);
    const unlistenWakeup = listen("wakeup-completed", refreshFromEvent);
    const unlistenGateway = listen("agent-session-updated", refreshFromEvent);
    return () => {
      window.removeEventListener(AGENT_SESSIONS_CHANGED, refreshFromEvent);
      cleanupTauriListener(unlistenWakeup);
      cleanupTauriListener(unlistenGateway);
    };
  }, [refresh]);

  useEffect(() => {
    const unlisten = listen<{ sessionId: string; event: { event: string } }>(
      "agent-stream-event",
      (event) => {
        const e = event.payload?.event?.event;
        if (e === "subagentSpawned" || e === "subagentCompleted") {
          void refresh();
        }
      },
    );
    return () => {
      cleanupTauriListener(unlisten);
    };
  }, [refresh]);

  const create = useCallback(
    async (
      name: string,
      model: string,
      provider: string = "ollama",
      projectId?: string,
      reasoningMode?: string | null,
      supportsThinking?: boolean,
    ) => {
      const session = await invoke<AgentSessionMeta>("create_agent_session", {
        name,
        model,
        provider,
        projectId: projectId ?? null,
        reasoningMode: reasoningMode ?? null,
        supportsThinking: supportsThinking ?? null,
      });
      await refresh();
      return session;
    },
    [refresh],
  );

  const rename = useCallback(async (id: string, name: string) => {
    await invoke("rename_agent_session", { id, name });
    await refresh();
  }, [refresh]);

  const remove = useCallback(async (id: string) => {
    await invoke("delete_agent_session", { id });
    await refresh();
  }, [refresh]);

  const archive = useCallback(async (id: string) => {
    await invoke("archive_agent_session", { id });
    await refresh();
    notifyAgentSessionsChanged();
  }, [refresh]);

  const restore = useCallback(async (id: string) => {
    await invoke("restore_agent_session", { id });
    await refresh();
  }, [refresh]);

  const updateModel = useCallback(
    async (
      id: string,
      model: string,
      provider: string = "ollama",
      reasoningMode?: string | null,
      supportsThinking?: boolean,
    ) => {
      await invoke("update_session_model", {
        id,
        model,
        provider,
        reasoningMode: reasoningMode ?? null,
        supportsThinking: supportsThinking ?? null,
      });
      await refresh();
    },
    [refresh],
  );

  const updateReasoning = useCallback(
    async (id: string, reasoningMode: string | null, supportsThinking?: boolean) => {
      await invoke("update_session_reasoning", {
        id,
        reasoningMode,
        supportsThinking: supportsThinking ?? null,
      });
      await refresh();
    },
    [refresh],
  );

  return {
    sessions,
    loading,
    refresh,
    create,
    rename,
    remove,
    archive,
    restore,
    updateModel,
    updateReasoning,
  };
}
