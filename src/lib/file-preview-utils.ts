import type { AgentMessage, ToolActivityRecord } from "@/types/agent";
import type { FileOperation, FileOperationGroups } from "@/types/file-preview";
import { inferSavedToolPaths } from "./tool-file-path";
import { sumFileOperations, toolToFileOperations } from "./file-preview-operation-builder";

export { countLines, fileNameFromPath } from "./file-preview-operation-builder";

export function shortPath(path: string, baseDir?: string): string {
  if (!baseDir) return path;
  const normalizedPath = path.replaceAll("\\", "/");
  const normalizedBase = baseDir.replaceAll("\\", "/").replace(/\/$/, "");
  return normalizedPath.startsWith(`${normalizedBase}/`)
    ? normalizedPath.slice(normalizedBase.length + 1)
    : path;
}

const MAX_FILE_OPERATIONS = 500;
// Affichage par fichier ; distinct de la collecte Rust plafonnée à 500 fichiers.
const MAX_PREVIEW_FILE_CHANGES = 10;

interface CollectFileOperationsOptions {
  liveTools?: ToolActivityRecord[];
  baseDir?: string;
}

export function normalizeFileOperationPath(path: string): string {
  return path.replaceAll("\\", "/").replace(/\/+$/, "");
}

export function fullFileOperationPath(path: string, baseDir?: string): string {
  const normalizedPath = normalizeFileOperationPath(path);
  if (!baseDir || isAbsolutePath(normalizedPath)) return normalizedPath;
  const normalizedBase = normalizeFileOperationPath(baseDir);
  return `${normalizedBase}/${normalizedPath}`.replace(/\/+/g, "/");
}

export function collectFileOperations(
  messages: AgentMessage[],
  options: CollectFileOperationsOptions = {},
): FileOperation[] {
  return collectFileOperationGroups(messages, options).all;
}

export function collectMessageFileOperations(
  message: AgentMessage,
  baseDir?: string,
): FileOperation[] {
  return collectLatestToolOperations(
    toolsFromMessage(message),
    message.id,
    message.timestamp,
    baseDir,
  );
}

export function collectFileOperationGroups(
  messages: AgentMessage[],
  options: CollectFileOperationsOptions = {},
): FileOperationGroups {
  const byPath = new Map<string, FileOperation>();
  if (options.liveTools?.length) {
    appendToolOperations(
      byPath,
      options.liveTools,
      "live",
      new Date().toISOString(),
      options.baseDir,
    );
  }

  const latestLive = collectLatestToolOperations(
    options.liveTools ?? [],
    "live",
    new Date().toISOString(),
    options.baseDir,
  );

  let latest = latestLive;
  for (let i = messages.length - 1; i >= 0 && byPath.size < MAX_FILE_OPERATIONS; i--) {
    if (latest.length === 0) {
      latest = collectLatestToolOperations(
        toolsFromMessage(messages[i]),
        messages[i].id,
        messages[i].timestamp,
        options.baseDir,
      );
    }
    appendToolOperations(
      byPath,
      toolsFromMessage(messages[i]),
      messages[i].id,
      messages[i].timestamp,
      options.baseDir,
    );
  }

  return { all: Array.from(byPath.values()), latest };
}

function toolsFromMessage(message: AgentMessage): ToolActivityRecord[] {
  const segmentTools = message.segments
    ?.flatMap((segment) => inferSavedToolPaths(segment.tools))
    ?? [];
  return segmentTools.length > 0 ? segmentTools : message.tool_activities ?? [];
}

/** Une ligne par fichier : dernier contenu, total et détail borné des changements. */
function appendToolOperations(
  byPath: Map<string, FileOperation>,
  tools: ToolActivityRecord[],
  messageId: string,
  timestamp: string,
  baseDir: string | undefined,
) {
  const inferred = inferSavedToolPaths(tools);
  for (let index = inferred.length - 1; index >= 0; index--) {
    if (byPath.size >= MAX_FILE_OPERATIONS) return;
    const tool = inferred[index];
    const operations = toolToFileOperations(tool, messageId, index, timestamp);
    for (const operation of operations) {
      if (byPath.size >= MAX_FILE_OPERATIONS) return;
      const key = fullFileOperationPath(operation.path, baseDir);
      if (!key) continue;
      const newer = byPath.get(key);
      byPath.set(key, newer
        ? addOlderChange(newer, operation)
        : { ...operation, id: `file:${messageId}:${key}` });
    }
  }
}

function addOlderChange(file: FileOperation, older: FileOperation): FileOperation {
  const changes = file.changes ?? [file];
  const total = { ...file, ...sumFileOperations([file, older]) };
  if (changes.length < MAX_PREVIEW_FILE_CHANGES) return { ...total, changes: [...changes, older] };
  const hidden = file.olderChanges ?? { count: 0, additions: 0, deletions: 0 };
  return { ...total, olderChanges: { count: hidden.count + 1, ...sumFileOperations([hidden, older]) } };
}

function collectLatestToolOperations(
  tools: ToolActivityRecord[],
  messageId: string,
  timestamp: string,
  baseDir: string | undefined,
): FileOperation[] {
  const byPath = new Map<string, FileOperation>();
  appendToolOperations(byPath, tools, messageId, timestamp, baseDir);
  return Array.from(byPath.values());
}

function isAbsolutePath(path: string): boolean {
  return path.startsWith("/") || /^[A-Za-z]:\//.test(path);
}
