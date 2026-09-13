import type { ToolActivity } from "@/hooks/agent-chat-utils";
import { isPendingTool } from "@/hooks/active-stream-item";
import type {
  ToolActivityRecord,
  ToolArtifact,
  ToolErrorInfo,
  ToolFileChangeRecord,
  ToolResultStatus,
} from "@/types/agent";
import { isLegacyShellStopError } from "./tool-shell-display";

export interface RenderableTool {
  name: string;
  summary: string;
  domain?: "memory";
  isActive?: boolean;
  pending?: boolean;
  args?: Record<string, unknown>;
  result?: string;
  liveOutput?: string;
  liveElapsedMs?: number;
  is_error?: boolean;
  status?: ToolResultStatus;
  error?: ToolErrorInfo;
  warnings?: string[];
  truncated?: boolean;
  content?: string;
  old_text?: string;
  new_text?: string;
  start_line?: number;
  legacySuccessfulStop?: boolean;
  resolved_path?: string;
  file_changes?: ToolFileChangeRecord[];
  artifacts?: ToolArtifact[];
}

function str(value: unknown, fallback = ""): string {
  return typeof value === "string" ? value : fallback;
}

function toolSummary(tool: ToolActivity): string {
  const args = tool.args;
  if (tool.displaySummary !== undefined) return tool.displaySummary;
  if (tool.name === "load_skill") return "";
  if (tool.name === "bash") return str(args.command);
  if (tool.name === "bash_control") return str(args.session_id);
  if (tool.name === "grep" || tool.name === "glob") return str(args.pattern);
  if (
    tool.name === "read_file" ||
    tool.name === "write_file" ||
    tool.name === "edit_file"
  ) {
    return str(args.path);
  }
  if (tool.name === "list_dir") return str(args.path, ".");
  if (tool.name === "web_search") return str(args.query);
  if (tool.name === "web_fetch") return str(args.url);
  if (tool.name === "create_branch" || tool.name === "checkout_branch")
    return str(args.branch_name);
  if (
    [
      "read_spreadsheet",
      "read_document",
      "write_spreadsheet",
      "write_document",
    ].includes(tool.name)
  ) {
    return str(args.path);
  }
  if (tool.name === "transform_image") return str(args.input_path);
  return JSON.stringify(args).slice(0, 80);
}

export function streamToolToRenderable(
  tool: ToolActivity,
  isActive?: boolean,
): RenderableTool {
  const summary = toolSummary(tool);
  const legacySuccessfulStop = isLegacyShellStopError(
    { ...tool, summary },
    tool.isError,
  );
  return {
    name: tool.name,
    summary,
    domain: tool.domain,
    isActive,
    pending: isPendingTool(tool),
    args: tool.args,
    result: tool.result,
    liveOutput: tool.liveOutput,
    liveElapsedMs: tool.liveElapsedMs,
    is_error: legacySuccessfulStop ? false : tool.isError,
    status: tool.status,
    error: tool.error,
    warnings: tool.warnings,
    truncated: tool.truncated,
    legacySuccessfulStop,
    resolved_path: tool.resolvedPath,
    file_changes: tool.fileChanges,
    content: tool.name === "write_file" ? str(tool.args.content) : undefined,
    old_text: tool.name === "edit_file" ? str(tool.args.old_string) : undefined,
    new_text: tool.name === "edit_file" ? str(tool.args.new_string) : undefined,
    start_line: tool.startLine,
    artifacts: tool.artifacts,
  };
}

export function savedToolToRenderable(
  tool: ToolActivityRecord,
): RenderableTool {
  const isLegacySkillId =
    tool.name === "load_skill" &&
    tool.summary.trimStart().startsWith('{"skill_id":');
  const legacySuccessfulStop = isLegacyShellStopError(tool, tool.is_error);
  return {
    name: tool.name,
    summary: isLegacySkillId ? "" : tool.summary,
    domain: tool.domain,
    args: tool.args,
    result: tool.result,
    is_error: legacySuccessfulStop ? false : tool.is_error,
    status: tool.result_meta?.status,
    error: tool.result_meta?.error,
    warnings: tool.result_meta?.warnings,
    truncated: tool.result_meta?.truncated,
    legacySuccessfulStop,
    resolved_path: tool.resolved_path,
    file_changes: tool.file_changes,
    content: tool.content,
    old_text: tool.old_text,
    new_text: tool.new_text,
    start_line: tool.start_line,
    artifacts: tool.artifacts,
  };
}
