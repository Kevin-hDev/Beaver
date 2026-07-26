import type { KeyboardEvent, MouseEvent, ReactNode } from "react";
import { Spinner } from "@/components/ui/icons";
import { CaretDown, CaretUp } from "@/components/ui/icons";
import { isFileTool } from "@/lib/tool-file-path";
import { FileIcon } from "@/components/file-preview/file-icon";
import { ToolIcon } from "./tool-icons";
import { ToolStatusIcon } from "./tool-status-icon";
import { ToolResultCode, ToolResultMarkdown } from "./tool-result-markdown";
import { useCollapsiblePresence } from "./use-collapsible-presence";

const RESULT_PREVIEW_TOOLS = new Set([
  "bash", "grep", "glob", "read_file", "list_dir",
  "read_spreadsheet", "read_document", "read_image",
  "web_search", "web_fetch", "forecast_data_audit", "forecast", "forecast_read",
]);

// Outils dont le résultat texte est rendu en Markdown (style bulle, sans
// couleurs de code). Les autres gardent le rendu texte brut.
const MARKDOWN_RESULT_TOOLS = new Set([
  "bash", "grep", "glob", "list_dir", "web_search", "web_fetch",
]);

const TEXT_RESULT_TOOLS = new Set(["read_file"]);

function hasPreviewContent(children: ReactNode): boolean {
  if (!children) return false;
  if (Array.isArray(children)) return children.some((c) => !!c);
  return true;
}

export function ToolItem({
  name, summary, icon, displayName, displaySummary, dir, fileName,
  additions, deletions, done, isActive, isError, errorMessage, result,
  previewPath, onFilePreview, children,
}: {
  name: string; summary: string; icon?: string; displayName?: string; displaySummary?: string;
  dir?: string; fileName?: string;
  additions?: number; deletions?: number; done: boolean; isActive?: boolean; isError?: boolean; errorMessage?: string;
  result?: string; previewPath?: string; onFilePreview?: (path: string) => void; children?: ReactNode;
}) {
  const hasPreview = hasPreviewContent(children);
  const hasResult = !!result && !isError && !hasPreview && RESULT_PREVIEW_TOOLS.has(name);
  const canToggle = hasPreview || hasResult;
  const showCommandPreview = name === "bash" && hasResult;
  const { open: isOpen, mounted, toggle, onTransitionEnd } = useCollapsiblePresence();
  const targetPath = previewPath?.trim() || summary.trim();
  const clickablePath = isFileTool(name) && targetPath.length > 0 && !!onFilePreview;
  const shownName = displayName ?? name;
  const shownSummary = displaySummary ?? summary;
  const hasFilePath = !!dir || !!fileName;
  const activeClass = isActive ? " stream-active-label" : "";
  const openPreview = () => {
    if (!clickablePath || !onFilePreview) return;
    onFilePreview(targetPath);
  };
  const handlePreviewClick = (e: MouseEvent<HTMLElement>) => {
    if (!clickablePath) return;
    e.stopPropagation();
    openPreview();
  };
  const handlePreviewKeyDown = (e: KeyboardEvent<HTMLElement>) => {
    if (!clickablePath) return;
    if (e.key === "Enter" || e.key === " ") {
      e.preventDefault();
      openPreview();
    }
  };

  const labelButton = canToggle ? (
    <button type="button" className="tb-toggle" onClick={toggle}>
      {icon && <ToolIcon name={icon} size="var(--icon-sm)" className="tb-tool-icon" aria-hidden="true" />}
      <span className={`tb-tool-verb${activeClass}`}>{shownName}</span>
      <span className="tb-arrow tb-tool-arrow" aria-hidden="true">
        {isOpen ? <CaretUp size="var(--icon-sm)" weight="bold" /> : <CaretDown size="var(--icon-sm)" weight="bold" />}
      </span>
    </button>
  ) : (
    <span className="tb-static-label">
      {icon && <ToolIcon name={icon} size="var(--icon-sm)" className="tb-tool-icon" aria-hidden="true" />}
      <span className={`tb-tool-verb${activeClass}`}>{shownName}</span>
    </span>
  );

  // Cas fichier : nom + icône + stats collés à droite, dossiers tronqués à gauche
  const fileContent = hasFilePath ? (
    <>
      {dir && <span className={`tb-item-dirs${activeClass}`}>{dir}</span>}
      <span
        className="tb-item-name"
        role={clickablePath ? "button" : undefined}
        tabIndex={clickablePath ? 0 : undefined}
        onClick={handlePreviewClick}
        onKeyDown={handlePreviewKeyDown}
      >
        {fileName && <FileIcon name={fileName} size="var(--icon-sm)" />}
        <span className={`tb-item-name-text${activeClass}`}>{fileName ?? shownSummary}</span>
      </span>
      {additions != null && deletions != null && (
        <span className="tb-change-stats">
          <span className="tb-change-add">+{additions}</span>
          <span className="tb-change-del">-{deletions}</span>
        </span>
      )}
    </>
  ) : null;

  // Cas non-fichier : résumé simple tronquable
  const summaryContent = !hasFilePath ? (
    <span
      className={`tb-item-summary${activeClass}`}
      role={clickablePath ? "button" : undefined}
      tabIndex={clickablePath ? 0 : undefined}
      onClick={handlePreviewClick}
      onKeyDown={handlePreviewKeyDown}
    >{shownSummary}</span>
  ) : null;

  return (
    <div>
      <div className={`tb-row${isActive ? " stream-active" : ""}`}>
        {labelButton}
        {fileContent}
        {summaryContent}
        {!done && <Spinner size="var(--icon-sm)" className="tb-spinner" />}
        {done && isError && (
          <ToolStatusIcon
            size="var(--icon-sm)"
            message={errorMessage}
          />
        )}
      </div>
      {canToggle && (
        <div className={`tb-accordion${isOpen ? " tb-open" : ""}`} onTransitionEnd={onTransitionEnd}>
          {mounted && (
            <div className="tb-accordion-inner">
              {showCommandPreview && <div className="chat-column-surface tb-command-preview">{summary}</div>}
              {hasPreview && children}
              {hasResult && (
                MARKDOWN_RESULT_TOOLS.has(name) ? (
                  <ToolResultMarkdown content={result} />
                ) : TEXT_RESULT_TOOLS.has(name) ? (
                  <ToolResultCode content={result} path={summary} />
                ) : (
                  <div className="chat-column-surface tb-result-preview">{result}</div>
                )
              )}
            </div>
          )}
        </div>
      )}
    </div>
  );
}
