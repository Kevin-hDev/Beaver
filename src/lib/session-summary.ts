import { countLines } from "./file-preview-utils";
import type {
  AgentMessage,
  AgentSession,
  AgentSessionMeta,
  AgentTodoRun,
  SubagentInfo,
  ToolActivityRecord,
} from "@/types/agent";

export interface SessionChangeSummary {
  additions: number;
  deletions: number;
  files: number;
}

export const EMPTY_CHANGE_SUMMARY: SessionChangeSummary = {
  additions: 0,
  deletions: 0,
  files: 0,
};

export function summarizeLastRequestChanges(messages: AgentMessage[]): SessionChangeSummary {
  for (let index = messages.length - 1; index >= 0; index -= 1) {
    if (messages[index].role !== "assistant") continue;
    const summary = summarizeToolChanges(toolsFromMessage(messages[index]));
    if (hasChangeSummary(summary)) return summary;
  }
  return EMPTY_CHANGE_SUMMARY;
}

function summarizeToolChanges(tools: ToolActivityRecord[]): SessionChangeSummary {
  return tools.reduce<SessionChangeSummary>((summary, tool) => {
    const next = summarizeToolChange(tool);
    return addChangeSummaries(summary, next);
  }, EMPTY_CHANGE_SUMMARY);
}

export function summarizeToolChange(tool: ToolActivityRecord): SessionChangeSummary {
  if (tool.is_error || tool.domain === "memory") return EMPTY_CHANGE_SUMMARY;
  if (tool.name === "write_file" && tool.content != null) {
    return { additions: countLines(tool.content), deletions: 0, files: 1 };
  }
  if (tool.name === "edit_file" && tool.old_text != null && tool.new_text != null) {
    return {
      additions: countLines(tool.new_text),
      deletions: countLines(tool.old_text),
      files: 1,
    };
  }
  return EMPTY_CHANGE_SUMMARY;
}

export function addChangeSummaries(
  left: SessionChangeSummary,
  right: SessionChangeSummary,
): SessionChangeSummary {
  return {
    additions: left.additions + right.additions,
    deletions: left.deletions + right.deletions,
    files: left.files + right.files,
  };
}

export function hasChangeSummary(summary: SessionChangeSummary): boolean {
  return summary.files > 0 || summary.additions > 0 || summary.deletions > 0;
}

export function visibleTodoRuns(session: Pick<AgentSession, "todo_runs"> | null): AgentTodoRun[] {
  return (session?.todo_runs ?? []).filter((run) => run.status === "active" || run.status === "paused");
}

export function childSubagents(parentSessionId: string, sessions: AgentSessionMeta[]): SubagentInfo[] {
  return sessions
    .filter((session) => session.parent_session_id === parentSessionId)
    .map((session) => ({
      sessionId: session.id,
      name: session.name,
      type: session.subagent_type ?? "explorer",
      status: session.subagent_status ?? "completed",
      promptPreview: "",
      description: session.subagent_description,
      colorKey: session.subagent_color_key,
      summary: session.subagent_summary,
      lastActivity: session.subagent_last_activity,
      runId: session.subagent_run_id,
    }));
}

function toolsFromMessage(message: AgentMessage): ToolActivityRecord[] {
  if (message.segments && message.segments.length > 0) {
    return message.segments.flatMap((segment) => segment.tools);
  }
  return message.tool_activities ?? [];
}
