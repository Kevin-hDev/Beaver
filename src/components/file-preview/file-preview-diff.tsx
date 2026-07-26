import { useMemo } from "react";
import { highlightLines } from "@/lib/highlight";
import { shouldWrapFile } from "@/lib/code-language";
import { extractDiffContext } from "@/lib/diff-context";
import type { FileOperation } from "@/types/file-preview";
import "@/components/agent-local/tool-previews.css";

function findStartLine(content: string, needle: string): number {
  if (!needle) return 1;
  const idx = content.indexOf(needle);
  if (idx < 0) return 1;
  return content.slice(0, idx).split("\n").length;
}

function DiffLine({ html, lineNum, mode }: { html: string; lineNum: number; mode: "ok" | "error" | "context" }) {
  const prefix = mode === "ok" ? "+" : mode === "error" ? "-" : " ";
  return (
    <div className={`tp-line tp-line-${mode}`}>
      <span className="tp-num">{lineNum}</span>
      <span className={`tp-prefix tp-prefix-${mode}`}>{prefix}</span>
      <span className={`tp-code tp-code-${mode}`} dangerouslySetInnerHTML={{ __html: html || " " }} />
    </div>
  );
}

export function FilePreviewDiff({
  operation,
  currentContent,
}: {
  operation: FileOperation;
  currentContent: string;
}) {
  const oldLines = useMemo(
    () => highlightLines(operation.oldText ?? "", operation.path),
    [operation.oldText, operation.path],
  );
  const newLines = useMemo(
    () => highlightLines(operation.newText ?? "", operation.path),
    [operation.newText, operation.path],
  );

  const startLine = useMemo(
    () => operation.startLine ?? findStartLine(currentContent, operation.newText ?? operation.oldText ?? ""),
    [currentContent, operation.startLine, operation.newText, operation.oldText],
  );

  const ctx = useMemo(
    () => extractDiffContext(currentContent, startLine, newLines.length),
    [currentContent, startLine, newLines.length],
  );

  const beforeLines = useMemo(
    () => ctx.before.length > 0 ? highlightLines(ctx.before.join("\n"), operation.path) : [],
    [ctx.before, operation.path],
  );
  const afterLines = useMemo(
    () => ctx.after.length > 0 ? highlightLines(ctx.after.join("\n"), operation.path) : [],
    [ctx.after, operation.path],
  );

  const wrap = shouldWrapFile(operation.path);
  const inner = (
    <>
      {beforeLines.length > 0 && beforeLines.map((html, i) => (
        <DiffLine key={`ctx-b-${i}`} html={html} lineNum={ctx.beforeStartLine + i} mode="context" />
      ))}
      {oldLines.map((html, i) => (
        <DiffLine key={`old-${i}`} html={html} lineNum={startLine + i} mode="error" />
      ))}
      {newLines.map((html, i) => (
        <DiffLine key={`new-${i}`} html={html} lineNum={startLine + i} mode="ok" />
      ))}
      {afterLines.length > 0 && afterLines.map((html, i) => (
        <DiffLine key={`ctx-a-${i}`} html={html} lineNum={ctx.afterStartLine + i} mode="context" />
      ))}
    </>
  );

  if (wrap) {
    return <div className="tp-wrapper" style={{ margin: 0, border: "none", borderRadius: 0 }}>{inner}</div>;
  }
  return (
    <div className="tp-wrapper tp-nowrap" style={{ margin: 0, border: "none", borderRadius: 0 }}>
      <div className="tp-inner">{inner}</div>
    </div>
  );
}
