import type {
  ToolErrorInfo,
  ToolFileChangeRecord,
  ToolResultStatus,
} from "@/types/agent";
import type { ToolActivity } from "./agent-chat-utils";

export interface ToolResultUpdate {
  name: string;
  callIndex: number;
  callId?: string;
  content: string;
  isError: boolean;
  status?: ToolResultStatus;
  error?: ToolErrorInfo;
  warnings?: string[];
  truncated?: boolean;
  resolvedPath?: string;
  domain?: "memory";
  affectedPaths?: string[];
  fileChanges?: ToolFileChangeRecord[];
  startLine?: number;
  displaySummary?: string;
}

export interface AppliedToolResult {
  tools: ToolActivity[];
  appliedIndex: number;
}

export function applyToolResult(
  tools: ToolActivity[],
  update: ToolResultUpdate,
): AppliedToolResult {
  const index = findTarget(tools, update);
  if (index < 0) return { tools, appliedIndex: -1 };

  const next = [...tools];
  next[index] = mergeResult(next[index], update);
  return { tools: next, appliedIndex: index };
}

function findTarget(tools: ToolActivity[], update: ToolResultUpdate): number {
  if (update.callId) {
    const byId = pendingIndices(tools, (tool) => tool.callId === update.callId);
    if (byId.length === 1) return byId[0];
    if (byId.length > 1) {
      const exact = byId.filter((index) =>
        tools[index].callIndex === update.callIndex);
      return exact.length === 1 ? exact[0] : -1;
    }
    if (tools.some((tool) => tool.callId !== undefined)) return -1;
  }

  if (update.callIndex >= 0) {
    const byOriginalIndex = pendingIndices(tools, (tool) =>
      tool.callIndex === update.callIndex);
    if (byOriginalIndex.length === 1) return byOriginalIndex[0];
    if (byOriginalIndex.length > 1) return -1;

    const legacy = tools[update.callIndex];
    if (legacy && legacy.callIndex === undefined
      && legacy.name === update.name && isPending(legacy)) {
      return update.callIndex;
    }
    if (tools.some((tool) => tool.callIndex !== undefined)) return -1;
  }

  return tools.findIndex((tool) =>
    isPending(tool) && tool.name === update.name);
}

function pendingIndices(
  tools: ToolActivity[],
  matches: (tool: ToolActivity) => boolean,
): number[] {
  return tools.flatMap((tool, index) =>
    isPending(tool) && matches(tool) ? [index] : []);
}

function isPending(tool: ToolActivity): boolean {
  return tool.result === undefined && tool.isError === undefined;
}

function mergeResult(tool: ToolActivity, update: ToolResultUpdate): ToolActivity {
  return {
    ...tool,
    result: update.content,
    isError: update.isError,
    status: update.status ?? (update.isError ? "error" : "success"),
    error: update.error,
    warnings: update.warnings,
    truncated: update.truncated,
    liveOutput: undefined,
    liveElapsedMs: undefined,
    resolvedPath: update.resolvedPath ?? tool.resolvedPath,
    domain: update.domain ?? tool.domain,
    affectedPaths: update.affectedPaths?.length
      ? update.affectedPaths : tool.affectedPaths,
    fileChanges: update.fileChanges?.length
      ? update.fileChanges : tool.fileChanges,
    startLine: update.startLine ?? tool.startLine,
    displaySummary: update.displaySummary ?? tool.displaySummary,
  };
}
