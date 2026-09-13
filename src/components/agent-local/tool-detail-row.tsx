import { useTranslation } from "react-i18next";
import {
  toolErrorHasLocalizedMessage,
  toolErrorMessage,
  toolErrorResultIsMachineCode,
} from "@/lib/tool-error-message";
import { sanitizeToolErrorDetails } from "@/lib/tool-error-sanitize";
import {
  ContentPreview,
  DiffPreview,
  WebResultsPreview,
} from "./tool-previews";
import {
  DocumentResultPreview,
  ReadSpreadsheetPreview,
  WriteDocumentPreview,
  WriteSpreadsheetPreview,
} from "./tool-office-previews";
import { ToolItem } from "./tool-item";
import { toolDisplayInfo } from "./tool-display";
import { shellCommandPreview } from "./tool-shell-display";
import { isAdmissionError } from "@/lib/admission-error";
import { ToolArtifacts } from "./tool-artifacts";
import type { RenderableTool } from "./tool-renderable";

export type { RenderableTool } from "./tool-renderable";
export {
  savedToolToRenderable,
  streamToolToRenderable,
} from "./tool-renderable";

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
  const skipWrite =
    tool.name === "write_file" &&
    previousTools.some(
      (prev) => prev.name === "edit_file" && prev.summary === tool.summary,
    );
  const done = tool.result !== undefined || tool.is_error !== undefined;
  const operations = tool.content ?? tool.args?.operations;
  const documentContent = tool.content ?? tool.args?.content;
  const display = toolDisplayInfo(tool, projectPath, t);
  const rawResult = tool.legacySuccessfulStop
    ? t("agentLocal.toolActivity.processStoppedResult")
    : (tool.result ?? tool.liveOutput);
  const errorMessage = tool.is_error
    ? toolErrorMessage(tool.name, tool.result ?? "", tool.error, t)
    : undefined;
  const notices = [
    ...(tool.warnings ?? []),
    ...(tool.truncated ? [t("agentLocal.toolActivity.resultTruncated")] : []),
  ].map(sanitizeToolErrorDetails);
  const resultDetails =
    tool.error?.code === "tool_interrupted"
      ? errorMessage
      : tool.is_error &&
          (isAdmissionError(tool.result) ||
            toolErrorHasLocalizedMessage(tool.error) ||
            toolErrorResultIsMachineCode(tool.result))
        ? ""
        : tool.is_error
          ? sanitizeToolErrorDetails(tool.result ?? "")
          : rawResult;
  const result = [resultDetails, ...notices].filter(Boolean).join("\n\n");
  const showWebPreview =
    (tool.name === "web_search" || tool.name === "web_fetch") &&
    tool.result &&
    !tool.is_error;

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
        <WebResultsPreview
          content={tool.result}
          isSearch={tool.name === "web_search"}
        />
      )}
      {tool.name === "read_spreadsheet" && tool.result && !tool.is_error && (
        <ReadSpreadsheetPreview result={tool.result} />
      )}
      {tool.name === "read_document" && tool.result && !tool.is_error && (
        <DocumentResultPreview result={tool.result} />
      )}
      {tool.name === "write_spreadsheet" &&
        tool.result &&
        !tool.is_error &&
        operations != null && (
          <WriteSpreadsheetPreview operations={operations} />
        )}
      {tool.name === "write_document" &&
        tool.result &&
        !tool.is_error &&
        documentContent != null && (
          <WriteDocumentPreview content={documentContent} />
        )}
      {!!tool.artifacts?.length && (
        <ToolArtifacts
          artifacts={tool.artifacts}
          onFilePreview={onFilePreview}
        />
      )}
    </ToolItem>
  );
}
