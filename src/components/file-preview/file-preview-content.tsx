import { lazy, Suspense, useCallback, useEffect, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { listen } from "@tauri-apps/api/event";
import { readFilePreview, readGitFilePreview } from "@/services/file-preview";
import { highlightLines } from "@/lib/highlight";
import { cleanupTauriListener } from "@/lib/tauri-listen";
import type { FileOperation } from "@/types/file-preview";
import { FilePreviewDiff } from "./file-preview-diff";
import { GitDiffPreview, RecordedDiffPreview } from "./git-diff-preview";
import "@/components/agent-local/tool-previews.css";

const SpreadsheetPreview = lazy(() =>
  import("./spreadsheet-preview").then((m) => ({ default: m.SpreadsheetPreview })),
);
const DocumentPreview = lazy(() =>
  import("./document-preview").then((m) => ({ default: m.DocumentPreview })),
);
const PdfPreview = lazy(() =>
  import("./pdf-preview").then((m) => ({ default: m.PdfPreview })),
);

const SPREADSHEET_EXTS = new Set(["xlsx", "xls", "csv", "ods", "xlsm", "tsv"]);

function fileExt(path: string): string {
  return path.split(".").pop()?.toLowerCase() ?? "";
}

interface FilePreviewContentProps {
  operation: FileOperation;
  baseDir?: string;
}

export function FilePreviewContent({ operation, baseDir }: FilePreviewContentProps) {
  const { t } = useTranslation();
  const ext = fileExt(operation.path);

  if (operation.recordedStatus) {
    return (
      <RecordedDiffPreview
        data={operation.recordedDiff}
        path={operation.path}
        status={operation.recordedStatus}
      />
    );
  }

  if (SPREADSHEET_EXTS.has(ext)) {
    return (
      <Suspense fallback={<div className="fp-empty">{t("filePreview.loading")}</div>}>
        <SpreadsheetPreview path={operation.path} baseDir={baseDir} savedOps={operation.content} source={operation.source} />
      </Suspense>
    );
  }
  if (ext === "docx") {
    return (
      <Suspense fallback={<div className="fp-empty">{t("filePreview.loading")}</div>}>
        <DocumentPreview path={operation.path} baseDir={baseDir} savedBlocks={operation.content} source={operation.source} />
      </Suspense>
    );
  }
  if (ext === "pdf") {
    return (
      <Suspense fallback={<div className="fp-empty">{t("filePreview.loading")}</div>}>
        <PdfPreview path={operation.path} baseDir={baseDir} source={operation.source} />
      </Suspense>
    );
  }

  if (operation.gitDiff) {
    return <GitDiffPreview source={operation.gitDiff} path={operation.path} baseDir={baseDir} />;
  }

  return <TextPreviewContent operation={operation} baseDir={baseDir} />;
}

function TextPreviewContent({ operation, baseDir }: FilePreviewContentProps) {
  const { t } = useTranslation();
  const hasSavedContent = operation.type === "write" && !!operation.content;

  const [state, setState] = useState<{ loading: boolean; content: string; error: boolean }>({
    loading: !hasSavedContent,
    content: hasSavedContent ? operation.content! : "",
    error: false,
  });

  useEffect(() => {
    if (hasSavedContent) return;
    let alive = true;
    // eslint-disable-next-line react-hooks/set-state-in-effect -- fetch→setState is intentional
    setState({ loading: true, content: "", error: false });
    readTextPreview(operation.path, operation.source, baseDir)
      .then((content) => {
        if (alive) setState({ loading: false, content, error: false });
      })
      .catch(() => {
        if (alive) setState({ loading: false, content: "", error: true });
      });
    return () => { alive = false; };
  }, [operation.path, operation.source, baseDir, hasSavedContent]);

  const reload = useCallback(() => {
    if (hasSavedContent) return;
    readTextPreview(operation.path, operation.source, baseDir)
      .then((content) => setState({ loading: false, content, error: false }))
      .catch(() => {});
  }, [operation.path, operation.source, baseDir, hasSavedContent]);

  useEffect(() => {
    if (operation.source) return;
    const unlisten = listen<{ path: string }>("file-tree-changed", (event) => {
      const changedDir = event.payload.path;
      const parentDir = operation.path.substring(0, operation.path.lastIndexOf("/"));
      if (changedDir === parentDir || operation.path.startsWith(changedDir)) {
        reload();
      }
    });
    return () => { cleanupTauriListener(unlisten); };
  }, [operation.path, operation.source, reload]);

  const highlighted = useMemo(
    () => state.content ? highlightLines(state.content, operation.path) : [],
    [state.content, operation.path],
  );

  if (state.loading) {
    return <div className="fp-empty">{t("filePreview.loading")}</div>;
  }
  if (state.error) {
    return <div className="fp-empty">{t("filePreview.fileNotFound")}</div>;
  }
  if (operation.type === "edit") {
    return <FilePreviewDiff operation={operation} currentContent={state.content} />;
  }

  return (
    <div className="tp-wrapper" style={{ margin: 0, border: "none", borderRadius: 0 }}>
      {highlighted.map((html, i) => (
        <div key={i} className="tp-line tp-line-context">
          <span className="tp-num">{i + 1}</span>
          <span className="tp-prefix tp-prefix-context"> </span>
          <span className="tp-code tp-code-context" dangerouslySetInnerHTML={{ __html: html || " " }} />
        </div>
      ))}
    </div>
  );
}

async function readTextPreview(
  path: string,
  source: FileOperation["source"],
  baseDir?: string,
) {
  return source
    ? readGitFilePreview(source, baseDir)
    : readFilePreview(path, baseDir);
}
