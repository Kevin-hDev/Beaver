import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { cleanupTauriListener } from "@/lib/tauri-listen";
import { AGENT_SESSIONS_CHANGED } from "@/hooks/agent-session-events";
import { isHiddenAgentTool } from "@/lib/hidden-agent-tools";
import { toolsToRecords, type ToolActivity } from "./agent-chat-utils";
import { applyToolResult } from "./agent-chat-tool-results";
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
          payload.event.data.toolCallIndex,
          payload.event.data.toolCallId,
        );
        return;
      }
      if (payload.event.event === "toolResult") {
        if (isHiddenAgentTool(payload.event.data.name)) return;
        const next = applyToolResult(liveToolsRef.current, {
          name: payload.event.data.name,
          callIndex: payload.event.data.toolCallIndex ?? -1,
          callId: payload.event.data.toolCallId,
          content: payload.event.data.content,
          isError: payload.event.data.isError,
          status: payload.event.data.status,
          error: payload.event.data.error,
          warnings: payload.event.data.warnings,
          truncated: payload.event.data.truncated,
          resolvedPath: payload.event.data.resolvedPath,
          domain: payload.event.data.domain,
          affectedPaths: payload.event.data.affectedPaths,
          fileChanges: payload.event.data.fileChanges,
          startLine: payload.event.data.startLine,
          displaySummary: payload.event.data.displaySummary,
        });
        liveToolsRef.current = next.tools;
        const completed = next.tools[next.appliedIndex];
        const summary = completed
          ? summarizeToolChange(toolsToRecords([completed])[0])
          : EMPTY_CHANGE_SUMMARY;
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
  callIndex?: number,
  callId?: string,
) {
  if (isHiddenAgentTool(name)) return;
  tools.push({ name, args, domain, callIndex, callId });
}
