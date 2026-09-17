import type { AgentTodoItem, StreamEvent, SubagentInfo } from "@/types/agent";

export type SessionRefreshEvent =
  | "done"
  | "todoUpdated"
  | "planPreviewUpdated"
  | "planModeUpdated"
  | "subagentSpawned"
  | "subagentCompleted"
  | "compressionComplete"
  | "archiveSubagent";

export interface StreamProjectionState {
  todos: AgentTodoItem[] | null;
  subagents: {
    active: SubagentInfo[];
    completed: SubagentInfo[];
    runId?: string;
  };
  sessionRevision: number;
  lastSessionEvent: SessionRefreshEvent | null;
}

export function createStreamProjection(): StreamProjectionState {
  return {
    todos: null,
    subagents: { active: [], completed: [] },
    sessionRevision: 0,
    lastSessionEvent: null,
  };
}

export function applyStreamProjection(
  current: StreamProjectionState,
  event: Extract<StreamEvent, {
    event: "todoUpdated" | "subagentSpawned" | "subagentCompleted";
  }>,
): StreamProjectionState {
  if (event.event === "todoUpdated") {
    return markSessionUpdate({ ...current, todos: event.data.todos }, event.event);
  }

  if (event.event === "subagentSpawned") {
    const runId = event.data.runId;
    const sameRun = !runId || !current.subagents.runId || current.subagents.runId === runId;
    const active = sameRun ? current.subagents.active : [];
    const completed = sameRun ? current.subagents.completed : [];
    const child: SubagentInfo = {
      sessionId: event.data.subagentSessionId,
      name: event.data.subagentName,
      type: event.data.subagentType as "explorer" | "coder",
      status: "running",
      promptPreview: event.data.promptPreview,
      description: event.data.subagentDescription,
      colorKey: event.data.subagentColorKey,
      runId,
      spawnedAt: Date.now(),
    };
    return markSessionUpdate({
      ...current,
      subagents: {
        active: [...active.filter(({ sessionId }) => sessionId !== child.sessionId), child],
        completed: completed.filter(({ sessionId }) => sessionId !== child.sessionId),
        runId,
      },
    }, event.event);
  }

  const found = current.subagents.active.find(
    ({ sessionId }) => sessionId === event.data.subagentSessionId,
  );
  const child: SubagentInfo = {
    sessionId: event.data.subagentSessionId,
    name: found?.name ?? "agent",
    type: found?.type ?? "explorer",
    status: event.data.status,
    promptPreview: found?.promptPreview ?? "",
    description: found?.description ?? "",
    colorKey: found?.colorKey,
    summary: event.data.summary,
    lastActivity: found?.lastActivity,
    runId: event.data.runId ?? found?.runId ?? current.subagents.runId,
  };
  return markSessionUpdate({
    ...current,
    subagents: {
      ...current.subagents,
      active: current.subagents.active.filter(({ sessionId }) => sessionId !== child.sessionId),
      completed: [
        ...current.subagents.completed.filter(({ sessionId }) => sessionId !== child.sessionId),
        child,
      ],
    },
  }, event.event);
}

export function removeProjectedSubagent(
  current: StreamProjectionState,
  sessionId: string,
): StreamProjectionState {
  return {
    ...current,
    subagents: {
      ...current.subagents,
      active: current.subagents.active.filter((item) => item.sessionId !== sessionId),
    },
  };
}

export function refreshEvent(event: StreamEvent): SessionRefreshEvent | null {
  if (event.event === "done" || event.event === "planPreviewUpdated"
    || event.event === "planModeUpdated" || event.event === "compressionComplete") {
    return event.event;
  }
  if (event.event === "toolResult" && !event.data.isError
    && event.data.name === "archive_subagent") return "archiveSubagent";
  return null;
}

export function markSessionUpdate(
  current: StreamProjectionState,
  event: SessionRefreshEvent,
): StreamProjectionState {
  return {
    ...current,
    sessionRevision: current.sessionRevision + 1,
    lastSessionEvent: event,
  };
}
