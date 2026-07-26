import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { AgentSessionMeta } from "@/types/agent";
import { notifyAgentSessionsChanged } from "./agent-session-events";

export function useArchivedAgentSessions() {
  const [sessions, setSessions] = useState<AgentSessionMeta[]>([]);
  const [loading, setLoading] = useState(true);

  const refresh = useCallback(async () => {
    try {
      const list = await invoke<AgentSessionMeta[]>("list_archived_agent_sessions");
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

  const restore = useCallback(async (id: string) => {
    await invoke("restore_agent_session", { id });
    notifyAgentSessionsChanged();
    await refresh();
  }, [refresh]);

  const remove = useCallback(async (id: string) => {
    await invoke("delete_agent_session", { id });
    notifyAgentSessionsChanged();
    await refresh();
  }, [refresh]);

  return { sessions, loading, refresh, restore, remove };
}
