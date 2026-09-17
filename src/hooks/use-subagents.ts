import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { agentStreamManager, type StreamSnapshot } from "./agent-stream-manager";
import type { AgentSessionMeta, SubagentInfo } from "@/types/agent";

function fromSession(item: AgentSessionMeta): SubagentInfo {
  return {
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
  };
}

function hasLiveSubagents(snapshot: StreamSnapshot | null): snapshot is StreamSnapshot {
  if (!snapshot) return false;
  const { subagents } = snapshot.projection;
  return Boolean(subagents.runId || subagents.active.length || subagents.completed.length);
}

export function useSubagents(parentSessionId: string | undefined) {
  const [active, setActive] = useState<SubagentInfo[]>([]);
  const [completed, setCompleted] = useState<SubagentInfo[]>([]);

  useEffect(() => {
    let cancelled = false;
    let latestSnapshot: StreamSnapshot | null = null;
    queueMicrotask(() => {
      if (cancelled || hasLiveSubagents(latestSnapshot)) return;
      setActive([]);
      setCompleted([]);
    });
    if (!parentSessionId) return () => { cancelled = true; };

    const applySnapshot = (snapshot: StreamSnapshot | null) => {
      if (cancelled || !hasLiveSubagents(snapshot)) return;
      latestSnapshot = snapshot;
      setActive(snapshot.projection.subagents.active);
      if (snapshot.projection.subagents.completed.length > 0) {
        setCompleted(snapshot.projection.subagents.completed);
      }
    };
    const unsubscribe = agentStreamManager.subscribe(parentSessionId, applySnapshot);
    applySnapshot(agentStreamManager.getSnapshot(parentSessionId));

    void invoke<AgentSessionMeta[]>("list_subagents", {
      parentSessionId,
      runId: null,
    }).then((items) => {
      if (cancelled) return;
      const mapped = items.map(fromSession);
      const durableCompleted = mapped.filter((item) => item.status !== "running");
      if (!hasLiveSubagents(latestSnapshot)) {
        setActive(mapped.filter((item) => item.status === "running"));
        setCompleted(durableCompleted);
        return;
      }
      setActive(latestSnapshot.projection.subagents.active);
      setCompleted(latestSnapshot.projection.subagents.completed.length > 0
        ? latestSnapshot.projection.subagents.completed
        : durableCompleted);
    }).catch(() => { /* conserve la dernière liste confirmée */ });

    return () => {
      cancelled = true;
      unsubscribe();
    };
  }, [parentSessionId]);

  const cancelSubagent = useCallback(async (sessionId: string) => {
    await invoke("cancel_subagent", { subagentSessionId: sessionId });
    if (parentSessionId) agentStreamManager.removeSubagent(parentSessionId, sessionId);
    setActive((previous) => previous.filter((item) => item.sessionId !== sessionId));
  }, [parentSessionId]);

  return { active, completed, cancelSubagent };
}
