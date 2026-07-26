import type { ToolFileChangeRecord } from "@/types/agent";
import type { ToolActivity } from "./agent-chat-utils";

export function applyToolResult(
  tools: ToolActivity[],
  index: number,
  content: string,
  isError: boolean,
  resolvedPath?: string,
  domain?: "memory",
  affectedPaths?: string[],
  fileChanges?: ToolFileChangeRecord[],
  startLine?: number,
  displaySummary?: string,
): ToolActivity[] {
  const next = [...tools];
  const apply = (i: number) => {
    next[i] = { ...next[i], result: content, isError };
    if (resolvedPath) next[i].resolvedPath = resolvedPath;
    if (domain) next[i].domain = domain;
    if (affectedPaths?.length) next[i].affectedPaths = affectedPaths;
    if (fileChanges?.length) next[i].fileChanges = fileChanges;
    if (startLine !== undefined) next[i].startLine = startLine;
    if (displaySummary !== undefined) next[i].displaySummary = displaySummary;
  };
  if (index >= 0 && index < next.length && !next[index].result) {
    apply(index);
  } else {
    const pendingIndex = next.findIndex((tool) => !tool.result);
    if (pendingIndex >= 0) apply(pendingIndex);
  }
  return next;
}
