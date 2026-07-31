import type { ToolActivityRecord } from "@/types/agent";
import type { FileOperation } from "@/types/file-preview";

const FILE_SEPARATOR = /[/\\]/;
const OFFICE_WRITE = ["write_spreadsheet", "write_document"];
const MAX_TOOL_FILE_CHANGES = 500;

export function fileNameFromPath(path: string): string {
  return path.split(FILE_SEPARATOR).filter(Boolean).pop() ?? path;
}

export function countLines(text?: string): number {
  if (!text) return 0;
  return text.split(/\r?\n/).length;
}

export function toolToFileOperations(
  tool: ToolActivityRecord,
  messageId: string,
  index: number,
  timestamp: string,
): FileOperation[] {
  if (tool.domain === "memory") return [];
  if (tool.file_changes?.length) {
    return tool.file_changes
      .filter((change) => change.path.trim())
      .slice(0, MAX_TOOL_FILE_CHANGES)
      .map((change, pathIndex) => ({
        id: `${messageId}-${index}-${pathIndex}`,
        path: change.path,
        name: fileNameFromPath(change.path),
        type: change.status === "added" ? "write" : "edit",
        timestamp,
        additions: change.additions,
        deletions: change.deletions,
        recordedStatus: change.status,
        recordedDiff: change.diff,
      }));
  }
  if (tool.is_error) return [];
  if ((tool.name === "bash" || tool.name === "bash_write") && tool.affected_paths?.length) {
    return tool.affected_paths
      .filter((path) => path.trim())
      .map((path, pathIndex) => ({
        id: `${messageId}-${index}-${pathIndex}`,
        path,
        name: fileNameFromPath(path),
        type: "write",
        timestamp,
        additions: 0,
        deletions: 0,
      }));
  }

  const path = tool.resolved_path?.trim() || tool.summary.trim();
  if (!path) return [];
  if (tool.name === "write_file" && tool.content != null) {
    return [baseOperation(messageId, index, timestamp, path, {
      type: "write",
      content: tool.content,
      additions: countLines(tool.content),
      deletions: 0,
    })];
  }
  if (tool.name === "edit_file" && tool.old_text != null && tool.new_text != null) {
    return [baseOperation(messageId, index, timestamp, path, {
      type: "edit",
      oldText: tool.old_text,
      newText: tool.new_text,
      startLine: tool.start_line,
      additions: countLines(tool.new_text),
      deletions: countLines(tool.old_text),
    })];
  }
  if (OFFICE_WRITE.includes(tool.name)) {
    return [baseOperation(messageId, index, timestamp, path, {
      type: "write",
      content: tool.content,
      additions: 0,
      deletions: 0,
    })];
  }
  return [];
}

function baseOperation(
  messageId: string,
  index: number,
  timestamp: string,
  path: string,
  details: Partial<FileOperation> & Pick<FileOperation, "type" | "additions" | "deletions">,
): FileOperation {
  return {
    id: `${messageId}-${index}`,
    path,
    name: fileNameFromPath(path),
    timestamp,
    ...details,
  };
}
