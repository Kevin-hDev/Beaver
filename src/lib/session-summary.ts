import { sumFileOperations } from "./file-preview-operation-builder";
import { collectFileOperations } from "./file-preview-utils";
import { normalizeSavedToolHistory } from "./saved-tool-history";
import { planStreamEndArtifacts } from "./stream-end-artifacts";
import type {
  AgentMessage,
  AgentSession,
  AgentSessionMeta,
  AgentTodoRun,
  SubagentInfo,
} from "@/types/agent";
import type { FileOperation } from "@/types/file-preview";

export interface SessionChangeSummary {
  additions: number;
  deletions: number;
  files: number;
}

const EMPTY_CHANGE_SUMMARY: SessionChangeSummary = {
  additions: 0,
  deletions: 0,
  files: 0,
};

export function summarizeLastRequestChanges(messages: AgentMessage[], baseDir?: string): SessionChangeSummary {
  const displayed = normalizeSavedToolHistory(messages);
  // Le résumé live reste l'autorité pendant le stream ; ici on reconstruit les groupes enregistrés.
  const bubbles = planStreamEndArtifacts(displayed, false, "");
  for (let index = displayed.length - 1; index >= 0; index -= 1) {
    const bubble = bubbles.get(displayed[index].id);
    if (!bubble) continue;
    const summary = summarizeFileOperations(collectFileOperations(bubble.messages, { baseDir }));
    if (hasChangeSummary(summary)) return summary;
  }
  return EMPTY_CHANGE_SUMMARY;
}

export function summarizeFileOperations(operations: FileOperation[]): SessionChangeSummary {
  return { ...sumFileOperations(operations), files: operations.length };
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
