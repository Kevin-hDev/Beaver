import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { cleanupTauriListener } from "@/lib/tauri-listen";
import { AGENT_SESSIONS_CHANGED } from "@/hooks/agent-session-events";
import { isHiddenAgentTool } from "@/lib/hidden-agent-tools";
import { toolsToRecords, type ToolActivity } from "./agent-chat-utils";
import {
  addChangeSummaries,
  childSubagents,
  EMPTY_CHANGE_SUMMARY,
  hasChangeSummary,
  summarizeLastRequestChanges,
  summarizeToolChange,
  visibleTodoRuns,
} from "@/lib/session-summary";
import type { AgentSession, AgentSessionMeta, StreamEvent } from "@/types/agent";
import type { SessionChangeSummary } from "@/lib/session-summary";

interface StreamEnvelope {
  sessionId: string;
  event: StreamEvent;
}

const REFRESH_EVENTS = new Set<StreamEvent["event"]>([
  "done",
  "todoUpdated",
  "planPreviewUpdated",
  "planModeUpdated",
  "subagentSpawned",
  "subagentCompleted",
  "compressionComplete",
]);

const SUMMARY_REFRESH_TOOLS = new Set(["archive_subagent"]);

export function useSessionSummary(sessionId: string | null) {
  const [session, setSession] = useState<AgentSession | null>(null);
  const [subagentSessions, setSubagentSessions] = useState<AgentSessionMeta[]>([]);
  const [liveChanges, setLiveChanges] = useState<{ sessionId: string; summary: SessionChangeSummary } | null>(null);
  const timerRef = useRef<number | null>(null);
  const requestSeqRef = useRef(0);
  const liveToolsRef = useRef<ToolActivity[]>([]);
  const liveRequestChangesRef = useRef<SessionChangeSummary>(EMPTY_CHANGE_SUMMARY);

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
      setSession(null);
      setSubagentSessions([]);
    }
  }, [sessionId]);

  const scheduleRefresh = useCallback((delayMs = 0) => {
    if (timerRef.current !== null) window.clearTimeout(timerRef.current);
    timerRef.current = window.setTimeout(() => void refresh(), delayMs);
  }, [refresh]);

  useEffect(() => {
    let cancelled = false;
    liveToolsRef.current = [];
    liveRequestChangesRef.current = EMPTY_CHANGE_SUMMARY;
    queueMicrotask(() => {
      if (!cancelled) void refresh();
    });
    return () => {
      cancelled = true;
      setLiveChanges(null);
      liveToolsRef.current = [];
      liveRequestChangesRef.current = EMPTY_CHANGE_SUMMARY;
      requestSeqRef.current += 1;
      if (timerRef.current !== null) window.clearTimeout(timerRef.current);
    };
  }, [refresh, sessionId]);

  useEffect(() => {
    if (!sessionId) return;
    const streamUnlisten = listen<StreamEnvelope>("agent-stream-event", (event) => {
      const payload = event.payload;
      if (payload.sessionId !== sessionId) return;
      if (payload.event.event === "toolCall") {
        trackLiveToolCall(
          liveToolsRef.current,
          payload.event.data.name,
          payload.event.data.arguments,
          payload.event.data.domain,
        );
        return;
      }
      if (payload.event.event === "toolResult") {
        const next = applyLiveToolResult(
          liveToolsRef.current,
          payload.event.data.toolCallIndex ?? -1,
          payload.event.data.content,
          payload.event.data.isError,
          payload.event.data.resolvedPath,
          payload.event.data.domain,
          payload.event.data.startLine,
        );
        liveToolsRef.current = next.tools;
        const summary = next.completedTool ? summarizeToolChange(toolsToRecords([next.completedTool])[0]) : EMPTY_CHANGE_SUMMARY;
        if (hasChangeSummary(summary)) {
          liveRequestChangesRef.current = addChangeSummaries(liveRequestChangesRef.current, summary);
          setLiveChanges({ sessionId, summary: liveRequestChangesRef.current });
        }
        if (!payload.event.data.isError && SUMMARY_REFRESH_TOOLS.has(payload.event.data.name)) {
          scheduleRefresh(80);
        }
        return;
      }
      if (payload.event.event === "done" || payload.event.event === "error") {
        liveToolsRef.current = [];
        liveRequestChangesRef.current = EMPTY_CHANGE_SUMMARY;
      }
      if (!REFRESH_EVENTS.has(payload.event.event)) return;
      scheduleRefresh(payload.event.event === "done" ? 300 : 80);
    });
    const sessionUnlisten = listen("agent-session-updated", () => scheduleRefresh(80));
    const refreshFromWindow = () => scheduleRefresh(80);
    window.addEventListener(AGENT_SESSIONS_CHANGED, refreshFromWindow);
    return () => {
      cleanupTauriListener(streamUnlisten);
      cleanupTauriListener(sessionUnlisten);
      window.removeEventListener(AGENT_SESSIONS_CHANGED, refreshFromWindow);
    };
  }, [scheduleRefresh, sessionId]);

  const savedChanges = useMemo(() => summarizeLastRequestChanges(session?.messages ?? []), [session?.messages]);
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

function trackLiveToolCall(
  tools: ToolActivity[],
  name: string,
  args: Record<string, unknown>,
  domain?: "memory",
) {
  if (isHiddenAgentTool(name)) return;
  tools.push({ name, args, domain });
}

function applyLiveToolResult(
  tools: ToolActivity[],
  index: number,
  content: string,
  isError: boolean,
  resolvedPath?: string,
  domain?: "memory",
  startLine?: number,
): { tools: ToolActivity[]; completedTool?: ToolActivity } {
  const next = [...tools];
  const apply = (i: number) => {
    next[i] = { ...next[i], result: content, isError };
    if (resolvedPath) next[i].resolvedPath = resolvedPath;
    if (domain) next[i].domain = domain;
    if (startLine !== undefined) next[i].startLine = startLine;
    return next[i];
  };
  if (index >= 0 && index < next.length && !next[index].result) {
    return { tools: next, completedTool: apply(index) };
  }
  for (let i = 0; i < next.length; i += 1) {
    if (!next[i].result) return { tools: next, completedTool: apply(i) };
  }
  return { tools: next };
}
