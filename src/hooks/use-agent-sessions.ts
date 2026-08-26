import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { runReasoningMutation } from "./session-reasoning-mutation";
import { listen } from "@tauri-apps/api/event";
import { cleanupTauriListener } from "@/lib/tauri-listen";
import type { AgentSessionMeta } from "@/types/agent";
import { AGENT_SESSIONS_CHANGED, notifyAgentSessionsChanged } from "./agent-session-events";
import { admissionErrorMessage } from "@/lib/admission-error";
import { showToast } from "@/lib/toast-emitter";
import i18n from "@/i18n";

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
      fastModeEnabled = false,
    ) => {
      const session = await invoke<AgentSessionMeta>("create_agent_session", {
        name,
        model,
        provider,
        projectId: projectId ?? null,
        reasoningMode: reasoningMode ?? null,
        supportsThinking: supportsThinking ?? null,
        fastModeEnabled,
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

  /* `projectId` nul range les conversations qui n'appartiennent à aucun
     projet. Chaque liste garde le sien : rien ne se mélange entre elles. */
  const reorder = useCallback(async (projectId: string | null, ids: string[]) => {
    await invoke("reorder_agent_sessions", { projectId, ids });
    await refresh();
  }, [refresh]);

  const reorderPinned = useCallback(async (ids: string[]) => {
    await invoke("reorder_pinned_agent_sessions", { ids });
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

  /* Une seule commande côté interface : c'est l'état connu de la session qui
     décide du sens, et la liste rechargée confirme le résultat. */
  const togglePin = useCallback(async (id: string) => {
    const pinned = sessions.some((s) => s.id === id && Boolean(s.pinned_at));
    await invoke(pinned ? "unpin_agent_session" : "pin_agent_session", { id });
    await refresh();
    notifyAgentSessionsChanged();
  }, [refresh, sessions]);

  const updateModel = useCallback(
    async (
      id: string,
      model: string,
      provider: string = "ollama",
      reasoningMode?: string | null,
      supportsThinking?: boolean,
    ) => {
      try {
        await invoke("update_session_model", {
          id,
          model,
          provider,
          reasoningMode: reasoningMode ?? null,
          supportsThinking: supportsThinking ?? null,
        });
        await refresh();
      } catch (error) {
        showToast(admissionErrorMessage(error, i18n.t, "errors.sessionSaveFailed"), "error");
      }
    },
    [refresh],
  );

  const updateReasoning = useCallback(
    async (id: string, reasoningMode: string | null, supportsThinking?: boolean) => {
      try {
        await runReasoningMutation(id, async () => {
          await invoke("update_session_reasoning", {
            id,
            reasoningMode,
            supportsThinking: supportsThinking ?? null,
          });
        });
        await refresh();
      } catch (error) {
        showToast(admissionErrorMessage(error, i18n.t, "errors.sessionSaveFailed"), "error");
      }
    },
    [refresh],
  );

  const updateContinuity = useCallback(
    async (id: string, setting: "off" | "local" | "remote") => {
      try {
        await invoke("update_session_continuity", { id, setting });
        await refresh();
      } catch (error) {
        showToast(admissionErrorMessage(error, i18n.t, "errors.sessionSaveFailed"), "error");
      }
    },
    [refresh],
  );

  return {
    sessions,
    loading,
    refresh,
    create,
    rename,
    reorder,
    reorderPinned,
    remove,
    archive,
    restore,
    togglePin,
    updateModel,
    updateReasoning,
    updateContinuity,
  };
}
