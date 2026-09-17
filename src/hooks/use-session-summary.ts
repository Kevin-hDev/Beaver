import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { cleanupTauriListener } from "@/lib/tauri-listen";
import { AGENT_SESSIONS_CHANGED } from "@/hooks/agent-session-events";
import { collectFileOperations } from "@/lib/file-preview-utils";
import { toolsToRecords } from "./agent-chat-utils";
import { isPendingTool } from "./active-stream-item";
import { agentStreamManager, type StreamSnapshot } from "./agent-stream-manager";
import {
  childSubagents,
  hasChangeSummary,
  summarizeFileOperations,
  summarizeLastRequestChanges,
  visibleTodoRuns,
} from "@/lib/session-summary";
import type { AgentSession, AgentSessionMeta } from "@/types/agent";
import type { SessionChangeSummary } from "@/lib/session-summary";

export function useSessionSummary(sessionId: string | null, baseDir?: string) {
  const [session, setSession] = useState<AgentSession | null>(null);
  const [subagentSessions, setSubagentSessions] = useState<AgentSessionMeta[]>([]);
  const [liveChanges, setLiveChanges] = useState<{ sessionId: string; summary: SessionChangeSummary } | null>(null);
  const timerRef = useRef<number | null>(null);
  const requestSeqRef = useRef(0);
  const streamRevisionRef = useRef(0);

  const refresh = useCallback(async () => {
    const requestSeq = requestSeqRef.current + 1;
    requestSeqRef.current = requestSeq;
    if (!sessionId) {
      setSession(null);
      setSubagentSessions([]);
      return;
    }
    try {
      const [nextSession, children] = await Promise.all([
        invoke<AgentSession>("get_agent_session", { id: sessionId }),
        invoke<AgentSessionMeta[]>("list_subagents", { parentSessionId: sessionId, runId: null }),
      ]);
      if (requestSeqRef.current !== requestSeq) return;
      setSession(nextSession);
      setSubagentSessions(children);
    } catch {
      if (requestSeqRef.current !== requestSeq) return;
      // Une invalidation ratée ne doit pas effacer la dernière donnée confirmée.
    }
  }, [sessionId]);

  const scheduleRefresh = useCallback((delayMs = 0) => {
    if (timerRef.current !== null) window.clearTimeout(timerRef.current);
    timerRef.current = window.setTimeout(() => void refresh(), delayMs);
  }, [refresh]);

  useEffect(() => {
    let cancelled = false;
    streamRevisionRef.current = 0;
    queueMicrotask(() => {
      if (cancelled) return;
      setSession(null);
      setSubagentSessions([]);
      void refresh();
    });
    return () => {
      cancelled = true;
      setLiveChanges(null);
      streamRevisionRef.current = 0;
      requestSeqRef.current += 1;
      if (timerRef.current !== null) window.clearTimeout(timerRef.current);
    };
  }, [refresh, sessionId]);

  useEffect(() => {
    if (!sessionId) return;
    const applySnapshot = (snapshot: StreamSnapshot) => {
      const tools = [
        ...snapshot.completedSegments.flatMap((segment) => segment.tools),
        ...snapshot.currentTools,
      ].filter((tool) => !isPendingTool(tool));
      const summary = summarizeFileOperations(collectFileOperations([], {
        liveTools: toolsToRecords(tools),
        baseDir,
      }));
      if (hasChangeSummary(summary)) setLiveChanges({ sessionId, summary });

      const { sessionRevision, lastSessionEvent } = snapshot.projection;
      if (sessionRevision === streamRevisionRef.current) return;
      streamRevisionRef.current = sessionRevision;
      scheduleRefresh(lastSessionEvent === "done" ? 300 : 80);
    };
    const unsubscribeStream = agentStreamManager.subscribe(sessionId, applySnapshot);
    const snapshot = agentStreamManager.getSnapshot(sessionId);
    if (snapshot) applySnapshot(snapshot);
    const sessionUnlisten = listen("agent-session-updated", () => scheduleRefresh(80));
    const refreshFromWindow = () => scheduleRefresh(80);
    window.addEventListener(AGENT_SESSIONS_CHANGED, refreshFromWindow);
    return () => {
      unsubscribeStream();
      cleanupTauriListener(sessionUnlisten);
      window.removeEventListener(AGENT_SESSIONS_CHANGED, refreshFromWindow);
    };
  }, [baseDir, scheduleRefresh, sessionId]);

  const savedChanges = useMemo(
    () => summarizeLastRequestChanges(session?.messages ?? [], baseDir),
    [baseDir, session?.messages],
  );
  const changes = liveChanges?.sessionId === sessionId && hasChangeSummary(liveChanges.summary)
    ? liveChanges.summary
    : savedChanges;

  return useMemo(() => ({
    session,
    todoRuns: visibleTodoRuns(session),
    plans: session?.plan_runs ?? [],
    subagents: sessionId ? childSubagents(sessionId, subagentSessions) : [],
    changes,
  }), [changes, session, sessionId, subagentSessions]);
}

export type SessionSummaryHookState = ReturnType<typeof useSessionSummary>;
