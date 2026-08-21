import { useTranslation } from "react-i18next";
import type { ToolActivity } from "@/hooks/agent-chat-utils";
import {
  toolErrorHasLocalizedMessage,
  toolErrorMessage,
  toolErrorResultIsMachineCode,
} from "@/lib/tool-error-message";
import { sanitizeToolErrorDetails } from "@/lib/tool-error-sanitize";
import type {
  ToolActivityRecord,
  ToolErrorInfo,
  ToolResultStatus,
} from "@/types/agent";
import { ContentPreview, DiffPreview, WebResultsPreview } from "./tool-previews";
import {
  DocumentResultPreview,
  ReadSpreadsheetPreview,
  WriteDocumentPreview,
  WriteSpreadsheetPreview,
} from "./tool-office-previews";
import { ToolItem } from "./tool-item";
import { toolDisplayInfo } from "./tool-display";
import { isLegacyShellStopError, shellCommandPreview } from "./tool-shell-display";
import { isAdmissionError } from "@/lib/admission-error";

export interface RenderableTool {
  name: string;
  summary: string;
  domain?: "memory";
  isActive?: boolean;
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
}

function str(v: unknown, fallback = ""): string {
  return typeof v === "string" ? v : fallback;
}

function toolSummary(t: ToolActivity): string {
  const a = t.args;
  if (t.displaySummary !== undefined) return t.displaySummary;
  if (t.name === "load_skill") return "";
  if (t.name === "bash") return str(a.command);
  if (t.name === "bash_control") return str(a.session_id);
  if (t.name === "grep" || t.name === "glob") return str(a.pattern);
  if (t.name === "read_file" || t.name === "write_file") return str(a.path);
  if (t.name === "edit_file") return str(a.path);
  if (t.name === "list_dir") return str(a.path, ".");
  if (t.name === "web_search") return str(a.query);
  if (t.name === "web_fetch") return str(a.url);
  if (t.name === "create_branch" || t.name === "checkout_branch") return str(a.branch_name);
  if (["read_spreadsheet", "read_document", "write_spreadsheet", "write_document"].includes(t.name)) {
    return str(a.path);
  }
  if (t.name === "transform_image") return str(a.input_path);
  return JSON.stringify(a).slice(0, 80);
}

export function streamToolToRenderable(t: ToolActivity, isActive?: boolean): RenderableTool {
  const summary = toolSummary(t);
  const legacySuccessfulStop = isLegacyShellStopError({ ...t, summary }, t.isError);
  return {
    name: t.name,
    summary,
    domain: t.domain,
    isActive,
    args: t.args,
    result: t.result,
    liveOutput: t.liveOutput,
    liveElapsedMs: t.liveElapsedMs,
    is_error: legacySuccessfulStop ? false : t.isError,
    status: t.status,
    error: t.error,
    warnings: t.warnings,
    truncated: t.truncated,
    legacySuccessfulStop,
    resolved_path: t.resolvedPath,
    content: t.name === "write_file" ? str(t.args.content) : undefined,
    old_text: t.name === "edit_file" ? str(t.args.old_string) : undefined,
    new_text: t.name === "edit_file" ? str(t.args.new_string) : undefined,
    start_line: t.startLine,
  };
}

export function savedToolToRenderable(t: ToolActivityRecord): RenderableTool {
  const isLegacySkillId = t.name === "load_skill"
    && t.summary.trimStart().startsWith('{"skill_id":');
  const legacySuccessfulStop = isLegacyShellStopError(t, t.is_error);
  return {
    name: t.name,
    summary: isLegacySkillId ? "" : t.summary,
    domain: t.domain,
    args: t.args,
    result: t.result,
    is_error: legacySuccessfulStop ? false : t.is_error,
    status: t.result_meta?.status,
    error: t.result_meta?.error,
    warnings: t.result_meta?.warnings,
    truncated: t.result_meta?.truncated,
    legacySuccessfulStop,
    resolved_path: t.resolved_path,
    content: t.content,
    old_text: t.old_text,
    new_text: t.new_text,
    start_line: t.start_line,
  };
}

export function ToolDetailRow({
  tool,
  previousTools,
  isActive,
  onFilePreview,
  projectPath,
}: {
  tool: RenderableTool;
  previousTools: RenderableTool[];
  isActive?: boolean;
  onFilePreview?: (path: string) => void;
  projectPath?: string;
}) {
  const { t } = useTranslation();
  const skipWrite = tool.name === "write_file"
    && previousTools.some((prev) => prev.name === "edit_file" && prev.summary === tool.summary);
  const done = tool.result !== undefined || tool.is_error !== undefined;
  const operations = tool.content ?? tool.args?.operations;
  const documentContent = tool.content ?? tool.args?.content;
  const display = toolDisplayInfo(tool, projectPath, t);
  const rawResult = tool.legacySuccessfulStop
    ? t("agentLocal.toolActivity.processStoppedResult")
    : tool.result ?? tool.liveOutput;
  const errorMessage = tool.is_error
    ? toolErrorMessage(tool.name, tool.result ?? "", tool.error, t)
    : undefined;
  const notices = [
    ...(tool.warnings ?? []),
    ...(tool.truncated ? [t("agentLocal.toolActivity.resultTruncated")] : []),
  ].map(sanitizeToolErrorDetails);
  const resultDetails = tool.is_error
    && (isAdmissionError(tool.result)
      || toolErrorHasLocalizedMessage(tool.error)
      || toolErrorResultIsMachineCode(tool.result))
    ? ""
    : tool.is_error
    ? sanitizeToolErrorDetails(tool.result ?? "")
    : rawResult;
  const result = [resultDetails, ...notices].filter(Boolean).join("\n\n");
  const showWebPreview = (tool.name === "web_search" || tool.name === "web_fetch")
    && tool.result
    && !tool.is_error;

  return (
    <ToolItem
      name={tool.name}
      summary={tool.summary}
      icon={display.icon}
      displayName={display.label}
      displaySummary={display.summary}
      dir={display.dir}
      fileName={display.fileName}
      additions={display.additions}
      deletions={display.deletions}
      done={done}
      isActive={isActive}
      isError={tool.is_error}
      errorMessage={errorMessage}
      result={result || undefined}
      forceResultPreview={notices.length > 0}
      commandPreview={shellCommandPreview(tool, previousTools)}
      elapsedMs={tool.result === undefined ? tool.liveElapsedMs : undefined}
      previewPath={tool.resolved_path}
      onFilePreview={onFilePreview}
    >
      {tool.name === "write_file" && tool.content && !skipWrite && (
        <ContentPreview content={tool.content} path={tool.summary} />
      )}
      {tool.old_text != null && tool.new_text != null && (
        <DiffPreview
          oldText={tool.old_text}
          newText={tool.new_text}
          path={tool.summary}
          startLine={tool.start_line}
        />
      )}
      {showWebPreview && tool.result && (
        <WebResultsPreview content={tool.result} isSearch={tool.name === "web_search"} />
      )}
      {tool.name === "read_spreadsheet" && tool.result && !tool.is_error && (
        <ReadSpreadsheetPreview result={tool.result} />
      )}
      {tool.name === "read_document" && tool.result && !tool.is_error && (
        <DocumentResultPreview result={tool.result} />
      )}
      {tool.name === "write_spreadsheet" && tool.result && !tool.is_error && operations != null && (
        <WriteSpreadsheetPreview operations={operations} />
      )}
      {tool.name === "write_document" && tool.result && !tool.is_error && documentContent != null && (
        <WriteDocumentPreview content={documentContent} />
      )}
    </ToolItem>
  );
}
