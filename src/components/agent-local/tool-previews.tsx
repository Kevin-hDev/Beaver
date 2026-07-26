import { useEffect, useMemo, useState } from "react";
import { highlightLines } from "@/lib/highlight";
import { shouldWrapFile } from "@/lib/code-language";
import { extractDiffContext } from "@/lib/diff-context";
import { readFilePreview } from "@/services/file-preview";
import "./tool-previews.css";
import "@/components/file-preview/file-preview-highlight.css";

type LineMode = "ok" | "error" | "context";

function CodeLines({ lines, mode, path, startLine = 1 }: {
  lines: string[];
  mode: LineMode;
  path?: string;
  startLine?: number;
}) {
  return (
    <>
      {lines.map((line, i) => (
        <div key={`${mode}-${i}`} className={`tp-line tp-line-${mode}`}>
          <span className="tp-num">{startLine + i}</span>
          {path
            ? <span className={`tp-code tp-code-${mode}`} dangerouslySetInnerHTML={{ __html: line || " " }} />
            : <span className={`tp-code tp-code-${mode}`}>{line}</span>
          }
        </div>
      ))}
    </>
  );
}

function useDiffInfo(path?: string, newText?: string, oldText?: string, fallback?: number) {
  const [startLine, setStartLine] = useState(fallback ?? 1);
  const [content, setContent] = useState<string | null>(null);

  useEffect(() => {
    if (!path) return;

    readFilePreview(path)
      .then((fileContent) => {
        setContent(fileContent);
        if (!fallback) {
          const needle = newText ?? oldText ?? "";
          if (!needle) return;
          const idx = fileContent.indexOf(needle);
          if (idx >= 0) {
            setStartLine(fileContent.slice(0, idx).split("\n").length);
          }
        }
      })
      .catch(() => {
        setContent(null);
      });
  }, [path, newText, oldText, fallback]);

  return { startLine: fallback ?? startLine, content };
}

export function ContentPreview({ content, path }: { content: string; path?: string }) {
  const lines = useMemo(() => path ? highlightLines(content, path) : content.split("\n"), [content, path]);
  const wrap = !path || shouldWrapFile(path);

  if (wrap) {
    return (
      <div className="chat-column-surface tp-wrapper">
        <CodeLines lines={lines} mode="ok" path={path} />
      </div>
    );
  }

  return (
    <div className="chat-column-surface tp-wrapper tp-nowrap">
      <div className="tp-inner">
        <CodeLines lines={lines} mode="ok" path={path} />
      </div>
    </div>
  );
}

export function DiffPreview({ oldText, newText, path, startLine }: {
  oldText: string;
  newText: string;
  path?: string;
  startLine?: number;
}) {
  const { startLine: resolvedLine, content } = useDiffInfo(path, newText, oldText, startLine);
  const oldLines = useMemo(() => path ? highlightLines(oldText, path) : oldText.split("\n"), [oldText, path]);
  const newLines = useMemo(() => path ? highlightLines(newText, path) : newText.split("\n"), [newText, path]);

  const ctx = useMemo(() => {
    if (!content || !path) return null;
    return extractDiffContext(content, resolvedLine, newText.split("\n").length);
  }, [content, path, resolvedLine, newText]);

  const beforeLines = useMemo(
    () => ctx?.before.length ? highlightLines(ctx.before.join("\n"), path!) : [],
    [ctx, path],
  );
  const afterLines = useMemo(
    () => ctx?.after.length ? highlightLines(ctx.after.join("\n"), path!) : [],
    [ctx, path],
  );

  const wrap = !path || shouldWrapFile(path);
  const inner = (
    <>
      {beforeLines.length > 0 && ctx && (
        <CodeLines lines={beforeLines} mode="context" path={path} startLine={ctx.beforeStartLine} />
      )}
      <CodeLines lines={oldLines} mode="error" path={path} startLine={resolvedLine} />
      <CodeLines lines={newLines} mode="ok" path={path} startLine={resolvedLine} />
      {afterLines.length > 0 && ctx && (
        <CodeLines lines={afterLines} mode="context" path={path} startLine={ctx.afterStartLine} />
      )}
    </>
  );

  if (wrap) {
    return <div className="chat-column-surface tp-wrapper">{inner}</div>;
  }
  return (
    <div className="chat-column-surface tp-wrapper tp-nowrap">
      <div className="tp-inner">{inner}</div>
    </div>
  );
}

export function WebResultsPreview({ content, isSearch }: { content: string; isSearch: boolean }) {
  if (isSearch) {
    const blocks = content.split("\n\n").filter(Boolean);
    return (
      <div className="chat-column-surface tp-web-wrapper">
        {blocks.slice(0, 8).map((block, i) => {
          const lines = block.split("\n");
          const title = (lines[0] ?? "").replace(/\*\*/g, "");
          const url = lines[1] ?? "";
          const favicon = faviconUrl(url);
          return (
            <div key={i} className="tp-web-item">
              <div className="tp-web-title">{title}</div>
              <div className="tp-web-url">
                <span className="tp-web-url-text">{url}</span>
                {favicon && (
                  <img
                    className="tp-web-favicon"
                    src={favicon}
                    alt=""
                    loading="lazy"
                    onError={(e) => { e.currentTarget.style.display = "none"; }}
                  />
                )}
              </div>
            </div>
          );
        })}
      </div>
    );
  }
  const preview = content.length > 300 ? content.slice(0, 300) + "..." : content;
  return (
    <div className="chat-column-surface tp-web-fetch">
      {preview}
    </div>
  );
}

function faviconUrl(rawUrl: string): string | null {
  try {
    const parsed = new URL(rawUrl);
    if (parsed.protocol !== "http:" && parsed.protocol !== "https:") return null;
    return `${parsed.origin}/favicon.ico`;
  } catch {
    return null;
  }
}
